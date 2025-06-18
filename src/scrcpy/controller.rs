use std::{collections::HashMap, net::SocketAddrV4, thread};

use bevy::log;
use copypasta::{ClipboardContext, ClipboardProvider};
use flume::{Receiver, Sender};
use tokio::{
    net::TcpListener,
    sync::{
        broadcast,
        mpsc::{self, UnboundedReceiver},
    },
};
use tokio_util::sync::CancellationToken;

use crate::{
    scrcpy::{
        connection::ScrcpyConnection,
        control_msg::{ControlReceiverMsg, ControlSenderMsg},
    },
    utils::share::ControlledDevice,
};

#[derive(Debug)]
pub enum ControllerCommand {
    // scid: String
    ConnectMainControl(String),
    ConnectMainVideo(String),
    ConnectMainAudio(String),
    ConnectSubControl(String),
    ShutdownMain(String),
    ShutdownSub(String),
}

pub struct Controller;

impl Controller {
    pub fn start(
        addr: SocketAddrV4,
        cs_rx: Receiver<ControlSenderMsg>,
        v_tx: Sender<Vec<u8>>,
        a_tx: Sender<Vec<u8>>,
        d_rx: UnboundedReceiver<ControllerCommand>,
    ) {
        thread::spawn(move || {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async move {
                    Controller::run_server(addr, cs_rx, v_tx, a_tx, d_rx).await;
                });
        });
    }

    async fn cr_msg_handler(mut cr_rx: UnboundedReceiver<ControlReceiverMsg>) {
        loop {
            match cr_rx.recv().await {
                Some(msg) => match msg {
                    ControlReceiverMsg::Clipboard { length: _, text } => {
                        let mut ctx = ClipboardContext::new().unwrap();
                        ctx.set_contents(text).unwrap();
                    }
                    ControlReceiverMsg::AckClipboard { .. } => {}
                    ControlReceiverMsg::UhidOutput { .. } => {}
                    ControlReceiverMsg::Unknown => {
                        log::warn!("[Controller] Unknown message from main device")
                    }
                },
                None => {
                    log::info!("[Controller] C-R channel closed, exiting handler.");
                    break;
                }
            }
        }
    }

    async fn run_server(
        addr: SocketAddrV4,
        cs_rx: Receiver<ControlSenderMsg>,
        v_tx: Sender<Vec<u8>>,
        a_tx: Sender<Vec<u8>>,
        mut d_rx: UnboundedReceiver<ControllerCommand>,
    ) {
        log::info!("[Controller] Starting scrcpy controller on: {}", addr);
        let listener = TcpListener::bind(addr).await.unwrap();

        // scrcpy device msg handler
        let (cr_tx, cr_rx) = mpsc::unbounded_channel::<ControlReceiverMsg>();
        tokio::spawn(async move { Self::cr_msg_handler(cr_rx).await });

        // control msg broadcast
        let (csb_tx, csb_rx) = broadcast::channel::<Vec<u8>>(120);
        tokio::spawn(async move {
            while let Ok(msg) = cs_rx.recv_async().await {
                if let Err(e) = csb_tx.send(msg.into()) {
                    log::error!("[Controller] Failed to boardcasting control message: {}", e);
                }
            }
        });

        // receive command from web server to accept and shutdown scrcpy connection
        log::info!("[Controller] Starting to receive command from web server");
        let mut signal_map: HashMap<String, CancellationToken> = HashMap::new();
        loop {
            match d_rx.recv().await {
                Some(cmd) => match cmd {
                    ControllerCommand::ConnectMainControl(scid) => {
                        let socket_id = "main_control".to_string();

                        if !ControlledDevice::is_scid_controlled(&scid).await {
                            panic!("Controlled device not recorded: {}", scid)
                        }

                        let token = CancellationToken::new();
                        signal_map.insert(socket_id.clone(), token.clone());

                        log::info!("[Controller] Connecting scrcpy main control connection: {}", scid);
                        let csb_rx_copy = csb_rx.resubscribe();
                        let cr_tx_copy = cr_tx.clone();
                        match listener.accept().await {
                            Ok((socket, _)) => {
                                tokio::spawn(async move {
                                    ScrcpyConnection::new(socket)
                                        .handle_control(csb_rx_copy, cr_tx_copy, scid, true, token)
                                        .await;
                                });
                            }
                            Err(e) => {
                                log::error!("[Controller] Error accepting connection: {}", e);
                                ControlledDevice::remove_device(&scid).await;
                                signal_map.remove(&socket_id);
                            }
                        }
                    }
                    ControllerCommand::ConnectMainVideo(scid) => {
                        let socket_id = "main_video".to_string();

                        if !ControlledDevice::is_scid_controlled(&scid).await {
                            panic!("Controlled device not recorded: {}", scid)
                        }

                        let token = CancellationToken::new();
                        signal_map.insert(socket_id.clone(), token.clone());

                        log::info!("[Controller] Connecting scrcpy main video connection: {}", scid);
                        let v_tx_copy = v_tx.clone();
                        match listener.accept().await {
                            Ok((socket, _)) => {
                                tokio::spawn(async move {
                                    ScrcpyConnection::new(socket)
                                        .handle_video(token, v_tx_copy)
                                        .await;
                                });
                            }
                            Err(e) => {
                                log::error!("[Controller] Error accepting connection: {}", e);
                                ControlledDevice::remove_device(&scid).await;
                                signal_map.remove(&socket_id);
                            }
                        }
                    }
                    ControllerCommand::ConnectMainAudio(scid) => {
                        let socket_id = "main_audio".to_string();

                        if !ControlledDevice::is_scid_controlled(&scid).await {
                            panic!("Controlled device not recorded: {}", scid)
                        }

                        let token = CancellationToken::new();
                        signal_map.insert(socket_id.clone(), token.clone());

                        log::info!("[Controller] Connecting scrcpy main audio connection: {}", scid);
                        let a_tx_copy = a_tx.clone();
                        match listener.accept().await {
                            Ok((socket, _)) => {
                                tokio::spawn(async move {
                                    ScrcpyConnection::new(socket)
                                        .handle_audio(token, a_tx_copy)
                                        .await;
                                });
                            }
                            Err(e) => {
                                log::error!("[Controller] Error accepting connection: {}", e);
                                ControlledDevice::remove_device(&scid).await;
                                signal_map.remove(&socket_id);
                            }
                        }
                    }
                    ControllerCommand::ConnectSubControl(scid) => {
                        let socket_id = format!("sub_control_{}", scid);

                        if !ControlledDevice::is_scid_controlled(&scid).await {
                            panic!("Controlled device not recorded: {}", scid)
                        }

                        let token = CancellationToken::new();
                        signal_map.insert(socket_id.clone(), token.clone());

                        log::info!("[Controller] Connecting scrcpy sub control connection: {}", scid);
                        let csb_rx_copy = csb_rx.resubscribe();
                        let cr_tx_copy = cr_tx.clone();
                        match listener.accept().await {
                            Ok((socket, _)) => {
                                tokio::spawn(async move {
                                    ScrcpyConnection::new(socket)
                                        .handle_control(csb_rx_copy, cr_tx_copy, scid, false, token)
                                        .await;
                                });
                            }
                            Err(e) => {
                                log::error!("[Controller] Error accepting connection: {}", e);
                                ControlledDevice::remove_device(&scid).await;
                                signal_map.remove(&socket_id);
                            }
                        }
                    }
                    ControllerCommand::ShutdownMain(scid) => {
                        if !signal_map.contains_key("main_control") {
                            log::warn!("[Controller] Scrcpy main connection not found");
                        } else {
                            log::info!("[Controller] Shutting down scrcpy main connection: {}", scid);
                            for socket_id in ["main_control", "main_video", "main_audio"] {
                                if let Some(token) = signal_map.get(socket_id) {
                                    token.cancel();
                                    signal_map.remove(socket_id);
                                }
                            }
                            for token in signal_map.values() {
                                token.cancel();
                            }
                            signal_map.clear();
                        }
                    }
                    ControllerCommand::ShutdownSub(scid) => {
                        let socket_id = format!("sub_control_{}", scid);
                        if !signal_map.contains_key(&socket_id) {
                            log::warn!("[Controller] Scrcpy sub connection not found: {}", socket_id);
                        } else {
                            log::info!("[Controller] Shutting down scrcpy sub connection: {}", scid);
                            if let Some(token) = signal_map.get(&socket_id) {
                                token.cancel();
                                signal_map.remove(&socket_id);
                            }
                        }
                    }
                },
                None => {
                    log::info!("[Controller] Controller commmand channel closed, exiting handler.");
                    break;
                }
            }
        }
        log::info!("[Controller] Scrcpy controller stopped");
    }
}
