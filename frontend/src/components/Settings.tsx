import { useTranslation } from "react-i18next";
import { useAppDispatch, useAppSelector } from "../store/store";
import { ItemBox, ItemBoxContainer } from "./common/ItemBox";
import {
  Button,
  Flex,
  Input,
  InputNumber,
  Select,
  Slider,
  Space,
  Switch,
} from "antd";
import {
  forceSetLocalConfig,
  setAdbPath,
  setControllerPort,
  setVerticalPosition,
  setverticalMaskHeight,
  setWebPort,
  sethorizontalMaskWidth,
  setHorizontalPosition,
  setMappingLabelOpacity,
  setClipboardSync,
  setLanguage,
  setVideoCodec,
  setAudioCodec,
  setAudioBitRate,
  setVideoBitRate,
  setVideoMaxSize,
  setVideoMaxFps,
} from "../store/localConfig";
import { setIsLoading } from "../store/other";
import { requestGet } from "../utils";
import i18n from "../i18n";
import { useMessageContext } from "../hooks";
import { SyncOutlined } from "@ant-design/icons";

const languageOptions = [
  {
    label: "简体中文",
    value: "zh-CN",
  },
  {
    label: "English",
    value: "en-US",
  },
];

const videoCodecOptions = ["H264", "H265", "AV1"].map((v) => ({
  value: v,
  label: v,
}));

const audioCodecOptions = ["AAC", "OPUS", "FLAC", "RAW"].map((v) => ({
  value: v,
  label: v,
}));

export default function Settings() {
  const { t } = useTranslation();
  const dispatch = useAppDispatch();
  const messageApi = useMessageContext();
  const localConfig = useAppSelector((state) => state.localConfig);

  async function loadLocalConfig() {
    dispatch(setIsLoading(true));
    try {
      const res = await requestGet("/api/config/get_config");
      dispatch(forceSetLocalConfig(res.data));
      i18n.changeLanguage(res.data.language);
    } catch (err: any) {
      messageApi?.error(err);
    }
    dispatch(setIsLoading(false));
  }

  return (
    <div className="page-container">
      <section>
        <Flex align="start" justify="space-between">
          <h2 className="title-with-line" style={{ marginBottom: 0 }}>
            {t("settings.title.header")}
          </h2>
          <Button
            type="primary"
            icon={<SyncOutlined />}
            shape="circle"
            onClick={loadLocalConfig}
          ></Button>
        </Flex>
        <h3 className="title-with-line-sub">{t("settings.title.basic")}</h3>
        <ItemBoxContainer className="mb-6">
          <ItemBox label={t("settings.language")}>
            <Select
              className="w-sm"
              value={localConfig.language}
              options={languageOptions}
              onChange={(v) => dispatch(setLanguage(v))}
            />
          </ItemBox>
          <ItemBox label={t("settings.adbPath")}>
            <Input
              className="w-sm"
              value={localConfig.adbPath}
              onChange={(e) => dispatch(setAdbPath(e.target.value))}
            />
          </ItemBox>
          <ItemBox label={t("settings.clipboardSync")}>
            <Switch
              checked={localConfig.clipboardSync}
              onChange={(v) => dispatch(setClipboardSync(v))}
            />
          </ItemBox>
        </ItemBoxContainer>
        <h3 className="title-with-line-sub">{t("settings.title.mask")}</h3>
        <ItemBoxContainer className="mb-6">
          <ItemBox label={t("settings.mappingLabelOpacity")}>
            <Slider
              className="w-sm"
              min={0}
              max={1}
              step={0.01}
              onChange={(v) => dispatch(setMappingLabelOpacity(v))}
              value={localConfig.mappingLabelOpacity}
            />
          </ItemBox>
          <ItemBox label={t("settings.horizontalMaskWidth")}>
            <InputNumber
              className="w-sm"
              controls={false}
              min={50}
              value={localConfig.horizontalMaskWidth}
              onChange={(v) =>
                v !== null && dispatch(sethorizontalMaskWidth(v))
              }
            />
          </ItemBox>
          <ItemBox label={t("settings.horizontalMaskPosition")}>
            <Space.Compact className="w-sm">
              <InputNumber
                prefix="X:"
                className="w-50%"
                controls={false}
                value={localConfig.horizontalPosition[0]}
                onChange={(v) =>
                  v !== null &&
                  dispatch(
                    setHorizontalPosition([
                      v,
                      localConfig.horizontalPosition[1],
                    ])
                  )
                }
              />
              <InputNumber
                prefix="Y:"
                className="w-50%"
                controls={false}
                value={localConfig.horizontalPosition[1]}
                onChange={(v) =>
                  v !== null &&
                  dispatch(
                    setHorizontalPosition([
                      localConfig.horizontalPosition[0],
                      v,
                    ])
                  )
                }
              />
            </Space.Compact>
          </ItemBox>
          <ItemBox label={t("settings.verticalMaskHeight")}>
            <InputNumber
              className="w-sm"
              controls={false}
              min={50}
              value={localConfig.verticalMaskHeight}
              onChange={(v) => v !== null && dispatch(setverticalMaskHeight(v))}
            />
          </ItemBox>
          <ItemBox label={t("settings.verticalMaskPosition")}>
            <Space.Compact className="w-sm">
              <InputNumber
                prefix="X:"
                className="w-50%"
                controls={false}
                value={localConfig.verticalPosition[0]}
                onChange={(v) =>
                  v !== null &&
                  dispatch(
                    setVerticalPosition([v, localConfig.verticalPosition[1]])
                  )
                }
              />
              <InputNumber
                prefix="Y:"
                className="w-50%"
                controls={false}
                value={localConfig.verticalPosition[1]}
                onChange={(v) =>
                  v !== null &&
                  dispatch(
                    setVerticalPosition([localConfig.verticalPosition[0], v])
                  )
                }
              />
            </Space.Compact>
          </ItemBox>
        </ItemBoxContainer>
        <h3 className="title-with-line-sub">{t("settings.title.video")}</h3>
        <ItemBoxContainer className="mb-6">
          <ItemBox label={t("settings.videoCodec")}>
            <Select
              className="w-sm"
              value={localConfig.videoCodec}
              options={videoCodecOptions}
              onChange={(v) => dispatch(setVideoCodec(v))}
            />
          </ItemBox>
          <ItemBox label={t("settings.videoBitRate")}>
            <InputNumber
              className="w-sm"
              controls={false}
              min={1000000}
              suffix="bps"
              value={localConfig.videoBitRate}
              onChange={(v) => v !== null && dispatch(setVideoBitRate(v))}
            />
          </ItemBox>
          <ItemBox
            label={t("settings.videoMaxSize")}
            tooltip={t("settings.zeroUnlimitedTip")}
          >
            <InputNumber
              className="w-sm"
              controls={false}
              min={0}
              value={localConfig.videoMaxSize}
              onChange={(v) => v !== null && dispatch(setVideoMaxSize(v))}
            />
          </ItemBox>
          <ItemBox
            label={t("settings.videoMaxFps")}
            tooltip={t("settings.zeroUnlimitedTip")}
          >
            <InputNumber
              className="w-sm"
              controls={false}
              min={0}
              value={localConfig.videoMaxFps}
              onChange={(v) => v !== null && dispatch(setVideoMaxFps(v))}
            />
          </ItemBox>
        </ItemBoxContainer>
        <h3 className="title-with-line-sub">{t("settings.title.audio")}</h3>
        <ItemBoxContainer className="mb-6">
          <ItemBox label={t("settings.audioCodec")}>
            <Select
              className="w-sm"
              value={localConfig.audioCodec}
              options={audioCodecOptions}
              onChange={(v) => dispatch(setAudioCodec(v))}
            />
          </ItemBox>
          <ItemBox label={t("settings.audioBitRate")}>
            <InputNumber
              className="w-sm"
              controls={false}
              min={1000}
              suffix="bps"
              value={localConfig.audioBitRate}
              onChange={(v) => v !== null && dispatch(setAudioBitRate(v))}
            />
          </ItemBox>
        </ItemBoxContainer>
        <h3 className="title-with-line-sub">{t("settings.title.advance")}</h3>
        <ItemBoxContainer className="mb-6">
          <ItemBox label={t("settings.webPort")}>
            <InputNumber
              className="w-sm"
              controls={false}
              value={localConfig.webPort}
              onChange={(v) => v !== null && dispatch(setWebPort(v))}
            />
          </ItemBox>
          <ItemBox label={t("settings.controllerPort")}>
            <InputNumber
              className="w-sm"
              controls={false}
              value={localConfig.controllerPort}
              onChange={(v) => v !== null && dispatch(setControllerPort(v))}
            />
          </ItemBox>
        </ItemBoxContainer>
      </section>
      <section>
        <h2 className="title-with-line">About</h2>
        Author, Current version, CheckForUpdate
      </section>
    </div>
  );
}
