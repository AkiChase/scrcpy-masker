export interface MappingConfig {
  title: string;
  version: string;
  original_size: {
    width: number;
    height: number;
  };
  mappings: MappingType[];
}

export type MappingType =
  | SingleTapConfig
  | RepeatTapConfig
  | MultipleTapConfig
  | SwipeConfig
  | DirectionPadConfig
  | MouseCastSpellConfig
  | PadCastSpellConfig
  | CancelCastConfig
  | ObservationConfig
  | FpsConfig
  | FireConfig
  | RawInputConfig;

export type Position = {
  x: number;
  y: number;
};

export type ButtonBinding = string[];

export interface SingleTapConfig {
  bind: ButtonBinding;
  duration: number;
  note: string;
  pointer_id: number;
  position: Position;
  sync: boolean;
  type: "SingleTap";
}

export interface RepeatTapConfig {
  bind: ButtonBinding;
  duration: number;
  interval: number;
  note: string;
  pointer_id: number;
  position: Position;
  type: "RepeatTap";
}

export interface MultipleTapItem {
  duration: number;
  position: Position;
  wait: number;
}

export interface MultipleTapConfig {
  bind: ButtonBinding;
  items: MultipleTapItem[];
  note: string;
  pointer_id: number;
  type: "MultipleTap";
}

export interface SwipeConfig {
  bind: ButtonBinding;
  interval: number;
  note: string;
  pointer_id: number;
  positions: Position[];
  type: "Swipe";
}

export interface DirectionButtonBinding {
  type: "Button";
  up: ButtonBinding;
  down: ButtonBinding;
  left: ButtonBinding;
  right: ButtonBinding;
}

export interface DirectionJoyStickBinding {
  type: "JoyStick";
  x: string;
  y: string;
}

export type DirectionBinding =
  | DirectionButtonBinding
  | DirectionJoyStickBinding;

export interface DirectionPadConfig {
  bind: DirectionBinding;
  initial_duration: number;
  max_offset_x: number;
  max_offset_y: number;
  note: string;
  pointer_id: number;
  position: Position;
  type: "DirectionPad";
}

export type MouseCastReleaseMode = "OnPress" | "OnRelease" | "OnSecondPress";

export interface MouseCastSpellConfig {
  bind: ButtonBinding;
  cast_no_direction: boolean;
  cast_radius: number;
  center: Position;
  drag_radius: number;
  horizontal_scale_factor: number;
  vertical_scale_factor: number;
  note: string;
  pointer_id: number;
  position: Position;
  release_mode: MouseCastReleaseMode;
  type: "MouseCastSpell";
}

export type PadCastReleaseMode = "OnRelease" | "OnSecondPress";

export interface PadCastSpellConfig {
  bind: ButtonBinding;
  block_direction_pad: boolean;
  drag_radius: number;
  note: string;
  pad_action: string;
  pad_bind: DirectionButtonBinding;
  pointer_id: number;
  position: Position;
  release_mode: PadCastReleaseMode;
  type: "PadCastSpell";
}

export interface CancelCastConfig {
  bind: ButtonBinding;
  note: string;
  position: Position;
  type: "CancelCast";
}

export interface ObservationConfig {
  bind: ButtonBinding;
  note: string;
  pointer_id: number;
  position: Position;
  sensitivity_x: number;
  sensitivity_y: number;
  type: "Observation";
}

export interface FpsConfig {
  bind: ButtonBinding;
  note: string;
  pointer_id: number;
  position: Position;
  sensitivity_x: number;
  sensitivity_y: number;
  type: "Fps";
}

export interface FireConfig {
  bind: ButtonBinding;
  note: string;
  pointer_id: number;
  position: Position;
  sensitivity_x: number;
  sensitivity_y: number;
  type: "Fire";
}

export interface RawInputConfig {
  bind: ButtonBinding;
  note: string;
  position: Position;
  type: "RawInput";
}
