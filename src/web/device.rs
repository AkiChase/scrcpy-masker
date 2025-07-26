use std::time::Duration;

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use rand::Rng;
use serde::Deserialize;
use serde_json::json;
use tokio::{
    sync::{broadcast, mpsc::UnboundedSender},
    time::sleep,
};

use crate::{
    config::LocalConfig,
    scrcpy::{
        adb::{Adb, Device},
        control_msg::ScrcpyControlMsg,
        controller::ControllerCommand,
    },
    utils::{relate_to_root_path, share::ControlledDevice},
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
        .route("/adb_connect", post(adb_connect))
        .route("/adb_pair", post(adb_pair))
        .route("/adb_screenshot", post(adb_screenshot))
        .with_state(AppStateDevice { cs_tx, d_tx })
}

async fn device_list() -> Result<JsonResponse, WebServerError> {
    let controlled_devices = ControlledDevice::get_device_list().await;
    let config = LocalConfig::get();
    let all_devices = Adb::new(config.adb_path)
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
    let suffix: String = (0..6)
        .map(|_| rng.random_range(1..=9).to_string())
        .collect();
    format!("10{}", suffix) // ensure 8 digits(HEX) and less than MAX_INT32
}

#[derive(Deserialize)]
struct PostDataControlDevice {
    device_id: String,
    video: bool,
    audio: bool,
}

async fn control_device(
    State(state): State<AppStateDevice>,
    Json(payload): Json<PostDataControlDevice>,
) -> Result<JsonResponse, WebServerError> {
    let device_id = payload.device_id;
    let video = payload.video;
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

    let remote = format!("localabstract:scrcpy_{}", scid);
    let local = format!("tcp:{}", LocalConfig::get().controller_port);
    Device::reverse(&device_id, &remote, &local).map_err(|e| WebServerError(500, e))?;
    log::info!("[WebServe] Reverse {} to {} successfully", remote, local);

    // create device
    let main = device_list.len() == 0;
    let mut socket_id: Vec<String> = Vec::new();
    let mut commands: Vec<ControllerCommand> = Vec::new();
    if main {
        if video {
            socket_id.push("main_video".to_string());
            commands.push(ControllerCommand::ConnectMainVideo(scid.clone()));
        }
        if audio {
            socket_id.push("main_audio".to_string());
            commands.push(ControllerCommand::ConnectMainAudio(scid.clone()));
        }
        socket_id.push("main_control".to_string());
        commands.push(ControllerCommand::ConnectMainControl(scid.clone()));
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
    sleep(Duration::from_millis(500)).await;
    log::info!("[WebServe] Start scrcpy app...");
    let h = Device::shell_process(
        &device_id,
        [
            "CLASSPATH=/data/local/tmp/scrcpy-server.jar",
            "app_process",
            "/",
            "com.genymobile.scrcpy.Server",
            version,
            &format!("scid={}", scid),
            &format!("video={}", video),
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
        "Set display power of all controlled devices successfully",
        None,
    ))
}

#[derive(Deserialize)]
struct PostDataAddress {
    address: String,
}

async fn adb_connect(Json(payload): Json<PostDataAddress>) -> Result<JsonResponse, WebServerError> {
    let config = LocalConfig::get();
    match Adb::new(config.adb_path).connect_device(&payload.address) {
        Ok(_) => Ok(JsonResponse::success(
            format!("Adb connect {} successfully", payload.address),
            None,
        )),
        Err(e) => Err(WebServerError::bad_request(format!(
            "Adb connect {} failed: {}",
            payload.address, e
        ))),
    }
}

#[derive(Deserialize)]
struct PostDataAdbPair {
    address: String,
    code: String,
}

async fn adb_pair(Json(payload): Json<PostDataAdbPair>) -> Result<JsonResponse, WebServerError> {
    let config = LocalConfig::get();
    match Adb::new(config.adb_path).pair_device(&payload.address, &payload.code) {
        Ok(_) => Ok(JsonResponse::success(
            format!(
                "Adb pair {} with code {} successfully",
                payload.address, payload.code
            ),
            None,
        )),
        Err(e) => Err(WebServerError::bad_request(format!(
            "Adb pair {} with code {} failed: {}",
            payload.address, payload.code, e
        ))),
    }
}

#[derive(Deserialize)]
struct PostDataId {
    id: String,
}

async fn adb_screenshot(
    Json(payload): Json<PostDataId>,
) -> Result<impl IntoResponse, WebServerError> {
    let src = "/data/local/tmp/_screenshot_scrcpy_masker.png";

    Device::shell(
        &payload.id,
        ["screencap", "-p", src],
        &mut std::io::stdout(),
    )
    .map_err(|e| {
        WebServerError::bad_request(format!(
            "Failed to take screenshot for device {}: {}",
            payload.id, e
        ))
    })?;

    let mut image_bytes = Vec::<u8>::new();
    Device::pull(&payload.id, src.to_string(), &mut image_bytes)
        .map_err(|e| WebServerError::bad_request(format!("Failed get screenshot file. {}", e)))?;

    Device::shell(&payload.id, ["rm", src], &mut std::io::stdout()).map_err(|e| {
        WebServerError::bad_request(format!(
            "Failed to remove screenshot file for device {}: {}",
            payload.id, e
        ))
    })?;

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("image/png"));
    headers.insert("Cache-Control", HeaderValue::from_static("no-cache"));

    Ok((StatusCode::OK, headers, image_bytes))
}
