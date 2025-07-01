use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use flume::Sender;
use serde::Deserialize;
use tokio::sync::oneshot;

use crate::{
    config::LocalConfig,
    mask::mask_command::MaskCommand,
    scrcpy::adb::Adb,
    utils::{mask_win_move_helper, share::ControlledDevice},
    web::{JsonResponse, WebServerError},
};

#[derive(Debug, Clone)]
pub struct AppStatConfig {
    m_tx: Sender<(MaskCommand, oneshot::Sender<Result<String, String>>)>,
}

pub fn routers(m_tx: Sender<(MaskCommand, oneshot::Sender<Result<String, String>>)>) -> Router {
    Router::new()
        .route("/get_config", get(get_config))
        .route("/update_config", post(update_config))
        .with_state(AppStatConfig { m_tx })
}

async fn get_config() -> Result<JsonResponse, WebServerError> {
    let config = LocalConfig::get();
    Ok(JsonResponse::success(
        "Successfully get config",
        Some(serde_json::to_value(&config).unwrap()),
    ))
}

#[derive(Deserialize)]
struct PostDataUpdateConfig {
    key: String,
    value: serde_json::Value,
}

async fn update_config(
    State(state): State<AppStatConfig>,
    Json(payload): Json<PostDataUpdateConfig>,
) -> Result<JsonResponse, WebServerError> {
    // sync with src/config.rs
    match payload.key.as_str() {
        "web_port" => {
            if let Some(value) = payload.value.as_u64() {
                LocalConfig::set_web_port(value as u16);
                return Ok(JsonResponse::success(
                    format!("Please restart app to apply new web_port"),
                    None,
                ));
            } else {
                return Err(WebServerError::bad_request("web_port must be u16"));
            }
        }
        "adb_path" => {
            if let Some(value) = payload.value.as_str() {
                match Adb::check_adb_path(value) {
                    Ok(_) => {
                        LocalConfig::set_adb_path(value.to_string());
                        return Ok(JsonResponse::success(
                            format!("Successfully updated adb_path"),
                            None,
                        ));
                    }
                    Err(e) => {
                        return Err(WebServerError::bad_request(format!(
                            "Failed to set adb_path, {}",
                            e
                        )));
                    }
                }
            } else {
                return Err(WebServerError::bad_request("adb_path must be string"));
            }
        }
        "controller_port" => {
            if let Some(value) = payload.value.as_u64() {
                LocalConfig::set_controller_port(value as u16);
                return Ok(JsonResponse::success(
                    format!("Please restart app to apply new controller_port"),
                    None,
                ));
            } else {
                return Err(WebServerError::bad_request("controller_port must be u16"));
            }
        }
        "vertical_mask_height" => {
            if let Some(value) = payload.value.as_u64() {
                LocalConfig::set_vertical_mask_height(value as u32);
                if let Some(main_device) = ControlledDevice::get_main_device().await {
                    let (device_w, device_h) = main_device.device_size;
                    let msg = mask_win_move_helper(device_w, device_h, &state.m_tx).await;
                    return Ok(JsonResponse::success(
                        format!("Successfully updated vertical_mask_height. {}", msg),
                        None,
                    ));
                }
            } else {
                return Err(WebServerError::bad_request(
                    "vertical_mask_height must be u32",
                ));
            }
        }
        "horizontal_mask_width" => {
            if let Some(value) = payload.value.as_u64() {
                LocalConfig::set_horizontal_mask_width(value as u32);
                if let Some(main_device) = ControlledDevice::get_main_device().await {
                    let (device_w, device_h) = main_device.device_size;
                    let msg = mask_win_move_helper(device_w, device_h, &state.m_tx).await;
                    return Ok(JsonResponse::success(
                        format!("Successfully updated horizontal_mask_width. {}", msg),
                        None,
                    ));
                }
            } else {
                return Err(WebServerError::bad_request(
                    "horizontal_mask_width must be u32",
                ));
            }
        }
        "vertical_position" => {
            if let Some(value) = payload.value.as_array() {
                if value.len() == 2 {
                    if let (Some(x), Some(y)) = (value[0].as_i64(), value[1].as_i64()) {
                        LocalConfig::set_vertical_position((x as i32, y as i32));
                        if let Some(main_device) = ControlledDevice::get_main_device().await {
                            let (device_w, device_h) = main_device.device_size;
                            let msg = mask_win_move_helper(device_w, device_h, &state.m_tx).await;
                            return Ok(JsonResponse::success(
                                format!("Successfully updated vertical_position. {}", msg),
                                None,
                            ));
                        }
                    } else {
                        return Err(WebServerError::bad_request(
                            "vertical_position must be array [i32, i32]",
                        ));
                    }
                } else {
                    return Err(WebServerError::bad_request(
                        "vertical_position must be array [i32, i32]",
                    ));
                }
            }
        }
        "horizontal_position" => {
            if let Some(value) = payload.value.as_array() {
                if value.len() == 2 {
                    if let (Some(x), Some(y)) = (value[0].as_i64(), value[1].as_i64()) {
                        LocalConfig::set_horizontal_position((x as i32, y as i32));
                        if let Some(main_device) = ControlledDevice::get_main_device().await {
                            let (device_w, device_h) = main_device.device_size;
                            let msg = mask_win_move_helper(device_w, device_h, &state.m_tx).await;
                            return Ok(JsonResponse::success(
                                format!("Successfully updated horizontal_position. {}", msg),
                                None,
                            ));
                        }
                    } else {
                        return Err(WebServerError::bad_request(
                            "horizontal_position must be array [i32, i32]",
                        ));
                    }
                } else {
                    return Err(WebServerError::bad_request(
                        "horizontal_position must be array [i32, i32]",
                    ));
                }
            }
        }
        _ => {
            return Err(WebServerError::bad_request(format!(
                "unknown config key: {}",
                payload.key
            )));
        }
    };

    Ok(JsonResponse::success(
        format!("Successfully updated config.{}", payload.key),
        None,
    ))
}
