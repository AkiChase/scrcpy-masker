use std::fs;

use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use flume::Sender;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::oneshot;

use crate::{
    config::LocalConfig,
    mask::{
        mapping::config::{
            MappingConfig, load_mapping_config, save_mapping_config, validate_mapping_config,
        },
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
        .route("/delete_mapping", post(delete_mapping))
        .route("/update_mapping", post(update_mapping))
        .route("/read_mapping", post(read_mapping))
        .route("/get_mapping_list", get(get_mapping_list))
        .with_state(AppStatMapping { m_tx })
}

#[derive(Deserialize)]
struct PostDataChangeActiveMapping {
    file: String,
}

async fn change_active_mapping(
    State(state): State<AppStatMapping>,
    Json(payload): Json<PostDataChangeActiveMapping>,
) -> Result<JsonResponse, WebServerError> {
    match load_mapping_config(&payload.file) {
        Ok((mapping_config, input_config)) => {
            let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<String, String>>();
            state
                .m_tx
                .send_async((
                    MaskCommand::ActiveMappingChange {
                        mapping: mapping_config,
                        input: input_config,
                        file: payload.file.clone(),
                    },
                    oneshot_tx,
                ))
                .await
                .unwrap();
            match oneshot_rx.await.unwrap() {
                Ok(msg) => {
                    LocalConfig::set_active_mapping_file(payload.file);
                    Ok(JsonResponse::success(msg, None))
                }
                Err(e) => Err(WebServerError::bad_request(e)),
            }
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

async fn create_mapping(
    Json(payload): Json<PostDataNewMapping>,
) -> Result<JsonResponse, WebServerError> {
    if !is_safe_file_name(payload.file.as_ref()) {
        return Err(WebServerError::bad_request(format!(
            "Mapping config name is not safe: {}",
            payload.file
        )));
    }

    let config_path = relate_to_root_path(["local", "mapping", &payload.file]);
    if config_path.exists() {
        return Err(WebServerError::bad_request(format!(
            "Mapping config already exists: {}",
            payload.file
        )));
    }

    validate_mapping_config(&payload.config).map_err(|e| WebServerError::bad_request(e))?;

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
    Json(payload): Json<PostDataMappingFile>,
) -> Result<JsonResponse, WebServerError> {
    if !is_safe_file_name(payload.file.as_ref()) {
        return Err(WebServerError::bad_request(format!(
            "Mapping config name is not safe: {}",
            payload.file
        )));
    }

    let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<String, String>>();
    state
        .m_tx
        .send_async((MaskCommand::GetActiveMapping, oneshot_tx))
        .await
        .unwrap();
    let file = oneshot_rx.await.unwrap().unwrap();
    if file == payload.file {
        return Err(WebServerError::bad_request(
            "Cannot delete active mapping".to_string(),
        ));
    }
    let file_path = relate_to_root_path(["local", "mapping", &payload.file]);
    if !file_path.exists() {
        return Err(WebServerError::bad_request(format!(
            "Mapping config not exists: {}",
            payload.file
        )));
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

async fn update_mapping(
    State(state): State<AppStatMapping>,
    Json(payload): Json<PostDataNewMapping>,
) -> Result<JsonResponse, WebServerError> {
    if !is_safe_file_name(payload.file.as_ref()) {
        return Err(WebServerError::bad_request(format!(
            "Mapping config name is not safe: {}",
            payload.file
        )));
    }

    validate_mapping_config(&payload.config).map_err(|e| WebServerError::bad_request(e))?;
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
        match load_mapping_config(&payload.file) {
            Ok((mapping_config, input_config)) => {
                log::info!(
                    "[Mask] Using mapping config {}: {}",
                    payload.file,
                    mapping_config.title
                );
                let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<String, String>>();
                state
                    .m_tx
                    .send_async((
                        MaskCommand::ActiveMappingChange {
                            mapping: mapping_config,
                            input: input_config,
                            file: payload.file.clone(),
                        },
                        oneshot_tx,
                    ))
                    .await
                    .unwrap();
                match oneshot_rx.await.unwrap() {
                    Ok(msg) => {
                        LocalConfig::set_active_mapping_file(payload.file.clone());
                        return Ok(JsonResponse::success(
                            format!(
                                "Successfully update mapping config {}. {}",
                                payload.file, msg
                            ),
                            None,
                        ));
                    }
                    Err(e) => {
                        return Err(WebServerError::bad_request(e));
                    }
                }
            }
            Err(e) => {
                return Err(WebServerError::bad_request(format!(
                    "Failed to load updated mapping config {}. {}",
                    payload.file, e
                )));
            }
        }
    }
    log::info!("[WebServer] Update mapping config {}", payload.file);
    Ok(JsonResponse::success(
        format!("Successfully update mapping config {}", payload.file),
        None,
    ))
}

async fn get_mapping_list() -> Result<JsonResponse, WebServerError> {
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

    Ok(JsonResponse::success(
        "Successfully read mapping list",
        Some(json!({
            "mapping_list": mapping_files,
        })),
    ))
}

async fn read_mapping(
    Json(payload): Json<PostDataMappingFile>,
) -> Result<JsonResponse, WebServerError> {
    if !is_safe_file_name(payload.file.as_ref()) {
        return Err(WebServerError::bad_request(format!(
            "Mapping config name is not safe: {}",
            payload.file
        )));
    }

    // load from file
    let path = relate_to_root_path(["local", "mapping", &payload.file]);
    if !path.exists() {
        return Err(WebServerError::bad_request(format!(
            "Mapping config not found {}",
            payload.file
        )));
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

    validate_mapping_config(&mapping_config).map_err(|e| {
        WebServerError::bad_request(format!("Invalid mapping config {}: {}", payload.file, e))
    })?;

    Ok(JsonResponse::success(
        format!("Successfully read mapping config {}", payload.file),
        Some(json!({
            "mapping_config": mapping_config,
        })),
    ))
}
