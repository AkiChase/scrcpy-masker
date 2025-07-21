import { createSlice, type PayloadAction } from "@reduxjs/toolkit";

export interface OtherState {
  maskArea: {
    width: number;
    height: number;
    left: number;
    top: number;
  };
}

const initialState: OtherState = {
  maskArea: {
    width: 1,
    height: 1,
    left: 0,
    top: 0,
  },
};

const otherSlice = createSlice({
  name: "other",
  initialState,
  reducers: {
    setMaskArea: (state, action: PayloadAction<OtherState["maskArea"]>) => {
      state.maskArea = action.payload;
    },
  },
});

export const { setMaskArea } = otherSlice.actions;

export default otherSlice.reducer;
