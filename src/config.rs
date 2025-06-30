use std::sync::RwLock;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

static CONFIG: Lazy<RwLock<LocalConfig>> = Lazy::new(|| RwLock::default());

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfig {
    // port
    pub web_port: u16,
    pub controller_port: u16,
    // adb
    pub adb_path: String,
    // mask area
    pub vertical_screen_height: u32,
    pub horizontal_screen_width: u32,
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
            vertical_screen_height: 1280,
            horizontal_screen_width: 720,
            vertical_position: (300, 300),
            horizontal_position: (300, 300),
            active_mapping_file: "default.ron".to_string(),
        }
    }
}

impl LocalConfig {
    fn save() {
        // TODO 保存到文件，注意其他函数做出修改后都要立即保存到文件
    }

    pub fn load() {
        // TODO 从文件加载
    }

    pub fn get() -> LocalConfig {
        CONFIG.read().unwrap().clone()
    }

    pub fn get_controller_port() -> u16 {
        CONFIG.read().unwrap().controller_port
    }

    pub fn set_adb_path(path: String) {
        CONFIG.write().unwrap().adb_path = path;
        Self::save();
    }

    pub fn set_active_mapping_file(file_name: String) {
        CONFIG.write().unwrap().active_mapping_file = file_name;
        Self::save();
    }
}
