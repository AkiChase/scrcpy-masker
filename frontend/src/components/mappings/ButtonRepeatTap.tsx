import { useEffect, useState } from "react";
import type { RepeatTapConfig } from "./mapping";
import { Flex, InputNumber, Tooltip, Typography } from "antd";
import {
  mappingButtonDragFactory,
  mappingButtonPresetStyle,
  mappingButtonTransformStyle,
} from "./tools";
import { useAppSelector } from "../../store/store";
import { ItemBoxContainer, ItemBox } from "../common/ItemBox";
import {
  SettingBind,
  SettingDelete,
  SettingModal,
  SettingNote,
  SettingPointerId,
} from "./Common";
import { useTranslation } from "react-i18next";

const PRESET_STYLE = mappingButtonPresetStyle(52);

export default function ButtonRepeatTap({
  index,
  config,
  originalSize,
  onConfigChange,
  onConfigDelete,
}: {
  index: number;
  config: RepeatTapConfig;
  originalSize: { width: number; height: number };
  onConfigChange: (config: RepeatTapConfig) => void;
  onConfigDelete: () => void;
}) {
  const id = `mapping-repeat-tap-${index}`;
  const bindText = config.bind.join("+");
  const className =
    "rounded-full absolute box-border border-solid border-2 color-text " +
    (config.bind.length > 0
      ? "border-text-secondary hover:border-text"
      : "border-primary hover:border-primary-hover");

  const maskArea = useAppSelector((state) => state.other.maskArea);
  const [showSetting, setShowSetting] = useState(false);

  useEffect(() => {
    const element = document.getElementById(id);
    if (element) {
      element.style.transform = mappingButtonTransformStyle(
        config.position.x,
        config.position.y,
        originalSize.width,
        originalSize.height,
        maskArea.width,
        maskArea.height
      );
    }
  }, [maskArea, index, config, originalSize]);

  const handleDrag = mappingButtonDragFactory(
    maskArea,
    originalSize,
    ({ x, y }) => {
      onConfigChange({
        ...config,
        position: {
          x,
          y,
        },
      });
    }
  );

  const handleSetting = (e: React.MouseEvent) => {
    if (e.button != 0) return;
    e.preventDefault();
    setShowSetting(true);
  };

  return (
    <>
      <SettingModal open={showSetting} onClose={() => setShowSetting(false)}>
        <Setting
          config={config}
          onConfigChange={onConfigChange}
          onConfigDelete={() => {
            setShowSetting(false);
            onConfigDelete();
          }}
        />
      </SettingModal>
      <Flex
        id={id}
        style={PRESET_STYLE}
        className={className}
        onMouseDown={handleDrag}
        onDoubleClick={handleSetting}
        justify="center"
        align="center"
      >
        <Tooltip trigger="click" title={`${config.type}: ${bindText}`}>
          <Typography.Text ellipsis={true} className="text-2.5 font-bold">
            {bindText}
          </Typography.Text>
        </Tooltip>
      </Flex>
    </>
  );
}

function Setting({
  config,
  onConfigChange,
  onConfigDelete,
}: {
  config: RepeatTapConfig;
  onConfigChange: (config: RepeatTapConfig) => void;
  onConfigDelete: () => void;
}) {
  const { t } = useTranslation();

  return (
    <div>
      <h1 className="title-with-line">
        {t("mappings.repeatTap.setting.title")}
      </h1>
      <ItemBoxContainer className="max-h-70vh overflow-y-auto pr-2 scrollbar">
        <SettingBind
          bind={config.bind}
          onBindChange={(bind) => onConfigChange({ ...config, bind })}
        />
        <SettingPointerId
          pointerId={config.pointer_id}
          onPointerIdChange={(pointerId) =>
            onConfigChange({ ...config, pointer_id: pointerId })
          }
        />
        <ItemBox label={t("mappings.repeatTap.setting.duration")}>
          <InputNumber
            className="w-full"
            value={config.duration}
            min={0}
            onChange={(v) =>
              v !== null && onConfigChange({ ...config, duration: v })
            }
          />
        </ItemBox>
        <ItemBox label={t("mappings.repeatTap.setting.interval")}>
          <InputNumber
            className="w-full"
            value={config.interval}
            min={0}
            onChange={(v) =>
              v !== null && onConfigChange({ ...config, interval: v })
            }
          />
        </ItemBox>
        <SettingNote
          note={config.note}
          onNoteChange={(note) => onConfigChange({ ...config, note })}
        />
        <SettingDelete onDelete={onConfigDelete} />
      </ItemBoxContainer>
    </div>
  );
}
