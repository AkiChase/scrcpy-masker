import React from "react";
import { Spin } from "antd";
import { LoadingOutlined } from "@ant-design/icons";

export default function LoadingWrapper({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <React.Suspense
      fallback={
        <Spin indicator={<LoadingOutlined spin />} size="large" fullscreen />
      }
    >
      {children}
    </React.Suspense>
  );
}
