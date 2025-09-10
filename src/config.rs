use std::{
    fs::{File, create_dir_all},
    io::Write,
    sync::RwLock,
};

use crate::utils::relate_to_root_path;
use once_cell::sync::Lazy;
use paste::paste;
use rust_i18n::t;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;

static CONFIG: Lazy<RwLock<LocalConfig>> = Lazy::new(|| RwLock::default());

// TODO 单独写外部脚本来捕获特定窗口，发送Post消息来设置蒙版相关配置（宽度>=高度则设置横屏相关配置，否则设置竖屏）

// TODO java/com/genymobile/scrcpy/Options.java 附加一些Options选项，比如视频比特率等等
// video_codec (H265 可能提供更好的质量，但 H264 应该能提供更低的延迟)
// video_bit_rate（视频码率，在设备支持的情况下码率越高，视频表现越好）
// max_size (可选限制视频最大尺寸，默认不限制。进行限制有助于提高性能)
// max_fps (可选限制视频最大帧数，默认不限制。此外，只有当屏幕内容发生变化时才会生成新的帧)
// audio_codec (The possible values are opus (default), aac, flac and raw (uncompressed PCM 16-bit LE))
// audio_bit_rate（音频码率，在设备支持的情况下码率越高，音频表现越好）

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LocalConfig {
    // port
    pub web_port: u16,
    pub controller_port: u16,
    // adb
    pub adb_path: String,
    // mask area
    pub vertical_mask_height: u32,
    pub horizontal_mask_width: u32,
    pub vertical_position: (i32, i32),
    pub horizontal_position: (i32, i32),
    // mapping
    pub active_mapping_file: String,
    pub mapping_label_opacity: f32,
    // language
    pub language: String,
    // clipboard sync
    pub clipboard_sync: bool,
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            adb_path: "adb".to_string(),
            web_port: 27799,
            controller_port: 27798,
            vertical_mask_height: 720,
            horizontal_mask_width: 1280,
            vertical_position: (100, 100),
            horizontal_position: (100, 100),
            active_mapping_file: "default.json".to_string(),
            mapping_label_opacity: 0.3,
            language: "en-US".to_string(),
            clipboard_sync: true,
        }
    }
}

macro_rules! define_setter {
    ($(($field:ident, $typ:ty)),* $(,)?) => {
        paste! {
            $(
                pub fn [<set_ $field>] (value: $typ) {
                    CONFIG.write().unwrap().$field = value;
                    Self::save().unwrap();
                }
            )*
        }
    };
}

impl LocalConfig {
    pub fn save() -> Result<(), String> {
        let config_json = to_string_pretty(&Self::get())
            .map_err(|e| format!("{}: {}", t!("localConfig.serializeConfigError"), e))?;

        let path = relate_to_root_path(["local", "config.json"]);
        if let Some(parent) = path.parent() {
            create_dir_all(parent)
                .map_err(|e| format!("{}: {}", t!("localConfig.createConfigDirError"), e))?;
        }
        let mut file = File::create(path)
            .map_err(|e| format!("{}: {}", t!("localConfig.createConfigError"), e))?;
        file.write_all(config_json.as_bytes())
            .map_err(|e| format!("{}: {}", t!("localConfig.writeConfigError"), e))?;
        Ok(())
    }

    pub fn load() -> Result<(), String> {
        let path = relate_to_root_path(["local", "config.json"]);
        let config_string = std::fs::read_to_string(&path).map_err(|e| {
            format!(
                "{} {}: {}",
                t!("localConfig.readConfigError"),
                path.to_str().unwrap(),
                e
            )
        })?;
        let config: LocalConfig = serde_json::from_str(&config_string)
            .map_err(|e| format!("{}: {}", t!("localConfig.serializeConfigError"), e))?;
        *CONFIG.write().unwrap() = config;
        Ok(())
    }

    pub fn get() -> LocalConfig {
        CONFIG.read().unwrap().clone()
    }

    pub fn get_clipboard_sync() -> bool {
        CONFIG.read().unwrap().clipboard_sync
    }

    define_setter!(
        (web_port, u16),
        (controller_port, u16),
        (adb_path, String),
        (vertical_mask_height, u32),
        (horizontal_mask_width, u32),
        (vertical_position, (i32, i32)),
        (horizontal_position, (i32, i32)),
        (active_mapping_file, String),
        (mapping_label_opacity, f32),
        (language, String),
        (clipboard_sync, bool)
    );
}
