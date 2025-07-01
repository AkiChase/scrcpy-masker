use axum::{Json, Router, extract::State, routing::post};
use flume::Sender;
use serde::Deserialize;
use tokio::sync::oneshot;

use crate::{
    config::LocalConfig,
    mask::{mapping::config::load_mapping_config, mask_command::MaskCommand},
    web::{JsonResponse, WebServerError},
};

#[derive(Debug, Clone)]
pub struct AppStatMapping {
    m_tx: Sender<(MaskCommand, oneshot::Sender<Result<String, String>>)>,
}

pub fn routers(m_tx: Sender<(MaskCommand, oneshot::Sender<Result<String, String>>)>) -> Router {
    Router::new()
        .route("/change_active_mapping", post(change_active_mapping))
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
                Err(e) => {
                    log::error!("{}", e);
                    Err(WebServerError::bad_request(e))
                }
            }
        }
        Err(e) => {
            log::error!("{}", e);
            Err(WebServerError::bad_request("Failed to load mapping config"))
        }
    }
}

// TODO 增删改查mapping方案
