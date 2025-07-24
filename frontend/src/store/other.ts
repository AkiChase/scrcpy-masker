import { createSlice, type PayloadAction } from "@reduxjs/toolkit";

export interface OtherState {
  maskArea: {
    width: number;
    height: number;
    left: number;
    top: number;
  };
  backgroundImage: string;
}

const initialState: OtherState = {
  maskArea: {
    width: 1,
    height: 1,
    left: 0,
    top: 0,
  },
  backgroundImage: "",
};

const otherSlice = createSlice({
  name: "other",
  initialState,
  reducers: {
    setMaskArea: (state, action: PayloadAction<OtherState["maskArea"]>) => {
      state.maskArea = action.payload;
    },
    setBackgroundImage: (
      state,
      action: PayloadAction<OtherState["backgroundImage"]>
    ) => {
      state.backgroundImage = action.payload;
    },
  },
});

export const { setMaskArea, setBackgroundImage } = otherSlice.actions;

export default otherSlice.reducer;
