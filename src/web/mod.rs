pub mod device;
pub mod mask;

use std::{net::SocketAddrV4, thread};

use axum::{
    Json, Router,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use flume::Sender;
use serde::Serialize;
use serde_json::Value;
use tokio::sync::{broadcast, mpsc::UnboundedSender};
use tower_http::services::ServeDir;

use crate::{
    mask::mask_command::MaskCommand,
    scrcpy::{control_msg::ScrcpyControlMsg, controller::ControllerCommand},
    utils::relate_to_root_path,
};

pub struct Server;

impl Server {
    pub fn start(
        addr: SocketAddrV4,
        cs_tx: broadcast::Sender<ScrcpyControlMsg>,
        d_tx: UnboundedSender<ControllerCommand>,
        m_tx: Sender<MaskCommand>,
    ) {
        thread::spawn(move || {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async move {
                    Server::run_server(addr, cs_tx, d_tx, m_tx).await;
                });
        });
    }

    async fn run_server(
        addr: SocketAddrV4,
        cs_tx: broadcast::Sender<ScrcpyControlMsg>,
        d_tx: UnboundedSender<ControllerCommand>,
        m_tx: Sender<MaskCommand>,
    ) {
        log::info!("[WebServe] Starting web server on: {}", addr);
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

        let ip_str = if addr.ip().is_unspecified() || addr.ip().is_loopback() {
            "localhost"
        } else {
            &addr.ip().to_string()
        };
        let url = format!("http://{}:{}", ip_str, addr.port());
        log::info!("[WebServe] Web server is accessible at: {}", url);
        // TODO 启动网页
        //opener::open(url)
        //     .unwrap_or_else(|e| log::error!("[WebServe] Failed to open browser: {}", e));

        axum::serve(listener, Self::app(cs_tx, d_tx, m_tx))
            .await
            .unwrap();
    }

    fn app(
        cs_tx: broadcast::Sender<ScrcpyControlMsg>,
        d_tx: UnboundedSender<ControllerCommand>,
        m_tx: Sender<MaskCommand>,
    ) -> Router {
        Router::new()
            .fallback_service(ServeDir::new(relate_to_root_path(["assets", "web"])))
            .nest("/api/device", device::routers(cs_tx, d_tx))
            .nest("/api/mask", mask::routers(m_tx))
    }
}

#[derive(Serialize)]
pub struct JsonResponse {
    pub code: u16,
    pub message: String,
    pub data: Option<Value>,
}

impl JsonResponse {
    pub fn new(code: u16, message: impl Into<String>, data: Option<Value>) -> Self {
        Self {
            code,
            message: message.into(),
            data,
        }
    }

    pub fn success(message: impl Into<String>, data: Option<Value>) -> Self {
        Self::new(200, message, data)
    }

    pub fn internal_error(message: impl Into<String>, data: Option<Value>) -> Self {
        Self::new(500, message, data)
    }
    pub fn bad_request(message: impl Into<String>, data: Option<Value>) -> Self {
        Self::new(400, message, data)
    }
}

impl IntoResponse for JsonResponse {
    fn into_response(self) -> Response {
        (StatusCode::from_u16(self.code).unwrap(), Json(self)).into_response()
    }
}

#[derive(Debug)]
pub struct WebServerError(u16, String);

impl WebServerError {
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self(500, message.into())
    }
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self(400, message.into())
    }
}

impl IntoResponse for WebServerError {
    fn into_response(self) -> Response {
        let res = JsonResponse {
            code: self.0,
            message: self.1,
            data: None,
        };
        log::error!("[WebServe] Response Error({}): {}", res.code, res.message);
        (StatusCode::from_u16(res.code).unwrap(), Json(res)).into_response()
    }
}
