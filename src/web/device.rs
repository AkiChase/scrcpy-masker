use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use rand::Rng;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::{broadcast, mpsc::UnboundedSender};

use crate::{
    config::LocalConfig,
    scrcpy::{
        adb::{Adb, Device},
        control_msg::ScrcpyControlMsg,
        controller::ControllerCommand,
    },
    utils::{
        relate_to_root_path,
        share::{AdbPath, ControlledDevice},
    },
    web::{JsonResponse, WebServerError},
};

#[derive(Debug, Clone)]
pub struct AppStateDevice {
    cs_tx: broadcast::Sender<ScrcpyControlMsg>,
    d_tx: UnboundedSender<ControllerCommand>,
}

pub fn routers(
    cs_tx: broadcast::Sender<ScrcpyControlMsg>,
    d_tx: UnboundedSender<ControllerCommand>,
) -> Router {
    Router::new()
        .route("/device_list", get(device_list))
        .route("/control_device", post(control_device))
        .route("/decontrol_device", post(decontrol_device))
        .route("/control/set_display_power", post(set_display_power))
        .with_state(AppStateDevice { cs_tx, d_tx })
}

async fn device_list() -> Result<JsonResponse, WebServerError> {
    let controlled_devices = ControlledDevice::get_device_list().await;
    let all_devices = Adb::new(AdbPath::get().await)
        .devices()
        .map_err(|e| WebServerError::internal_error(e))?;

    Ok(JsonResponse::success(
        "Successfully obtained device list",
        Some(json!({
            "controlled_devices": controlled_devices,
            "adb_devices": all_devices,
        })),
    ))
}

fn gen_scid() -> String {
    let mut rng = rand::rng();
    (0..8)
        .map(|_| rng.random_range(1..=9).to_string())
        .collect()
}

#[derive(Deserialize)]
struct PostDataControlDevice {
    device_id: String,
    vedio: bool,
    audio: bool,
}

async fn control_device(
    State(state): State<AppStateDevice>,
    Json(payload): Json<PostDataControlDevice>,
) -> Result<JsonResponse, WebServerError> {
    let device_id = payload.device_id;
    let vedio = payload.vedio;
    let audio = payload.audio;

    let device_list = ControlledDevice::get_device_list().await;
    // check if device is controlled
    if device_list
        .iter()
        .any(|device| device.device_id == device_id)
    {
        return Err(WebServerError(
            400,
            format!("Device is already controlled: {}", device_id),
        ));
    }

    // prepare for scrcpy app
    let scid = gen_scid();
    let version = "2.4";
    let scrcpy_path = relate_to_root_path(["assets", &format!("scrcpy-mask-server-v{}", version)]);
    Device::push(
        &device_id,
        scrcpy_path.to_str().unwrap(),
        "/data/local/tmp/scrcpy-server.jar",
    )
    .map_err(|e| WebServerError(500, e))?;
    log::info!("[WebServe] Push scrcpy server to device successfully");

    Device::reverse(
        &device_id,
        &format!("localabstract:scrcpy_{}", scid),
        &format!("tcp:{}", LocalConfig::get_controller_port()),
    )
    .map_err(|e| WebServerError(500, e))?;
    log::info!("[WebServe] Reverse local port to device successfully");

    // create device
    let main = device_list.len() == 0;
    let mut socket_id: Vec<String> = Vec::new();
    let mut commands: Vec<ControllerCommand> = Vec::new();
    if main {
        socket_id.push("main_control".to_string());
        commands.push(ControllerCommand::ConnectMainControl(scid.clone()));
        if vedio {
            socket_id.push("main_video".to_string());
            commands.push(ControllerCommand::ConnectMainVideo(scid.clone()));
        }
        if audio {
            socket_id.push("main_audio".to_string());
            commands.push(ControllerCommand::ConnectMainAudio(scid.clone()));
        }
    } else {
        socket_id.push(format!("sub_control_{}", scid));
        commands.push(ControllerCommand::ConnectSubControl(scid.clone()));
    }

    ControlledDevice::add_device(device_id.clone(), scid.clone(), main, socket_id).await;
    // send command to controller server
    for cmd in commands {
        state.d_tx.send(cmd).unwrap();
    }

    // run scrcpy app
    let h = Device::shell_process(
        &device_id,
        [
            "CLASSPATH=/data/local/tmp/scrcpy-server.jar",
            "app_process",
            "/",
            "com.genymobile.scrcpy.Server",
            version,
            &format!("scid={}", scid),
            &format!("video={}", vedio),
            &format!("audio={}", audio),
        ],
    );

    let scid_copy = scid.clone();
    tokio::spawn(async move {
        h.await.unwrap().unwrap();
        log::info!("[WebServe] Removing device after scrcpy app exit");
        ControlledDevice::remove_device(&scid_copy).await;
    });

    Ok(JsonResponse::success(
        "Trying to start scrcpy app",
        Some(json!({"scid": scid, "device_id": device_id})),
    ))
}

#[derive(Deserialize)]
struct PostDataDeControlDevice {
    device_id: String,
}

async fn decontrol_device(
    State(state): State<AppStateDevice>,
    Json(payload): Json<PostDataDeControlDevice>,
) -> Result<JsonResponse, WebServerError> {
    let device_id = payload.device_id;
    let device_list = ControlledDevice::get_device_list().await;
    for device in device_list {
        if device.device_id == device_id {
            let scid = device.scid.clone();
            if device.main {
                state
                    .d_tx
                    .send(ControllerCommand::ShutdownMain(scid))
                    .unwrap();
            } else {
                state
                    .d_tx
                    .send(ControllerCommand::ShutdownSub(scid))
                    .unwrap();
            }
            ControlledDevice::remove_device(&device.scid).await;
            return Ok(JsonResponse::success(
                format!("Decontrol device: {}", device_id),
                None,
            ));
        }
    }
    Err(WebServerError::bad_request(format!(
        "Device not found: {}",
        device_id
    )))
}

#[derive(Deserialize)]
struct PostDataSetDisplayPower {
    mode: bool,
}
async fn set_display_power(
    State(state): State<AppStateDevice>,
    Json(payload): Json<PostDataSetDisplayPower>,
) -> Result<JsonResponse, WebServerError> {
    if !ControlledDevice::is_any_device_controlled().await {
        return Err(WebServerError::bad_request("No device under controlled"));
    }

    state
        .cs_tx
        .send(ScrcpyControlMsg::SetDisplayPower { mode: payload.mode })
        .unwrap();
    Ok(JsonResponse::success(
        "Set display power successfully",
        None,
    ))
}
