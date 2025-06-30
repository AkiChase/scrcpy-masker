use axum::{Json, Router, extract::State, routing::post};
use flume::Sender;
use serde::Deserialize;

use crate::{
    mask::mask_command::MaskCommand,
    web::{JsonResponse, WebServerError},
};

#[derive(Debug, Clone)]
pub struct AppStateMask {
    m_tx: Sender<MaskCommand>,
}

pub fn routers(m_tx: Sender<MaskCommand>) -> Router {
    Router::new()
        .route("/win_move", post(win_move))
        .with_state(AppStateMask { m_tx })
}

#[derive(Deserialize)]
struct PostDataWinMove {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

async fn win_move(
    State(state): State<AppStateMask>,
    Json(payload): Json<PostDataWinMove>,
) -> Result<JsonResponse, WebServerError> {
    if payload.right - payload.left < 100 || payload.bottom - payload.top < 100 {
        return Err(WebServerError::bad_request(
            "The window size must be at least 100x100".to_string(),
        ));
    }

    state
        .m_tx
        .send_async(MaskCommand::WinMove {
            left: payload.left,
            top: payload.top,
            right: payload.right,
            bottom: payload.bottom,
        })
        .await
        .unwrap();
    Ok(JsonResponse::success("Move window successfully", None))
}
