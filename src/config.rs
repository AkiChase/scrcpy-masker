use std::{
    fs::{File, create_dir_all},
    io::Write,
    sync::RwLock,
};

use crate::utils::relate_to_root_path;
use once_cell::sync::Lazy;
use paste::paste;
use ron::ser::{PrettyConfig, to_string_pretty};
use serde::{Deserialize, Serialize};

static CONFIG: Lazy<RwLock<LocalConfig>> = Lazy::new(|| RwLock::default());

// TODO 单独写外部脚本来捕获特定窗口，发送Post消息来设置蒙版相关配置（宽度>=高度则设置横屏相关配置，否则设置竖屏）

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            adb_path: "adb".to_string(),
            web_port: 27799,
            controller_port: 27798,
            vertical_mask_height: 720,
            horizontal_mask_width: 1280,
            vertical_position: (300, 300),
            horizontal_position: (300, 300),
            active_mapping_file: "default.ron".to_string(),
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
        let pretty = PrettyConfig::default();
        let ron_string = to_string_pretty(&Self::get(), pretty)
            .map_err(|e| format!("Cannot serialize config: {}", e))?;

        let path = relate_to_root_path(["local", "config.ron"]);
        if let Some(parent) = path.parent() {
            create_dir_all(parent)
                .map_err(|e| format!("Cannot create directory for config file: {}", e))?;
        }
        let mut file =
            File::create(path).map_err(|e| format!("Cannot create mapping config file: {}", e))?;
        file.write_all(ron_string.as_bytes())
            .map_err(|e| format!("Cannot write to mapping config file: {}", e))?;
        Ok(())
    }

    pub fn load() -> Result<(), String> {
        let path = relate_to_root_path(["local", "config.ron"]);
        let ron_string = std::fs::read_to_string(&path)
            .map_err(|e| format!("Cannot read config file {}: {}", path.to_str().unwrap(), e))?;
        let config: LocalConfig = ron::from_str(&ron_string)
            .map_err(|e| format!("Cannot deserialize mapping config: {}", e))?;
        *CONFIG.write().unwrap() = config;
        Ok(())
    }

    pub fn get() -> LocalConfig {
        CONFIG.read().unwrap().clone()
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
    );
}
