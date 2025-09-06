import "./App.scss";
import { Layout, message, Spin } from "antd";
import { MessageContext } from "./hooks";
import { staticStore, useAppDispatch, useAppSelector } from "./store/store";
import { forceSetLocalConfig } from "./store/localConfig";
import { useEffect } from "react";
import { Content } from "antd/es/layout/layout";
import Sider from "./components/Sider";
import { useLocation, useOutlet } from "react-router-dom";
import KeepAlive, { useKeepAliveRef } from "keepalive-for-react";
import LoadingWrapper from "./components/common/LoadingWrapper";
import { requestGet } from "./utils";
import { setIsLoading } from "./store/other";
import i18n from "./i18n";

function App() {
  const dispatch = useAppDispatch();
  const [messageApi, contextHolder] = message.useMessage();
  const isLoading = useAppSelector((state) => state.other.isLoading);
  const location = useLocation();
  const aliveRef = useKeepAliveRef();

  const outlet = useOutlet();

  async function loadLocalConfig() {
    dispatch(setIsLoading(true));
    try {
      const res = await requestGet("/api/config/get_config");
      dispatch(forceSetLocalConfig(res.data));
      i18n.changeLanguage(res.data.language);
    } catch (err: any) {
      messageApi.error(err);
    }
    dispatch(setIsLoading(false));
  }

  useEffect(() => {
    staticStore.messageApi = messageApi;
    loadLocalConfig();

    // prevent backward
    history.pushState(null, "", window.location.href);
    const handlePopState = () => {
      history.pushState(null, "", window.location.href);
    };
    window.addEventListener("popstate", handlePopState);

    return () => {
      window.removeEventListener("popstate", handlePopState);
    };
  }, []);

  return (
    <MessageContext.Provider value={messageApi}>
      {contextHolder}
      <Spin spinning={isLoading} fullscreen delay={200} />
      <Layout className="min-h-100vh">
        <Sider />
        <Layout>
          <Content>
            <KeepAlive
              transition
              aliveRef={aliveRef}
              activeCacheKey={location.pathname}
            >
              <LoadingWrapper>
                <div className="page-container-parent scrollbar">{outlet}</div>
              </LoadingWrapper>
            </KeepAlive>
          </Content>
        </Layout>
      </Layout>
    </MessageContext.Provider>
  );
}

export default App;
