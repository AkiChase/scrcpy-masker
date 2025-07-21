import {
  Badge,
  Button,
  Descriptions,
  Input,
  Popover,
  Space,
  Table,
  type TableProps,
} from "antd";
import { useTranslation } from "react-i18next";
import type { AdbDevice, ControlledDevice, requestGet } from "../utils";
import {
  CloseCircleOutlined,
  InfoCircleOutlined,
  LinkOutlined,
} from "@ant-design/icons";
import IconButton from "./common/IconButton";
import { useMemo } from "react";
import { ItemBox, ItemBoxContainer } from "./common/ItemBox";

const res: Awaited<
  ReturnType<
    typeof requestGet<{
      adb_devices: AdbDevice[];
      controlled_devices: ControlledDevice[];
    }>
  >
> = {
  message: "Successfully obtained device list",
  data: {
    adb_devices: [
      {
        id: "127.0.0.1:26624",
        status: "device",
      },
      {
        id: "emulator-5554",
        status: "device",
      },
    ],
    controlled_devices: [
      {
        device_id: "127.0.0.1:26624",
        device_size: [2560, 1440],
        main: true,
        name: "2308CPXD0C",
        scid: "10464217",
        socket_ids: ["main_control", "sub_control_10464217", "test3"],
      },
    ],
  },
};

function ControlledDevices() {
  const { t } = useTranslation();

  const columns: TableProps<ControlledDevice>["columns"] = [
    {
      title: "ID",
      dataIndex: "device_id",
      key: "device_id",
      render: (_, record) => (
        <Space size="large">
          {record.device_id}
          {record.main && (
            <Badge
              color="green"
              text={t("devices.controlledDevices.mainDevice")}
            />
          )}
        </Space>
      ),
    },
    {
      title: t("devices.controlledDevices.name"),
      dataIndex: "name",
      key: "name",
    },
    {
      title: t("devices.controlledDevices.size"),
      dataIndex: "device_size",
      key: "device_size",
      render: (device_size) => {
        return `${device_size[0]}x${device_size[1]}`;
      },
    },
    {
      title: t("devices.controlledDevices.action"),
      key: "action",
      align: "center",
      render: (_, record) => (
        <Space size="middle" className="text-4">
          <Popover
            trigger="click"
            content={
              <Descriptions
                className="w-15rem"
                column={1}
                items={[
                  {
                    key: "scid",
                    label: "SCID",
                    children: record.scid,
                  },
                  {
                    key: "sockets",
                    label: "Sockets",
                    children: (
                      <Space direction="vertical" size={2}>
                        {record.socket_ids.map((socket_id) => (
                          <span key={socket_id}>{socket_id}</span>
                        ))}
                      </Space>
                    ),
                  },
                ]}
              />
            }
          >
            <IconButton
              tooltip={t("devices.controlledDevices.actionInfo")}
              size={18}
              color="info"
              icon={<InfoCircleOutlined />}
            />
          </Popover>
          <IconButton
            tooltip={t("devices.controlledDevices.actionClose")}
            size={18}
            color="error"
            icon={<CloseCircleOutlined />}
          />
        </Space>
      ),
    },
  ];

  return (
    <Table<ControlledDevice>
      rowKey={(record) => record.device_id}
      pagination={{ pageSize: 5 }}
      columns={columns}
      dataSource={res.data.controlled_devices}
    />
  );
}

function OtherDevices() {
  const { t } = useTranslation();
  const otherDevices = useMemo(
    () =>
      res.data.adb_devices.filter(
        (device: AdbDevice) =>
          res.data.controlled_devices.findIndex(
            (controlledDevice: ControlledDevice) =>
              controlledDevice.device_id === device.id
          ) === -1
      ),
    [res.data.adb_devices]
  );

  const columns: TableProps<AdbDevice>["columns"] = [
    {
      title: "ID",
      dataIndex: "id",
      key: "id",
    },
    {
      title: t("devices.otherDevices.status"),
      dataIndex: "status",
      key: "status",
    },
    {
      title: t("devices.otherDevices.action"),
      key: "action",
      align: "center",
      render: (_, record) => (
        <Space size="middle" className="text-4">
          <IconButton
            color="primary"
            tooltip={t("devices.otherDevices.actionControl")}
            size={18}
            icon={<LinkOutlined />}
            onClick={() => console.log(record)}
          />
        </Space>
      ),
    },
  ];

  return (
    <Table<AdbDevice>
      rowKey={(record) => record.id}
      pagination={{ pageSize: 5 }}
      columns={columns}
      dataSource={otherDevices}
    />
  );
}

export default function Devices() {
  const { t } = useTranslation();

  return (
    <div className="page-container">
      <section>
        <h2 className="title-with-line">{t("devices.adbTools.title")}</h2>
        <ItemBoxContainer className="mb-6">
          <ItemBox label={t("devices.adbTools.pair.label")}>
            <Space.Compact>
              <Input placeholder="ip:port" />
              <Input placeholder="code" />
              <Button type="primary">{t("devices.adbTools.pair.btn")}</Button>
            </Space.Compact>
          </ItemBox>
          <ItemBox label={t("devices.adbTools.connect.label")}>
            <Space.Compact>
              <Input placeholder="ip:port" />
              <Button type="primary">
                {t("devices.adbTools.connect.btn")}
              </Button>
            </Space.Compact>
          </ItemBox>
        </ItemBoxContainer>
      </section>
      <section>
        <h2 className="title-with-line">
          {t("devices.controlledDevices.title")}
        </h2>
        <ControlledDevices />
      </section>
      <section>
        <h2 className="title-with-line">{t("devices.otherDevices.title")}</h2>
        <OtherDevices />
      </section>
    </div>
  );
}
