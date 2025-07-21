import { useEffect, useState } from "react";
import type { SwipeConfig } from "./mapping";
import { Button, Flex, Popover, Tooltip, Typography } from "antd";
import {
  clientPositionToMappingPosition,
  mappingButtonDragFactory,
  mappingButtonPosition,
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
import { RollbackOutlined } from "@ant-design/icons";
import { useMessageContext } from "../../hooks";

const PRESET_STYLE = mappingButtonPresetStyle(52);

type Position = { x: number; y: number };

export default function ButtonSwipe({
  index,
  config,
  originalSize,
  onConfigChange,
  onConfigDelete,
}: {
  index: number;
  config: SwipeConfig;
  originalSize: { width: number; height: number };
  onConfigChange: (config: SwipeConfig) => void;
  onConfigDelete: () => void;
}) {
  const id = `mapping-single-tap-${index}`;
  const bindText = config.bind.join("+");
  const maskArea = useAppSelector((state) => state.other.maskArea);
  const [showSetting, setShowSetting] = useState(false);
  const [isEditingPos, setIsEditingPos] = useState(false);

  useEffect(() => {
    const element = document.getElementById(id);
    if (element) {
      const position = config.positions[0];
      element.style.transform = mappingButtonTransformStyle(
        position.x,
        position.y,
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
      const newConfig = {
        ...config,
      };
      newConfig.positions[0] = {
        x,
        y,
      };
      onConfigChange(newConfig);
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
          originalSize={originalSize}
          isEditing={isEditingPos}
          onIsEditingChange={(v) => setIsEditingPos(v)}
        />
      </SettingModal>
      <Flex
        id={id}
        style={PRESET_STYLE}
        className="rounded-full absolute box-border border-solid border-2 border-text-secondary hover:border-text color-text"
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
      {showSetting && !isEditingPos && (
        <Background positions={config.positions} originalSize={originalSize} />
      )}
    </>
  );
}

function Background({
  positions,
  originalSize,
}: {
  positions: Position[];
  originalSize: { width: number; height: number };
}) {
  const maskArea = useAppSelector((state) => state.other.maskArea);

  return (
    <div
      className="fixed bg-transparent"
      style={{
        left: maskArea.left,
        top: maskArea.top,
        width: maskArea.width,
        height: maskArea.height,
      }}
    >
      <svg className="w-full h-full absolute color-primary">
        <defs>
          <marker
            id="arrow"
            markerWidth="8"
            markerHeight="7"
            refX="8"
            refY="3.5"
            orient="auto"
            markerUnits="strokeWidth"
          >
            <path d="M0,0 L8,3.5 L0,7 Z" fill="currentColor" />
          </marker>
        </defs>
        {positions.map((pos, index) => {
          if (index === positions.length - 1) return null;
          const { x: x1, y: y1 } = mappingButtonPosition(
            pos.x,
            pos.y,
            originalSize.width,
            originalSize.height,
            maskArea.width,
            maskArea.height
          );
          const { x: x2, y: y2 } = mappingButtonPosition(
            positions[index + 1].x,
            positions[index + 1].y,
            originalSize.width,
            originalSize.height,
            maskArea.width,
            maskArea.height
          );

          return (
            <line
              key={index}
              x1={x1}
              y1={y1}
              x2={x2}
              y2={y2}
              stroke="currentColor"
              strokeWidth="2"
              markerEnd="url(#arrow)"
            />
          );
        })}
      </svg>
      {positions.map((position, index) => {
        return (
          <div
            key={index}
            className="rounded-full w-3 h-3 bg-primary absolute left--1.5 top--1.5 text-center text-bold"
            style={{
              transform: mappingButtonTransformStyle(
                position.x,
                position.y,
                originalSize.width,
                originalSize.height,
                maskArea.width,
                maskArea.height
              ),
            }}
          >
            <span className="relative bottom-5">{index + 1}</span>
          </div>
        );
      })}
    </div>
  );
}

type PositonEditorItemProps = {
  maskArea: { width: number; height: number; left: number; top: number };
  originalSize: { width: number; height: number };
  position: Position;
  index: number;
  onItemChange: (index: number, position: Position) => void;
  onItemDelete: (index: number) => void;
};

function PositonEditorItem({
  maskArea,
  originalSize,
  position,
  index,
  onItemChange,
  onItemDelete,
}: PositonEditorItemProps) {
  const { t } = useTranslation();

  const [open, setOpen] = useState(false);

  const handleDrag = mappingButtonDragFactory(
    maskArea,
    originalSize,
    (pos) => onItemChange(index, pos),
    100
  );

  return (
    <Popover
      destroyOnHidden
      trigger="contextMenu"
      open={open}
      onOpenChange={(open) => setOpen(open)}
      content={
        <ItemBoxContainer gap={12}>
          <ItemBox>
            <Button
              block
              type="primary"
              onClick={() => {
                setOpen(false);
                onItemDelete(index);
              }}
            >
              {t("mappings.swipe.setting.delete")}
            </Button>
          </ItemBox>
        </ItemBoxContainer>
      }
    >
      <div
        className="rounded-full w-3 h-3 bg-primary absolute left--1.5 top--1.5 text-center text-bold hover:bg-primary-hover active:bg-primary-active"
        style={{
          transform: mappingButtonTransformStyle(
            position.x,
            position.y,
            originalSize.width,
            originalSize.height,
            maskArea.width,
            maskArea.height
          ),
        }}
        onMouseDown={handleDrag}
      >
        <span className="relative bottom-5 whitespace-nowrap">{index + 1}</span>
      </div>
    </Popover>
  );
}

function PositonEditor({
  positions,
  originalSize,
  onExit,
  onChange,
}: {
  positions: Position[];
  originalSize: { width: number; height: number };
  onExit: () => void;
  onChange: (positions: Position[]) => void;
}) {
  const maskArea = useAppSelector((state) => state.other.maskArea);
  const messageApi = useMessageContext();
  const { t } = useTranslation();

  function handleItemDelete(index: number) {
    if (positions.length === 1) {
      messageApi?.warning(t("mappings.swipe.setting.keepLastOne"));
      return;
    }
    onChange(positions.filter((_, i) => i !== index));
  }

  function handleItemChange(index: number, position: Position) {
    onChange([
      ...positions.slice(0, index),
      position,
      ...positions.slice(index + 1),
    ]);
  }

  function handleEditorClick(e: React.MouseEvent) {
    if (e.target === e.currentTarget && e.button === 2) {
      onChange([
        ...positions,
        clientPositionToMappingPosition(
          e.clientX,
          e.clientY,
          maskArea.left,
          maskArea.top,
          maskArea.width,
          maskArea.height,
          originalSize.width,
          originalSize.height
        ),
      ]);
    }
  }

  return (
    <div
      className="select-none fixed bg-[var(--ant-color-bg-mask)] z-2000 border border-solid border-primary"
      style={{
        left: maskArea.left - 1,
        top: maskArea.top - 1,
        width: maskArea.width,
        height: maskArea.height,
      }}
    >
      <Button
        shape="circle"
        size="small"
        type="primary"
        icon={<RollbackOutlined />}
        className="absolute top-8 right-8 z-1"
        onClick={() => onExit()}
      />
      <svg
        className="w-full h-full absolute color-primary"
        onMouseDown={handleEditorClick}
        onContextMenu={(e) => e.preventDefault()}
      >
        <defs>
          <marker
            id="arrow"
            markerWidth="8"
            markerHeight="7"
            refX="8"
            refY="3.5"
            orient="auto"
            markerUnits="strokeWidth"
          >
            <path d="M0,0 L8,3.5 L0,7 Z" fill="currentColor" />
          </marker>
        </defs>
        {positions.map((pos, index) => {
          if (index === positions.length - 1) return null;
          const { x: x1, y: y1 } = mappingButtonPosition(
            pos.x,
            pos.y,
            originalSize.width,
            originalSize.height,
            maskArea.width,
            maskArea.height
          );
          const { x: x2, y: y2 } = mappingButtonPosition(
            positions[index + 1].x,
            positions[index + 1].y,
            originalSize.width,
            originalSize.height,
            maskArea.width,
            maskArea.height
          );

          return (
            <line
              key={index}
              x1={x1}
              y1={y1}
              x2={x2}
              y2={y2}
              stroke="currentColor"
              strokeWidth="2"
              markerEnd="url(#arrow)"
            />
          );
        })}
      </svg>
      {positions.map((position, index) => (
        <PositonEditorItem
          key={index}
          position={position}
          index={index}
          onItemChange={handleItemChange}
          onItemDelete={handleItemDelete}
          maskArea={maskArea}
          originalSize={originalSize}
        />
      ))}
    </div>
  );
}

function Setting({
  config,
  onConfigChange,
  onConfigDelete: onDelete,
  originalSize,
  isEditing,
  onIsEditingChange,
}: {
  config: SwipeConfig;
  onConfigChange: (config: SwipeConfig) => void;
  onConfigDelete: () => void;
  originalSize: { width: number; height: number };
  isEditing: boolean;
  onIsEditingChange: (v: boolean) => void;
}) {
  const { t } = useTranslation();
  const messageApi = useMessageContext();

  return (
    <div>
      <h1 className="title-with-line">{t("mappings.swipe.setting.title")}</h1>
      {isEditing && (
        <PositonEditor
          positions={config.positions}
          originalSize={originalSize}
          onExit={() => onIsEditingChange(false)}
          onChange={(positions) => onConfigChange({ ...config, positions })}
        />
      )}
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
        <SettingNote
          note={config.note}
          onNoteChange={(note) => onConfigChange({ ...config, note })}
        />
        <ItemBox label={t("mappings.swipe.setting.positions")}>
          <Button
            type="primary"
            size="small"
            onClick={() => {
              messageApi?.info(t("mappings.swipe.setting.positonsHelp"));
              onIsEditingChange(true);
            }}
          >
            {t("mappings.swipe.setting.edit")}
          </Button>
        </ItemBox>
        <SettingDelete onDelete={onDelete} />
      </ItemBoxContainer>
    </div>
  );
}
