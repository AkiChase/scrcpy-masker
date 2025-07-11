import "./App.scss";
import { Layout, message } from "antd";
import { MessageContext } from "./hooks";
import { useAppDispatch } from "./store/store";
import { forceSetLocalConfig } from "./store/localConfigReducer";
import { useEffect } from "react";
import { Content } from "antd/es/layout/layout";
import Sider from "./components/Sider";
import { Outlet } from "react-router-dom";

function App() {
  const dispatch = useAppDispatch();
  const [messageApi, contextHolder] = message.useMessage();
  async function loadLocalConfig() {
    try {
      // TODO 临时数据
      // const res = await requestGet("/api/config/get_config");
      const res: any = {
        data: {
          webPort: 1234,
          controllerPort: 1234,
          adbPath: "adb",
          verticalScreenHeight: 720,
          horizontalScreenWidth: 1280,
          verticalPosition: [800, 300],
          horizontalPosition: [300, 300],
          activeMappingFile: "test.json",
          mappingLabelOpacity: 0.3,
        },
      };
      dispatch(forceSetLocalConfig(res.data));
    } catch (err: any) {
      messageApi.error(err);
    }
  }

  useEffect(() => {
    loadLocalConfig();
  }, []);

  return (
    <MessageContext.Provider value={messageApi}>
      {contextHolder}
      <Layout className="min-h-100vh">
        <Sider />
        <Layout>
          <Content>
            <div className="page-container-parent scrollbar">
              <Outlet />
            </div>
          </Content>
        </Layout>
      </Layout>
    </MessageContext.Provider>
  );
}

export default App;
