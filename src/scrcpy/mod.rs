use serde::Serialize;

pub mod adb;
pub mod connection;
pub mod constant;
pub mod control_msg;
pub mod controller;

#[derive(Clone, Serialize)]
pub struct ScrcpyDevice {
    pub device_id: String,
    pub scid: String,
    pub socket_ids: Vec<String>,
    pub name: String,
    pub main: bool,
}

impl ScrcpyDevice {
    pub fn new(device_id: String, scid: String, main: bool, socket_ids: Vec<String>) -> Self {
        Self {
            device_id,
            scid,
            socket_ids,
            name: "Unknow".to_string(),
            main,
        }
    }
}
