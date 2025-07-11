import { defineConfig, presetWind3 } from "unocss";

export default defineConfig({
  presets: [presetWind3()],
  theme: {
    colors: {
      text: {
        DEFAULT: "var(--ant-color-text)",
        secondary: "var(--ant-color-text-secondary)",
        tertiary: "var(--ant-color-text-tertiary)",
        quaternary: "var(--ant-color-text-quaternary)",
      },
      primary: {
        DEFAULT: "var(--ant-color-primary)",
        hover: "var(--ant-color-primary-hover)",
        active: "var(--ant-color-primary-active)",

        bg: "var(--ant-color-primary-bg)",
        "bg-hover": "var(--ant-color-primary-bg-hover)",

        border: "var(--ant-color-primary-border)",
        "border-hover": "var(--ant-color-primary-border-hover)",

        text: "var(--ant-color-primary-text)",
        "text-hover": "var(--ant-color-primary-text-hover)",
        "text-active": "var(--ant-color-primary-text-active)",
      },
      success: {
        DEFAULT: "var(--ant-color-success)",
        hover: "var(--ant-color-success-hover)",
        active: "var(--ant-color-success-active)",

        bg: "var(--ant-color-success-bg)",
        "bg-hover": "var(--ant-color-success-bg-hover)",

        border: "var(--ant-color-success-border)",
        "border-hover": "var(--ant-color-success-border-hover)",

        text: "var(--ant-color-success-text)",
        "text-hover": "var(--ant-color-success-text-hover)",
        "text-active": "var(--ant-color-success-text-active)",
      },
    },
  },
});
