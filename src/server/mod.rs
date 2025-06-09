pub mod adb;
pub mod binary;
pub mod scrcpy;

use std::thread;

use bevy::log;
use tokio::net::TcpListener;

use crate::server::scrcpy::ScrcpyConnection;

pub struct Server;

impl Server {
    pub fn start(port: Option<u16>) {
        let port = port.unwrap_or(27183);
        thread::spawn(move || {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async move {
                    Server::run_server(port).await;
                });
        });
    }

    pub fn start_default() {
        Server::start(None);
    }

    async fn run_server(port: u16) {
        let address = format!("127.0.0.1:{}", port);
        log::info!("Starting server on {}", address);
        let listener = TcpListener::bind(address).await.unwrap();
        log::info!("Listening for scrcpy connections...");
        loop {
            match listener.accept().await {
                Ok((socket, _)) => {
                    tokio::spawn(async move {
                        ScrcpyConnection::new(socket).handle().await;
                    });
                }
                Err(e) => {
                    log::error!("Error accepting connection: {}", e);
                }
            }
        }
    }
}
