import type { MappingConfig, MappingType } from "./mapping";

import {
  Badge,
  Button,
  Dropdown,
  Flex,
  Modal,
  Select,
  Space,
  Table,
  type TableProps,
} from "antd";
import { useEffect, useMemo, useRef, useState } from "react";
import { useAppDispatch, useAppSelector } from "../../store/store";
import {
  CheckCircleOutlined,
  CopyOutlined,
  DeleteOutlined,
  EditOutlined,
  FileAddOutlined,
  FileSyncOutlined,
  SaveOutlined,
  SettingOutlined,
  SnippetsOutlined,
} from "@ant-design/icons";
import IconButton from "../common/IconButton";
import { deepClone, throttle } from "../../utils";
import { useMessageContext, useRefreshBackgroundImage } from "../../hooks";
import ButtonSingleTap from "./ButtonSingleTap";
import { setMaskArea } from "../../store/other";
import ButtonRepeatTap from "./ButtonRepeatTap";
import ButtonMultipleTap from "./ButtonMultipleTap";
import { clientPositionToMappingPosition } from "./tools";
import ButtonSwipe from "./ButtonSwipe";
import ButtonDirectionPad from "./ButtonDirectionPad";
import ButtonMouseCastSpell from "./ButtonMouseCastSpell";
import { CursorPos, DeviceBackground, RefreshImageButton } from "./Common";

type MappingFileTabelItem = {
  file: string;
  active: boolean;
  displayed: boolean;
};

function Manager({
  open,
  onCancel,
  mappingList,
  displayedMapping,
  onActiveAction,
  onDisplayAction,
}: {
  open: boolean;
  onCancel: () => void;
  mappingList: string[];
  displayedMapping: string;
  onActiveAction: (file: string) => void;
  onDisplayAction: (file: string) => void;
}) {
  const activeMappingFile = useAppSelector(
    (state) => state.localConfig.activeMappingFile
  );

  const mappingFiles = useMemo<MappingFileTabelItem[]>(() => {
    return mappingList.map((file) => {
      return {
        file,
        active: file === activeMappingFile,
        displayed: file === displayedMapping,
      };
    });
  }, [mappingList, activeMappingFile, displayedMapping]);

  const columns: TableProps<MappingFileTabelItem>["columns"] = [
    {
      title: (
        <Space size="large">
          文件
          <IconButton color="info" tooltip="新建" icon={<FileAddOutlined />} />
        </Space>
      ),
      dataIndex: "file",
      key: "file",
      render: (_, record) => (
        <Flex align="center" justify="space-between" className="p-r-3">
          <span>{record.file}</span>
          <Space size={32}>
            {record.displayed && <Badge status="processing" text="编辑" />}
            {record.active && <Badge status="success" text="启用" />}
          </Space>
        </Flex>
      ),
    },
    {
      title: "操作",
      key: "action",
      align: "center",
      width: 1,
      render: (_, record) => (
        <Space size="middle" className="text-4">
          <IconButton
            color="info"
            icon={<EditOutlined />}
            tooltip="编辑"
            onClick={() => onActiveAction(record.file)}
          />
          <IconButton
            color="success"
            tooltip="启用"
            icon={<CheckCircleOutlined />}
            onClick={() => onDisplayAction(record.file)}
          />
          <IconButton
            color="error"
            tooltip="删除"
            icon={<DeleteOutlined />}
            onClick={() => onDisplayAction(record.file)}
          />
          <IconButton
            color="info"
            tooltip="复制"
            icon={<CopyOutlined />}
            onClick={() => onDisplayAction(record.file)}
          />
          <IconButton
            color="info"
            tooltip="迁移到当前设备尺寸"
            icon={<SnippetsOutlined />}
            onClick={() => onDisplayAction(record.file)}
          />
        </Space>
      ),
    },
  ];

  return (
    <Modal
      title="配置管理"
      className="min-w-50vw"
      open={open}
      onCancel={onCancel}
      footer={null}
    >
      <Table<MappingFileTabelItem>
        size="small"
        rowKey={(record) => record.file}
        pagination={{ pageSize: 7 }}
        columns={columns}
        dataSource={mappingFiles}
      />
    </Modal>
  );
}

type EditState = {
  file: string;
  edited: boolean;
  current: MappingConfig;
  old: MappingConfig;
};

const mappingButtonMap = {
  SingleTap: ButtonSingleTap,
  RepeatTap: ButtonRepeatTap,
  MultipleTap: ButtonMultipleTap,
  Swipe: ButtonSwipe,
  DirectionPad: ButtonDirectionPad,
  MouseCastSpell: ButtonMouseCastSpell,
};

function Displayer({
  state,
  setState,
}: {
  state: EditState;
  setState: React.Dispatch<React.SetStateAction<EditState | null>>;
}) {
  const dispatch = useAppDispatch();
  const maskArea = useAppSelector((state) => state.other.maskArea);

  const cursorPosRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const displayerElement = document.getElementById(
      "mapping-displayer"
    ) as HTMLElement;

    const observer = new ResizeObserver(() => {
      const rect = displayerElement.getBoundingClientRect();
      dispatch(
        setMaskArea({
          width: rect.width - 2,
          height: rect.height - 2,
          left: rect.left + 1,
          top: rect.top + 1,
        })
      );
    });
    observer.observe(displayerElement);

    return () => {
      observer.disconnect();
    };
  }, []);

  function updateMapping(index: number, config: MappingType) {
    setState((prev) => {
      if (prev === null) return null;
      const newMappings = [...prev.current.mappings];
      newMappings[index] = deepClone(config);

      return {
        ...prev,
        edited: true,
        current: {
          ...prev.current,
          mappings: newMappings,
        },
      };
    });
  }

  function deleteMapping(index: number) {
    setState((prev) => {
      if (prev === null) return null;
      const newMappings = [...prev.current.mappings];
      newMappings.splice(index, 1);

      return {
        ...prev,
        edited: true,
        current: {
          ...prev.current,
          mappings: newMappings,
        },
      };
    });
  }

  const handleMouseMove = throttle((e: React.MouseEvent) => {
    if (cursorPosRef.current) {
      const { x, y } = clientPositionToMappingPosition(
        e.clientX,
        e.clientY,
        maskArea.left,
        maskArea.top,
        maskArea.width,
        maskArea.height,
        state.current.original_size.width,
        state.current.original_size.height
      );
      cursorPosRef.current.innerText = `(${x},${y})`;
    }
  }, 100);

  return (
    <div
      id="mapping-displayer"
      className="w-full h-full border-text-quaternary border-solid border relative select-none"
      onMouseMove={handleMouseMove}
    >
      <DeviceBackground />
      <Dropdown
        menu={{
          items: [
            {
              label: "添加按钮",
              key: "1",
            },
          ],
        }}
        trigger={["contextMenu"]}
      >
        <div className="w-full h-full absolute bg-transparent"></div>
      </Dropdown>
      <CursorPos ref={cursorPosRef} />
      {state.current.mappings.map((mapping, index) => {
        const props: any = {
          originalSize: state.current.original_size,
          index,
          config: mapping,
          onConfigChange: (config: any) => updateMapping(index, config),
          onConfigDelete: () => deleteMapping(index),
        };

        if (mapping.type in mappingButtonMap) {
          const ButtonComponent =
            mappingButtonMap[mapping.type as keyof typeof mappingButtonMap];
          return <ButtonComponent key={index} {...props} />;
        }

        return <div key={index}></div>;
      })}
    </div>
  );
}

export default function Mappings() {
  const messageApi = useMessageContext();
  const activeMappingFile = useAppSelector(
    (state) => state.localConfig.activeMappingFile
  );
  const refreshBackground = useRefreshBackgroundImage();

  const [displayedMappingFile, setDisplayedMappingFile] = useState("");
  const [isManagerOpen, setIsManagerOpen] = useState(false);
  const [editState, setEditState] = useState<EditState | null>(null);
  const [mappingList, setMappingList] = useState<string[]>([]);

  const mappingListOptions = useMemo(() => {
    return mappingList.map((item) => ({
      label: (
        <Flex justify="space-between" align="center">
          <span>{item}</span>
          {activeMappingFile === item && <Badge color="green" text="启用" />}
        </Flex>
      ),
      value: item,
    }));
  }, [mappingList]);

  useEffect(() => {
    setMappingList(mappingListTMP);
    setDisplayedMapping(activeMappingFile);

    refreshBackground();
  }, []);

  async function setDisplayedMapping(file: string) {
    try {
      // TODO 发送请求获取mappingconfig
      const mappingConfig = deepClone(mappingConfigTMP);
      setDisplayedMappingFile(file);
      setEditState({
        file: displayedMappingFile,
        edited: false,
        current: mappingConfig,
        old: deepClone(mappingConfig),
      });
    } catch (error: any) {
      messageApi?.error(error);
    }
  }

  return (
    <Flex vertical gap={32} className="page-container hide-scrollbar">
      <Manager
        open={isManagerOpen}
        onCancel={() => setIsManagerOpen(false)}
        mappingList={mappingList}
        displayedMapping={displayedMappingFile}
        onActiveAction={(file) => console.log("active change", file)}
        onDisplayAction={(file) => console.log("display change", file)}
      />
      <section>
        <Flex justify="space-between" align="center">
          <Space.Compact>
            <Select
              className="w-80"
              showSearch
              value={displayedMappingFile}
              onChange={(value) => setDisplayedMapping(value)}
              options={mappingListOptions}
            />
            <Button
              type="primary"
              disabled={editState === null || editState.edited === false}
              icon={<SaveOutlined />}
              onClick={() =>
                console.log("保存当前映射文件，注意进行校验特别是按键合法性")
              }
            >
              保存
            </Button>
            <Button
              type="primary"
              icon={<CheckCircleOutlined />}
              onClick={() => console.log("启用当前映射文件")}
            >
              启用
            </Button>
            <Button
              type="primary"
              icon={<FileSyncOutlined />}
              onClick={() => console.log("刷新mapping列表")}
            >
              刷新
            </Button>

            <Button
              type="primary"
              icon={<SettingOutlined />}
              onClick={() => setIsManagerOpen(true)}
            >
              管理
            </Button>
          </Space.Compact>
          <RefreshImageButton />
        </Flex>
      </section>
      <section className="flex-grow-1 flex-shrink-0 pb-4">
        {editState && <Displayer state={editState} setState={setEditState} />}
      </section>
    </Flex>
  );
}

const mappingListTMP = [
  "test.json",
  "default.json",
  "test2.json",
  "test3.json",
  "test4.json",
  "test5.json",
  "test6.json",
  "test7.json",
  "test8.json",
  "test9.json",
  "test10.json",
  "test11.json",
  "test12.json",
  "test13.json",
  "test14.json",
  "test15.json",
  "test16.json",
  "test17.json",
  "test18.json",
];
const mappingConfigTMP: MappingConfig = {
  mappings: [
    {
      bind: ["Digit1", "Digit2", "Digit3", "Digit4", "Digit5"],
      duration: 1000,
      note: "SingleTap",
      pointer_id: 1,
      position: {
        x: 100,
        y: 100,
      },
      sync: false,
      type: "SingleTap",
    },
    {
      bind: ["Digit2"],
      duration: 0,
      note: "SingleTap (sync)",
      pointer_id: 1,
      position: {
        x: 200,
        y: 100,
      },
      sync: true,
      type: "SingleTap",
    },
    {
      bind: ["Digit3"],
      duration: 30,
      interval: 100,
      note: "RepeatTap",
      pointer_id: 1,
      position: {
        x: 250,
        y: 200,
      },
      type: "RepeatTap",
    },
    {
      bind: ["ControlLeft", "Digit3"],
      duration: 30,
      interval: 100,
      note: "RepeatTap (multi-binding)",
      pointer_id: 2,
      position: {
        x: 250,
        y: 250,
      },
      type: "RepeatTap",
    },
    {
      bind: ["Digit4"],
      items: [
        {
          duration: 500,
          position: {
            x: 100,
            y: 100,
          },
          wait: 0,
        },
        {
          duration: 500,
          position: {
            x: 200,
            y: 200,
          },
          wait: 1000,
        },
        {
          duration: 500,
          position: {
            x: 300,
            y: 300,
          },
          wait: 1000,
        },
      ],
      note: "MultipleTap",
      pointer_id: 1,
      type: "MultipleTap",
    },
    {
      bind: ["Digit5"],
      interval: 1000,
      note: "Swipe",
      pointer_id: 1,
      positions: [
        {
          x: 100,
          y: 100,
        },
        {
          x: 200,
          y: 200,
        },
        {
          x: 300,
          y: 300,
        },
      ],
      type: "Swipe",
    },
    {
      bind: {
        down: ["KeyS"],
        left: ["KeyA"],
        right: ["KeyD"],
        type: "Button",
        up: ["KeyW"],
      },
      initial_duration: 100,
      max_offset_x: 100,
      max_offset_y: 100,
      note: "DirectionPad",
      pointer_id: 9,
      position: {
        x: 300,
        y: 1000,
      },
      type: "DirectionPad",
    },
    {
      bind: {
        type: "JoyStick",
        x: "LeftStickX",
        y: "LeftStickY",
      },
      initial_duration: 300,
      max_offset_x: 100,
      max_offset_y: 100,
      note: "DirectionPad gamepad",
      pointer_id: 9,
      position: {
        x: 500,
        y: 1000,
      },
      type: "DirectionPad",
    },
    {
      bind: ["KeyE"],
      cast_no_direction: true,
      cast_radius: 625,
      center: {
        x: 1280,
        y: 815,
      },
      drag_radius: 150,
      horizontal_scale_factor: 7,
      note: "MouseCastSpell (no direction)",
      pointer_id: 3,
      position: {
        x: 1900,
        y: 1150,
      },
      release_mode: "OnRelease",
      type: "MouseCastSpell",
      vertical_scale_factor: 10,
    },
    {
      bind: ["KeyQ"],
      cast_no_direction: false,
      cast_radius: 625,
      center: {
        x: 1280,
        y: 815,
      },
      drag_radius: 150,
      horizontal_scale_factor: 7,
      note: "MouseCastSpell (press to release)",
      pointer_id: 3,
      position: {
        x: 1900,
        y: 1150,
      },
      release_mode: "OnPress",
      type: "MouseCastSpell",
      vertical_scale_factor: 10,
    },
    {
      bind: ["AltLeft"],
      cast_no_direction: true,
      cast_radius: 625,
      center: {
        x: 1280,
        y: 815,
      },
      drag_radius: 150,
      horizontal_scale_factor: 7,
      note: "MouseCastSpell (second press to release)",
      pointer_id: 3,
      position: {
        x: 2100,
        y: 1030,
      },
      release_mode: "OnSecondPress",
      type: "MouseCastSpell",
      vertical_scale_factor: 10,
    },
    {
      bind: ["M-Back"],
      cast_no_direction: false,
      cast_radius: 625,
      center: {
        x: 1280,
        y: 815,
      },
      drag_radius: 150,
      horizontal_scale_factor: 7,
      note: "MouseCastSpell",
      pointer_id: 3,
      position: {
        x: 2250,
        y: 900,
      },
      release_mode: "OnRelease",
      type: "MouseCastSpell",
      vertical_scale_factor: 10,
    },
    {
      bind: ["KeyJ"],
      block_direction_pad: true,
      drag_radius: 150,
      note: "PadCastSpell",
      pad_action: "PadCastDirection1",
      pad_bind: {
        down: ["KeyS"],
        left: ["KeyA"],
        right: ["KeyD"],
        type: "Button",
        up: ["KeyW"],
      },
      pointer_id: 3,
      position: {
        x: 2000,
        y: 750,
      },
      release_mode: "OnRelease",
      type: "PadCastSpell",
    },
    {
      bind: ["Space"],
      note: "CancelCast",
      position: {
        x: 2200,
        y: 175,
      },
      type: "CancelCast",
    },
    {
      bind: ["M-Forward"],
      note: "Observation",
      pointer_id: 4,
      position: {
        x: 2000,
        y: 300,
      },
      sensitivity_x: 0.5,
      sensitivity_y: 0.5,
      type: "Observation",
    },
    {
      bind: ["Backquote"],
      note: "FPS",
      pointer_id: 0,
      position: {
        x: 1280,
        y: 720,
      },
      sensitivity_x: 1.2000000476837158,
      sensitivity_y: 1,
      type: "Fps",
    },
    {
      bind: ["M-Left"],
      note: "Fire",
      pointer_id: 1,
      position: {
        x: 2000,
        y: 1000,
      },
      sensitivity_x: 1,
      sensitivity_y: 0.5,
      type: "Fire",
    },
    {
      bind: ["Enter"],
      note: "RawInput",
      position: {
        x: 2000,
        y: 300,
      },
      type: "RawInput",
    },
  ],
  original_size: {
    height: 1440,
    width: 2560,
  },
  title: "Default",
  version: "0.0.1",
};
