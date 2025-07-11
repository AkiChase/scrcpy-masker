import { createBrowserRouter } from "react-router-dom";
import { lazy } from "react";
import LoadingWrapper from "./components/LoadingWrapper";
import App from "./App";

const Welcome = lazy(() => import("./components/Welcome"));
const Devices = lazy(() => import("./components/Devices"));
const Mappings = lazy(() => import("./components/mappings/Mappings"));
const Settings = lazy(() => import("./components/settings/Settings"));

const router = createBrowserRouter([
  {
    path: "/",
    element: <App />,
    children: [
      {
        index: true,
        element: (
          <LoadingWrapper>
            <Welcome />
          </LoadingWrapper>
        ),
      },
      {
        path: "devices",
        element: (
          <LoadingWrapper>
            <Devices />
          </LoadingWrapper>
        ),
      },
      {
        path: "mappings",
        element: (
          <LoadingWrapper>
            <Mappings />
          </LoadingWrapper>
        ),
      },
      {
        path: "settings",
        element: (
          <LoadingWrapper>
            <Settings />
          </LoadingWrapper>
        ),
      },
    ],
  },
]);

export default router;
