use std::fs;

use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use bevy::math::Vec2;
use flume::Sender;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::oneshot;

use crate::{
    config::LocalConfig,
    mask::{
        mapping::config::{MappingConfig, MappingType, save_mapping_config},
        mask_command::MaskCommand,
    },
    utils::{is_safe_file_name, relate_to_root_path},
    web::{JsonResponse, WebServerError},
};

#[derive(Debug, Clone)]
pub struct AppStatMapping {
    m_tx: Sender<(MaskCommand, oneshot::Sender<Result<String, String>>)>,
}

pub fn routers(m_tx: Sender<(MaskCommand, oneshot::Sender<Result<String, String>>)>) -> Router {
    Router::new()
        .route("/change_active_mapping", post(change_active_mapping))
        .route("/create_mapping", post(create_mapping))
        .route("/rename_mapping", post(rename_mapping))
        .route("/duplicate_mapping", post(duplicate_mapping))
        .route("/delete_mapping", post(delete_mapping))
        .route("/update_mapping", post(update_mapping))
        .route("/read_mapping", post(read_mapping))
        .route("/get_mapping_list", get(get_mapping_list))
        .route("/migrate_mapping", post(migrate_mapping))
        .with_state(AppStatMapping { m_tx })
}

#[derive(Deserialize)]
struct PostDataChangeActiveMapping {
    file: String,
}

async fn change_active_mapping(
    State(state): State<AppStatMapping>,
    Json(mut payload): Json<PostDataChangeActiveMapping>,
) -> Result<JsonResponse, WebServerError> {
    if !payload.file.ends_with(".json") {
        payload.file.push_str(".json");
    }

    let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<String, String>>();
    state
        .m_tx
        .send_async((
            MaskCommand::LoadAndActivateMappingConfig {
                file_name: payload.file.clone(),
            },
            oneshot_tx,
        ))
        .await
        .unwrap();
    match oneshot_rx.await.unwrap() {
        Ok(_) => {
            LocalConfig::set_active_mapping_file(payload.file.clone());
            log::info!("[WebServer] Set active mapping {}", payload.file);
            Ok(JsonResponse::success(
                format!("Successfully set active mapping {}", payload.file),
                None,
            ))
        }
        Err(e) => Err(WebServerError::bad_request(format!(
            "Failed to load mapping config {}. {}",
            payload.file, e
        ))),
    }
}

#[derive(Deserialize)]
struct PostDataNewMapping {
    file: String,
    config: MappingConfig,
}

async fn validate_config(
    m_tx: &Sender<(MaskCommand, oneshot::Sender<Result<String, String>>)>,
    config: &MappingConfig,
) -> Result<(), String> {
    let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<String, String>>();
    m_tx.send_async((
        MaskCommand::ValidateMappingConfig {
            config: config.clone(),
        },
        oneshot_tx,
    ))
    .await
    .unwrap();
    match oneshot_rx.await.unwrap() {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

async fn create_mapping(
    State(state): State<AppStatMapping>,
    Json(mut payload): Json<PostDataNewMapping>,
) -> Result<JsonResponse, WebServerError> {
    if !payload.file.ends_with(".json") {
        payload.file.push_str(".json");
    }

    let bad_request =
        |msg| -> Result<JsonResponse, WebServerError> { Err(WebServerError::bad_request(msg)) };

    if !is_safe_file_name(payload.file.as_ref()) {
        return bad_request(format!("Mapping config name is not safe: {}", payload.file));
    }

    let config_path = relate_to_root_path(["local", "mapping", &payload.file]);
    if config_path.exists() {
        return bad_request(format!("Mapping config already exists: {}", payload.file));
    }

    validate_config(&state.m_tx, &payload.config)
        .await
        .map_err(|e| WebServerError::bad_request(e))?;

    // save to file
    save_mapping_config(&payload.config, &config_path)
        .map_err(|e| WebServerError::bad_request(e))?;

    log::info!("[WebServer] Create new mapping config {}", payload.file);
    Ok(JsonResponse::success(
        format!("Successfully create new mapping config {}", payload.file),
        None,
    ))
}

#[derive(Deserialize)]
struct PostDataMappingFile {
    file: String,
}

async fn delete_mapping(
    State(state): State<AppStatMapping>,
    Json(mut payload): Json<PostDataMappingFile>,
) -> Result<JsonResponse, WebServerError> {
    if !payload.file.ends_with(".json") {
        payload.file.push_str(".json");
    }

    let bad_request =
        |msg| -> Result<JsonResponse, WebServerError> { Err(WebServerError::bad_request(msg)) };

    if !is_safe_file_name(payload.file.as_ref()) {
        return bad_request(format!("Mapping config name is not safe: {}", payload.file));
    }

    let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<String, String>>();
    state
        .m_tx
        .send_async((MaskCommand::GetActiveMapping, oneshot_tx))
        .await
        .unwrap();
    let file = oneshot_rx.await.unwrap().unwrap();
    if file == payload.file {
        return bad_request("Cannot delete active mapping".to_string());
    }
    let file_path = relate_to_root_path(["local", "mapping", &payload.file]);
    if !file_path.exists() {
        return bad_request(format!("Mapping config not exists: {}", payload.file));
    }
    fs::remove_file(file_path).map_err(|e| {
        WebServerError::bad_request(format!(
            "Failed to delete mapping config {}: {}",
            payload.file, e
        ))
    })?;

    log::info!("[WebServer] Delete mapping config {}", payload.file);
    Ok(JsonResponse::success(
        format!("Successfully delete mapping config {}", payload.file),
        None,
    ))
}

#[derive(Deserialize)]
struct PostDataRenameMappingFile {
    file: String,
    new_file: String,
}

async fn rename_mapping(
    State(state): State<AppStatMapping>,
    Json(mut payload): Json<PostDataRenameMappingFile>,
) -> Result<JsonResponse, WebServerError> {
    if !payload.file.ends_with(".json") {
        payload.file.push_str(".json");
    }

    if !payload.new_file.ends_with(".json") {
        payload.new_file.push_str(".json");
    }

    let bad_request =
        |msg| -> Result<JsonResponse, WebServerError> { Err(WebServerError::bad_request(msg)) };

    if !is_safe_file_name(payload.file.as_ref()) {
        return bad_request(format!("Mapping config name is not safe: {}", payload.file));
    }
    if !is_safe_file_name(payload.new_file.as_ref()) {
        return bad_request(format!(
            "New mapping config name is not safe: {}",
            payload.new_file
        ));
    }

    // rename file
    let old_path = relate_to_root_path(["local", "mapping", &payload.file]);
    if !old_path.exists() {
        return bad_request(format!(
            "Mapping config not found: {}",
            old_path.to_str().unwrap()
        ));
    }
    let new_path = relate_to_root_path(["local", "mapping", &payload.new_file]);
    if new_path.exists() {
        return bad_request(format!(
            "Mapping config already exists: {}",
            new_path.to_str().unwrap()
        ));
    }
    fs::rename(old_path, new_path).map_err(|e| WebServerError::internal_error(e.to_string()))?;

    // get active mapping file
    let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<String, String>>();
    state
        .m_tx
        .send_async((MaskCommand::GetActiveMapping, oneshot_tx))
        .await
        .unwrap();
    let file = oneshot_rx.await.unwrap().unwrap();
    if file == payload.file {
        // if active, set new active mapping
        let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<String, String>>();
        state
            .m_tx
            .send_async((
                MaskCommand::LoadAndActivateMappingConfig {
                    file_name: payload.new_file.clone(),
                },
                oneshot_tx,
            ))
            .await
            .unwrap();
        match oneshot_rx.await.unwrap() {
            Ok(_) => {
                LocalConfig::set_active_mapping_file(payload.new_file.clone());
                let msg = format!(
                    "Successfully rename mapping config from {} to {}. Successfully set active mapping {}",
                    payload.file, payload.new_file, payload.new_file
                );
                log::info!("[WebServer] {}", msg);
                return Ok(JsonResponse::success(msg, None));
            }
            Err(e) => {
                return Err(WebServerError::bad_request(format!(
                    "Failed to load mapping config {}. {}",
                    payload.new_file, e
                )));
            }
        }
    }

    let msg = format!(
        "Successfully rename mapping config from {} to {}",
        payload.file, payload.new_file
    );
    log::info!("[WebServer] {}", msg);
    Ok(JsonResponse::success(msg, None))
}

#[derive(Deserialize)]
struct PostDataDuplicateMappingFile {
    file: String,
    new_file: String,
}

async fn duplicate_mapping(
    Json(mut payload): Json<PostDataDuplicateMappingFile>,
) -> Result<JsonResponse, WebServerError> {
    if !payload.file.ends_with(".json") {
        payload.file.push_str(".json");
    }

    if !payload.new_file.ends_with(".json") {
        payload.new_file.push_str(".json");
    }

    let bad_request =
        |msg| -> Result<JsonResponse, WebServerError> { Err(WebServerError::bad_request(msg)) };

    if !is_safe_file_name(payload.file.as_ref()) {
        return bad_request(format!("Mapping config name is not safe: {}", payload.file));
    }
    if !is_safe_file_name(payload.new_file.as_ref()) {
        return bad_request(format!(
            "New mapping config name is not safe: {}",
            payload.new_file
        ));
    }

    let old_path = relate_to_root_path(["local", "mapping", &payload.file]);
    if !old_path.exists() {
        return bad_request(format!(
            "Mapping config not found: {}",
            old_path.to_str().unwrap()
        ));
    }
    let new_path = relate_to_root_path(["local", "mapping", &payload.new_file]);
    if new_path.exists() {
        return bad_request(format!(
            "Mapping config already exists: {}",
            new_path.to_str().unwrap()
        ));
    }
    fs::copy(old_path, new_path).map_err(|e| WebServerError::internal_error(e.to_string()))?;
    log::info!(
        "[WebServer] Copy mapping config from {} to {}",
        payload.file,
        payload.new_file
    );
    Ok(JsonResponse::success(
        format!(
            "Successfully copy mapping config from {} to {}",
            payload.file, payload.new_file
        ),
        None,
    ))
}

async fn update_mapping(
    State(state): State<AppStatMapping>,
    Json(mut payload): Json<PostDataNewMapping>,
) -> Result<JsonResponse, WebServerError> {
    if !payload.file.ends_with(".json") {
        payload.file.push_str(".json");
    }

    let bad_request =
        |msg| -> Result<JsonResponse, WebServerError> { Err(WebServerError::bad_request(msg)) };

    if !is_safe_file_name(payload.file.as_ref()) {
        return bad_request(format!("Mapping config name is not safe: {}", payload.file));
    }

    validate_config(&state.m_tx, &payload.config)
        .await
        .map_err(|e| WebServerError::bad_request(e))?;

    // save to file
    let config_path = relate_to_root_path(["local", "mapping", &payload.file]);
    save_mapping_config(&payload.config, &config_path)
        .map_err(|e| WebServerError::bad_request(e))?;

    // get active mapping file
    let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<String, String>>();
    state
        .m_tx
        .send_async((MaskCommand::GetActiveMapping, oneshot_tx))
        .await
        .unwrap();
    let file = oneshot_rx.await.unwrap().unwrap();
    if file == payload.file {
        // if active, refresh active mapping
        let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<String, String>>();
        state
            .m_tx
            .send_async((
                MaskCommand::LoadAndActivateMappingConfig {
                    file_name: payload.file.clone(),
                },
                oneshot_tx,
            ))
            .await
            .unwrap();
        match oneshot_rx.await.unwrap() {
            Ok(_) => {
                LocalConfig::set_active_mapping_file(payload.file.clone());
                let msg = format!(
                    "Successfully update and activate mapping config {}",
                    payload.file
                );
                log::info!("[WebServer] {}", msg);
                Ok(JsonResponse::success(msg, None))
            }
            Err(e) => Err(WebServerError::bad_request(format!(
                "Failed to load updated mapping config {}. {}",
                payload.file, e
            ))),
        }
    } else {
        log::info!("[WebServer] Update mapping config {}", payload.file);
        Ok(JsonResponse::success(
            format!("Successfully update mapping config {}", payload.file),
            None,
        ))
    }
}

async fn get_mapping_list(
    State(state): State<AppStatMapping>,
) -> Result<JsonResponse, WebServerError> {
    let dir_path = relate_to_root_path(["local", "mapping"]);
    let entries = fs::read_dir(dir_path).map_err(|e| {
        WebServerError::bad_request(format!("Unable to read mapping config dir: {}", e))
    })?;

    let mut mapping_files: Vec<String> = Vec::new();
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();

        if path.is_file() {
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Some(file_name) = path.file_name() {
                    if let Some(name_str) = file_name.to_str() {
                        mapping_files.push(name_str.to_string());
                    }
                }
            }
        }
    }

    // get active mapping file
    let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<String, String>>();
    state
        .m_tx
        .send_async((MaskCommand::GetActiveMapping, oneshot_tx))
        .await
        .unwrap();
    let file = oneshot_rx.await.unwrap().unwrap();

    Ok(JsonResponse::success(
        "Successfully read mapping list",
        Some(json!({
            "mapping_list": mapping_files,
            "active_mapping": file,
        })),
    ))
}

async fn read_mapping(
    State(state): State<AppStatMapping>,
    Json(mut payload): Json<PostDataMappingFile>,
) -> Result<JsonResponse, WebServerError> {
    if !payload.file.ends_with(".json") {
        payload.file.push_str(".json");
    }

    let bad_request =
        |msg| -> Result<JsonResponse, WebServerError> { Err(WebServerError::bad_request(msg)) };

    if !is_safe_file_name(payload.file.as_ref()) {
        return bad_request(format!("Mapping config name is not safe: {}", payload.file));
    }

    // load from file
    let path = relate_to_root_path(["local", "mapping", &payload.file]);
    if !path.exists() {
        return bad_request(format!("Mapping config not found {}", payload.file));
    }
    let config_string = std::fs::read_to_string(path).map_err(|e| {
        WebServerError::bad_request(format!(
            "Cannot read mapping config {}: {}",
            payload.file, e
        ))
    })?;
    let mapping_config: MappingConfig = serde_json::from_str(&config_string).map_err(|e| {
        WebServerError::bad_request(format!(
            "Cannot deserialize mapping config {}: {}",
            payload.file, e
        ))
    })?;

    validate_config(&state.m_tx, &mapping_config)
        .await
        .map_err(|e| {
            WebServerError::bad_request(format!("Invalid mapping config {}: {}", payload.file, e))
        })?;

    Ok(JsonResponse::success(
        format!("Successfully read mapping config {}", payload.file),
        Some(json!({
            "mapping_config": mapping_config,
        })),
    ))
}

#[derive(Deserialize)]
struct PostDataMigrateMappingFile {
    file: String,
    new_file: String,
    width: u32,
    height: u32,
}

async fn migrate_mapping(
    Json(mut payload): Json<PostDataMigrateMappingFile>,
) -> Result<JsonResponse, WebServerError> {
    if !payload.file.ends_with(".json") {
        payload.file.push_str(".json");
    }

    if !payload.new_file.ends_with(".json") {
        payload.new_file.push_str(".json");
    }

    let bad_request =
        |msg| -> Result<JsonResponse, WebServerError> { Err(WebServerError::bad_request(msg)) };

    if !is_safe_file_name(payload.file.as_ref()) {
        return bad_request(format!("Mapping config name is not safe: {}", payload.file));
    }

    let old_path = relate_to_root_path(["local", "mapping", &payload.file]);
    if !old_path.exists() {
        return bad_request(format!("Mapping config does not exist: {}", payload.file));
    }

    let new_path = relate_to_root_path(["local", "mapping", &payload.new_file]);
    if new_path.exists() {
        return bad_request(format!(
            "Mapping config already exists: {}",
            payload.new_file
        ));
    }

    let config_string = std::fs::read_to_string(old_path).map_err(|e| {
        WebServerError::bad_request(format!(
            "Cannot read mapping config {}: {}",
            payload.file, e
        ))
    })?;
    let mut mapping_config: MappingConfig = serde_json::from_str(&config_string).map_err(|e| {
        WebServerError::bad_request(format!(
            "Cannot deserialize mapping config {}: {}",
            payload.file, e
        ))
    })?;

    if payload.width == 0 || payload.height == 0 {
        return bad_request(format!(
            "Invalid size: {}, {}",
            payload.width, payload.height
        ));
    }

    let scale = Vec2::new(
        payload.width as f32 / mapping_config.original_size.width as f32,
        payload.height as f32 / mapping_config.original_size.height as f32,
    );

    mapping_config.original_size.width = payload.width;
    mapping_config.original_size.height = payload.height;

    mapping_config
        .mappings
        .iter_mut()
        .for_each(|mapping| match mapping {
            MappingType::SingleTap(m) => {
                m.position *= scale;
            }
            MappingType::RepeatTap(m) => {
                m.position *= scale;
            }
            MappingType::MultipleTap(m) => {
                m.items.iter_mut().for_each(|item| {
                    item.position *= scale;
                });
            }
            MappingType::Swipe(m) => {
                m.positions.iter_mut().for_each(|p| {
                    *p *= scale;
                });
            }
            MappingType::DirectionPad(m) => {
                m.position *= scale;
                m.max_offset_x *= scale.x;
                m.max_offset_y *= scale.y;
            }
            MappingType::MouseCastSpell(m) => {
                m.position *= scale;
                m.cast_radius *= scale.y;
                m.center *= scale;
                m.drag_radius *= scale.y;
            }
            MappingType::PadCastSpell(m) => {
                m.drag_radius *= scale.y;
                m.position *= scale;
            }
            MappingType::CancelCast(m) => {
                m.position *= scale;
            }
            MappingType::Observation(m) => {
                m.position *= scale;
            }
            MappingType::Fps(m) => {
                m.position *= scale;
            }
            MappingType::Fire(m) => {
                m.position *= scale;
            }
            MappingType::RawInput(m) => {
                m.position *= scale;
            }
            MappingType::Script(m) => {
                m.position *= scale;
            }
        });

    // save to file
    save_mapping_config(&mapping_config, &new_path).map_err(|e| WebServerError::bad_request(e))?;

    log::info!(
        "[WebServer] Migrate mapping config from {} to {}",
        payload.file,
        payload.new_file
    );
    Ok(JsonResponse::success(
        format!(
            "Successfully migrate mapping config from {} to {}",
            payload.file, payload.new_file
        ),
        None,
    ))
}
