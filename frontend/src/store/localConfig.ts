import { createSlice, type PayloadAction } from "@reduxjs/toolkit";
import { requestPost } from "../utils";

async function _updateLocalConfig(key: string, value: any) {
  try {
    await requestPost("/api/config/update_config", {
      key,
      value,
    });
  } catch (error: any) {
    console.error(error);
  }
}

const debounceMap = new Map<string, ReturnType<typeof setTimeout>>();

function updateLocalConfig(key: string, value: any) {
  if (debounceMap.has(key)) {
    clearTimeout(debounceMap.get(key)!);
  }

  const timeout = setTimeout(() => {
    _updateLocalConfig(key, value);
    debounceMap.delete(key);
  }, 500);

  debounceMap.set(key, timeout);
}

export interface LocalConfigState {
  // port
  webPort: number;
  controllerPort: number;
  // adb
  adbPath: string;
  // mask area
  verticalScreenHeight: number;
  horizontalScreenWidth: number;
  verticalPosition: [number, number];
  horizontalPosition: [number, number];
  // mapping
  activeMappingFile: string;
  mappingLabelOpacity: number;
}

const initialState: LocalConfigState = {
  webPort: 0,
  controllerPort: 0,
  adbPath: "",
  verticalScreenHeight: 0,
  horizontalScreenWidth: 0,
  verticalPosition: [0, 0],
  horizontalPosition: [0, 0],
  activeMappingFile: "",
  mappingLabelOpacity: 0,
};

const localConfigSlice = createSlice({
  name: "localConfig",
  initialState,
  reducers: {
    forceSetLocalConfig: (state, action: PayloadAction<LocalConfigState>) => {
      for (const [key, value] of Object.entries(action.payload)) {
        if (key in state) {
          (state as any)[key] = value;
        }
      }
    },
    setWebPort: (state, action: PayloadAction<number>) => {
      state.webPort = action.payload;
      updateLocalConfig("web_port", action.payload);
    },
    setControllerPort: (state, action: PayloadAction<number>) => {
      state.controllerPort = action.payload;
      updateLocalConfig("controller_port", action.payload);
    },
    setAdbPath: (state, action: PayloadAction<string>) => {
      state.adbPath = action.payload;
      updateLocalConfig("adb_path", action.payload);
    },
    setVerticalScreenHeight: (state, action: PayloadAction<number>) => {
      state.verticalScreenHeight = action.payload;
      updateLocalConfig("vertical_screen_height", action.payload);
    },
    setHorizontalScreenWidth: (state, action: PayloadAction<number>) => {
      state.horizontalScreenWidth = action.payload;
      updateLocalConfig("horizontal_screen_width", action.payload);
    },
    setVerticalPosition: (state, action: PayloadAction<[number, number]>) => {
      state.verticalPosition = action.payload;
      updateLocalConfig("vertical_position", action.payload);
    },
    setHorizontalPosition: (state, action: PayloadAction<[number, number]>) => {
      state.horizontalPosition = action.payload;
      updateLocalConfig("horizontal_position", action.payload);
    },
    setActiveMappingFile: (state, action: PayloadAction<string>) => {
      state.activeMappingFile = action.payload;
      // already updated by change_active_mapping
    },
    setMappingLabelOpacity: (state, action: PayloadAction<number>) => {
      state.mappingLabelOpacity = action.payload;
      updateLocalConfig("mapping_label_opacity", action.payload);
    },
  },
});

export const {
  forceSetLocalConfig,
  setWebPort,
  setControllerPort,
  setAdbPath,
  setVerticalScreenHeight,
  setHorizontalScreenWidth,
  setVerticalPosition,
  setHorizontalPosition,
  setActiveMappingFile,
} = localConfigSlice.actions;

export default localConfigSlice.reducer;
