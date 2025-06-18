use flume::Sender;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::{
        broadcast::{Receiver, error::RecvError},
        mpsc::UnboundedSender,
    },
};
use tokio_util::sync::CancellationToken;

use crate::{scrcpy::control_msg::ControlReceiverMsg, utils::share::ControlledDevice};

pub struct ScrcpyConnection {
    pub socket: TcpStream,
}

impl ScrcpyConnection {
    pub fn new(socket: TcpStream) -> Self {
        ScrcpyConnection { socket }
    }

    async fn control_writer(
        mut write_half: OwnedWriteHalf,
        token: CancellationToken,
        mut csb_rx: Receiver<Vec<u8>>,
    ) {
        tokio::select! {
            _ = token.cancelled()=>{
                log::info!("[Controller] Scrcpy control connection reader half cancelled manually");
            }
            _ = async {
                loop {
                    match csb_rx.recv().await {
                        Ok(data) => {
                            if let Err(e) = write_half.write_all(&data).await {
                                log::error!("[Controller] Failed to write to socket: {}", e);
                            }
                        }
                        Err(RecvError::Lagged(skipped)) => {
                            log::warn!("[Controller] Receiver lagged, skipped {} messages", skipped);
                        }
                        Err(e) => {
                            log::info!("[Controller] Control channel closed or error occurred: {}", e);
                            break;
                        }
                    }
                }
            }=>{
                log::error!("[Controller] Scrcpy control connection shutdown unexpectedly");
            }
        }
        write_half.shutdown().await.unwrap_or(())
    }

    async fn control_reader_handler(
        mut read_half: OwnedReadHalf,
        cr_tx: UnboundedSender<ControlReceiverMsg>,
        scid: String,
        main: bool,
    ) {
        // read metadata (device name)
        let mut buf: [u8; 64] = [0; 64];
        match read_half.read(&mut buf).await {
            Err(_e) => {
                log::error!("[Controller] Failed to read metadata");
                return;
            }
            Ok(0) => {
                log::error!("[Controller] Failed to read metadata");
                return;
            }
            Ok(n) => {
                let mut end = n;
                while buf[end - 1] == 0 {
                    end -= 1;
                }
                // update device name
                if let Ok(device_name_raw) = std::str::from_utf8(&buf[..n]) {
                    let device_name = device_name_raw.trim_end_matches(char::from(0));
                    ControlledDevice::update_device_name(scid, device_name.to_string()).await;
                } else {
                    log::warn!("[Controller] Received invalid UTF-8 device name");
                    ControlledDevice::update_device_name(scid, "INVALID_NAME".to_string()).await;
                }
            }
        };

        loop {
            match ControlReceiverMsg::read_msg(&mut read_half).await {
                Ok(msg) => {
                    // only forward message from main device
                    if main {
                        cr_tx.send(msg).unwrap();
                    }
                }
                Err(e) => {
                    log::error!("[Controller] {}", e);
                    break;
                }
            };
        }
    }

    async fn control_reader(
        read_half: OwnedReadHalf,
        token: CancellationToken,
        cr_tx: UnboundedSender<ControlReceiverMsg>,
        scid: String,
        main: bool,
    ) {
        tokio::select! {
            _ = token.cancelled()=>{
                log::info!("[Controller] Scrcpy control connection reader half cancelled manually");
            }
            _ = Self::control_reader_handler(read_half, cr_tx, scid, main)=>{
                log::error!("[Controller] Scrcpy control connection shutdown unexpectedly");
            }
        }
        // no need to shutdown the read_half
    }

    pub async fn handle_control(
        self,
        csb_rx: Receiver<Vec<u8>>,
        cr_tx: UnboundedSender<ControlReceiverMsg>,
        scid: String,
        main: bool,
        token: CancellationToken,
    ) {
        log::debug!("[Controller] handle scrcpy control connection...");
        let (read_half, write_half) = self.socket.into_split();
        let token_copy = token.clone();
        tokio::join!(
            Self::control_writer(write_half, token, csb_rx),
            Self::control_reader(read_half, token_copy, cr_tx, scid, main)
        );
    }

    pub async fn handle_video(&mut self, token: CancellationToken, _v_tx: Sender<Vec<u8>>) {
        // TODO handle scrcpy video connection
        log::debug!("[Controller] handle scrcpy vedio connection...");
        tokio::select! {
            // _ = self.control_loop(csb_rx)=>{
            //     log::error!("Scrcpy vedio connection shutdown unexpectedly");
            // }
            _ = token.cancelled()=>{
                log::info!("[Controller] Scrcpy vedio connection cancelled manually");
            }
        };
        self.socket.shutdown().await.unwrap();
    }

    pub async fn handle_audio(&mut self, token: CancellationToken, _a_tx: Sender<Vec<u8>>) {
        // TODO handle scrcpy audio connection
        log::debug!("[Controller] handle scrcpy audio connection...");
        tokio::select! {
            // _ = self.control_loop(csb_rx)=>{
            //     log::error!("[Controller] Scrcpy audio connection shutdown unexpectedly");
            // }
            _ = token.cancelled()=>{
                log::info!("[Controller] Scrcpy audio connection cancelled manually");
            }
        };
        self.socket.shutdown().await.unwrap();
    }
}
