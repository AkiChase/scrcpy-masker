import type { MessageInstance } from "antd/es/message/interface";
import { createContext, useContext } from "react";
import { useAppDispatch } from "./store/store";
import { setBackgroundImage } from "./store/other";
import { useTranslation } from "react-i18next";

export const MessageContext = createContext<MessageInstance | null>(null);
export const useMessageContext = () => useContext(MessageContext);

export function useRefreshBackgroundImage() {
  const dispatch = useAppDispatch();
  const messageApi = useMessageContext();
  const { t } = useTranslation();

  return async () => {
    try {
      const res = await fetch("/test.png");
      const blob = await res.blob();
      const url = URL.createObjectURL(blob);
      dispatch(setBackgroundImage(url));
    } catch (error) {
      messageApi?.error(t("mappings.common.refreshBgFail", error as string));
    }
  };
}
