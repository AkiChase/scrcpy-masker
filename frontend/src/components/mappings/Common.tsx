import {
  EditOutlined,
  CloseCircleOutlined,
  PlusOutlined,
} from "@ant-design/icons";
import {
  Input,
  Badge,
  Space,
  type InputRef,
  Flex,
  Tag,
  Modal,
  Switch,
  InputNumber,
  Button,
} from "antd";
import { useState, useRef, useEffect, type PropsWithChildren } from "react";
import IconButton from "../common/IconButton";

import type { ButtonBinding } from "./mapping";
import { EVENT_CODE_TO_KEY_CODE } from "./keyCode";
import { debounce } from "../../utils";
import { useTranslation } from "react-i18next";
import { ItemBox } from "../common/ItemBox";

const MOUSE_BUTTONS = ["M-Left", "M-Middle", "M-Right", "M-Forward", "M-Back"];

type SettingModalProps = PropsWithChildren<{
  open: boolean;
  onClose: () => void;
}>;

export function SettingModal({ children, open, onClose }: SettingModalProps) {
  return (
    <Modal
      footer={null}
      open={open}
      onCancel={onClose}
      destroyOnHidden={true}
      keyboard={false}
      className="w-min-50vw"
    >
      {children}
    </Modal>
  );
}

type SettingBindProps = {
  bind: ButtonBinding;
  onBindChange: (bind: ButtonBinding) => void;
};

export function SettingBind({ bind, onBindChange }: SettingBindProps) {
  const { t } = useTranslation();
  const [isManualInput, setIsManualInput] = useState(false);

  return (
    <ItemBox
      label={
        <Flex className="w-full" align="center" justify="space-between">
          <span>{t("mappings.common.bind.settingLabel")}</span>
          <Switch
            size="small"
            checkedChildren={t("mappings.common.bind.settingManual")}
            unCheckedChildren={t("mappings.common.bind.settingAuto")}
            checked={isManualInput}
            onChange={(checked) => setIsManualInput(checked)}
          />
        </Flex>
      }
    >
      <InputBinding
        manual={isManualInput}
        bind={bind}
        onBindChange={onBindChange}
      />
    </ItemBox>
  );
}

type SettingPointerIdProps = {
  pointerId: number;
  onPointerIdChange: (pointerId: number) => void;
};

export function SettingPointerId({
  pointerId,
  onPointerIdChange,
}: SettingPointerIdProps) {
  const { t } = useTranslation();

  return (
    <ItemBox label={t("mappings.common.pointerId.label")}>
      <InputNumber
        className="w-full"
        value={pointerId}
        min={0}
        step={1}
        onChange={(v) => v !== null && onPointerIdChange(v)}
      />
    </ItemBox>
  );
}

type SettingNoteProps = {
  note: string;
  onNoteChange: (note: string) => void;
};

export function SettingNote({ note, onNoteChange }: SettingNoteProps) {
  const { t } = useTranslation();

  return (
    <ItemBox label={t("mappings.common.note.label")}>
      <Input value={note} onChange={(e) => onNoteChange(e.target.value)} />
    </ItemBox>
  );
}

export function SettingDelete({ onDelete }: { onDelete: () => void }) {
  const { t } = useTranslation();

  return (
    <ItemBox>
      <Button block type="primary" onClick={onDelete}>
        {t("mappings.common.delete.label")}
      </Button>
    </ItemBox>
  );
}

function mappingButtonBindFactory(
  inputElement: HTMLElement,
  onBindChange: (bind: ButtonBinding) => void,
  onIsRecordingChange: (isRecording: boolean) => void
) {
  const pressedKeys = new Set<string>();

  const handleKeyDown = (e: KeyboardEvent) => {
    e.preventDefault();
    if (!pressedKeys.has(e.code)) {
      if (e.code in EVENT_CODE_TO_KEY_CODE) {
        pressedKeys.add(
          EVENT_CODE_TO_KEY_CODE[e.code as keyof typeof EVENT_CODE_TO_KEY_CODE]
        );
        onBindChange([...pressedKeys]);
      } else {
        console.warn("Unknow keycode: ", e.code);
      }
    }
  };

  const handleKeyUp = (e: KeyboardEvent) => {
    e.preventDefault();
    if (e.code in EVENT_CODE_TO_KEY_CODE) {
      pressedKeys.delete(
        EVENT_CODE_TO_KEY_CODE[e.code as keyof typeof EVENT_CODE_TO_KEY_CODE]
      );
    }
  };

  const handleMouseDown = (e: MouseEvent) => {
    if (!inputElement.contains(e.target as Node) && e.button === 0) {
      stopRecord();
      return;
    }
    e.preventDefault();

    const key =
      e.button >= 0 && e.button < MOUSE_BUTTONS.length
        ? MOUSE_BUTTONS[e.button]
        : `M-Other-${e.button}`;
    pressedKeys.add(key);
    onBindChange([...pressedKeys]);
  };

  const handleMouseUp = (e: MouseEvent) => {
    e.preventDefault();
    const key =
      e.button >= 0 && e.button < MOUSE_BUTTONS.length
        ? MOUSE_BUTTONS[e.button]
        : `M-Other-${e.button}`;

    pressedKeys.delete(key);
  };

  const handleWheel = (() => {
    const debounced = debounce((deltaY: number) => {
      const key = deltaY > 0 ? "ScrollUp" : "ScrollDown";
      pressedKeys.add(key);
      onBindChange([...pressedKeys]);
      pressedKeys.delete(key);
    }, 50);

    return (e: WheelEvent) => {
      e.preventDefault();

      if (e.deltaY === 0) return;
      debounced(e.deltaY);
    };
  })();

  const handleBlur = () => {
    pressedKeys.clear();
  };

  const handleContextMenu = (e: MouseEvent) => {
    e.preventDefault();
  };

  const startRecord = () => {
    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("keyup", handleKeyUp);
    window.addEventListener("blur", handleBlur);
    window.addEventListener("mousedown", handleMouseDown);
    window.addEventListener("mouseup", handleMouseUp);
    window.addEventListener("contextmenu", handleContextMenu);
    window.addEventListener("wheel", handleWheel, { passive: false });
    onIsRecordingChange(true);
  };

  const stopRecord = () => {
    window.removeEventListener("keydown", handleKeyDown);
    window.removeEventListener("keyup", handleKeyUp);
    window.removeEventListener("blur", handleBlur);
    window.removeEventListener("mousedown", handleMouseDown);
    window.removeEventListener("mouseup", handleMouseUp);
    window.removeEventListener("contextmenu", handleContextMenu);
    window.removeEventListener("wheel", handleWheel);
    onIsRecordingChange(false);
  };

  return {
    startRecord,
    stopRecord,
  };
}

function AutoInputBinding({
  bind,
  onBindChange,
}: {
  bind: ButtonBinding;
  onBindChange: (bind: ButtonBinding) => void;
}) {
  const { t } = useTranslation();
  const [isRecording, setIsRecording] = useState(false);
  const inputRef = useRef<InputRef>(null);
  const startRecord = useRef(() => {});

  useEffect(() => {
    startRecord.current = mappingButtonBindFactory(
      inputRef.current!.nativeElement as HTMLElement,
      onBindChange,
      setIsRecording
    ).startRecord;
  }, []);

  return (
    <Input
      ref={inputRef}
      value={bind.join("+")}
      placeholder={t("mappings.common.bind.autoInputPlaceholder")}
      readOnly
      onDoubleClick={() => {
        return !isRecording && startRecord.current();
      }}
      suffix={
        isRecording ? (
          <Badge color="red" text="Recording..." />
        ) : (
          <Space>
            <IconButton
              size={14}
              icon={<EditOutlined />}
              onClick={startRecord.current}
            />
            <IconButton
              size={14}
              icon={<CloseCircleOutlined />}
              onClick={() => onBindChange([])}
            />
          </Space>
        )
      }
    />
  );
}

function ManualInputBinding({
  bind,
  onBindChange,
}: {
  bind: ButtonBinding;
  onBindChange: (bind: ButtonBinding) => void;
}) {
  const { t } = useTranslation();
  const [inputVisible, setInputVisible] = useState(false);
  const [inputValue, setInputValue] = useState("");
  const [editInputIndex, setEditInputIndex] = useState(-1);
  const [editInputValue, setEditInputValue] = useState("");
  const inputRef = useRef<InputRef>(null);
  const editInputRef = useRef<InputRef>(null);

  useEffect(() => {
    if (inputVisible) inputRef.current?.focus();
  }, [inputVisible]);

  useEffect(() => {
    if (editInputIndex !== -1) editInputRef.current?.focus();
  }, [editInputIndex]);

  const showInput = () => {
    setInputVisible(true);
  };

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setInputValue(e.target.value);
  };

  const handleInputConfirm = () => {
    if (inputValue && !bind.includes(inputValue)) {
      onBindChange([...bind, inputValue]);
    }
    setInputVisible(false);
    setInputValue("");
  };

  const handleEditInputConfirm = () => {
    if (editInputValue && !bind.includes(editInputValue)) {
      const newTags = [...bind];
      newTags[editInputIndex] = editInputValue;
      onBindChange(newTags);
    }
    setEditInputIndex(-1);
    setEditInputValue("");
  };

  return (
    <Flex gap="4px 0" wrap className="pt-2.5">
      {bind.map<React.ReactNode>((btn, index) => {
        if (editInputIndex === index) {
          return (
            <Input
              ref={editInputRef}
              className="w-16"
              key={btn}
              size="small"
              value={editInputValue}
              onChange={(e) => setEditInputValue(e.target.value)}
              onBlur={handleEditInputConfirm}
              onPressEnter={handleEditInputConfirm}
            />
          );
        }
        return (
          <Tag
            key={btn}
            closable
            onClose={() => onBindChange(bind.filter((b) => btn !== b))}
            onDoubleClick={(e) => {
              setEditInputIndex(index);
              setEditInputValue(btn);
              e.preventDefault();
            }}
          >
            {btn}
          </Tag>
        );
      })}
      {inputVisible ? (
        <Input
          ref={inputRef}
          type="text"
          className="w-16"
          size="small"
          value={inputValue}
          onChange={handleInputChange}
          onBlur={handleInputConfirm}
          onPressEnter={handleInputConfirm}
        />
      ) : (
        <Tag icon={<PlusOutlined />} onClick={showInput}>
          {t("mappings.common.bind.manualInputNew")}
        </Tag>
      )}
    </Flex>
  );
}

function InputBinding({
  bind,
  onBindChange,
  manual,
}: {
  bind: ButtonBinding;
  onBindChange: (bind: ButtonBinding) => void;
  manual: boolean;
}) {
  return manual ? (
    <ManualInputBinding bind={bind} onBindChange={onBindChange} />
  ) : (
    <AutoInputBinding bind={bind} onBindChange={onBindChange} />
  );
}
