use flume::Sender;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::{
        broadcast::{self, error::RecvError},
        mpsc::UnboundedSender,
        watch,
    },
};
use tokio_util::sync::CancellationToken;

use crate::{
    scrcpy::control_msg::{ScrcpyControlMsg, ScrcpyDeviceMsg},
    utils::share::ControlledDevice,
};

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
        mut cs_rx: broadcast::Receiver<ScrcpyControlMsg>,
        mut watch_rx: watch::Receiver<(u32, u32)>,
    ) {
        tokio::select! {
            _ = token.cancelled()=>{
                log::info!("[Controller] Scrcpy control connection reader half cancelled manually");
            }
            _ = async {
                loop {
                    match cs_rx.recv().await {
                        Ok(mut msg) => {
                                // update position with device size
                                match &mut msg {
                                    ScrcpyControlMsg::InjectTouchEvent {
                                        x,
                                        y,
                                        w,
                                        h,
                                        action: _,
                                        pointer_id: _,
                                        pressure: _,
                                        action_button: _,
                                        buttons: _,
                                    } => {
                                        let (device_w, device_h) = watch_rx.borrow_and_update().clone();
                                        let (old_x, old_y) = (*x, *y);
                                        let (old_w, old_h) = (*w, *h);
                                        *x = old_x * device_w as i32 / old_w as i32;
                                        *y = old_y * device_h as i32 / old_h as i32;
                                        *w = device_w as u16;
                                        *h = device_h as u16;
                                    }
                                    ScrcpyControlMsg::InjectScrollEvent {
                                        x,
                                        y,
                                        w,
                                        h,
                                        hscroll: _,
                                        vscroll: _,
                                        buttons: _,
                                    } => {
                                        let (device_w, device_h) = watch_rx.borrow_and_update().clone();
                                        let (old_x, old_y) = (*x, *y);
                                        let (old_w, old_h) = (*w, *h);
                                        *x = old_x * device_w as i32 / old_w as i32;
                                        *y = old_y * device_h as i32 / old_h as i32;
                                        *w = device_w as u16;
                                        *h = device_h as u16;
                                    }
                                    _ => {}
                                };
                                let data:Vec<u8> = msg.into();
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
        cr_tx: UnboundedSender<ScrcpyDeviceMsg>,
        watch_tx: watch::Sender<(u32, u32)>,
        scid: &str,
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
                    ControlledDevice::update_device_name(scid.to_string(), device_name.to_string())
                        .await;
                } else {
                    log::warn!("[Controller] Received invalid UTF-8 device name");
                    ControlledDevice::update_device_name(
                        scid.to_string(),
                        "INVALID_NAME".to_string(),
                    )
                    .await;
                }
            }
        };

        loop {
            match ScrcpyDeviceMsg::read_msg(&mut read_half, scid.to_string()).await {
                Ok(msg) => {
                    if let ScrcpyDeviceMsg::Rotation {
                        rotation: _,
                        width,
                        height,
                        scid,
                    } = msg.clone()
                    {
                        ControlledDevice::update_device_size(scid, (width, height)).await;
                        watch_tx.send((width, height)).unwrap();
                    }

                    // only forward other message from main device
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
        cr_tx: UnboundedSender<ScrcpyDeviceMsg>,
        watch_tx: watch::Sender<(u32, u32)>,
        scid: &str,
        main: bool,
    ) {
        tokio::select! {
            _ = token.cancelled()=>{
                log::info!("[Controller] Scrcpy control connection reader half cancelled manually");
            }
            _ = Self::control_reader_handler(read_half, cr_tx, watch_tx, scid, main)=>{
                log::error!("[Controller] Scrcpy control connection shutdown unexpectedly");
            }
        }
        // no need to shutdown the read_half
    }

    pub async fn handle_control(
        self,
        cs_rx: broadcast::Receiver<ScrcpyControlMsg>,
        cr_tx: UnboundedSender<ScrcpyDeviceMsg>,
        scid: String,
        main: bool,
        token: CancellationToken,
    ) {
        log::debug!("[Controller] handle scrcpy control connection...");
        let (read_half, write_half) = self.socket.into_split();
        let token_copy = token.clone();
        let (watch_tx, watch_rx) = watch::channel::<(u32, u32)>((0, 0)); // share device size with writer
        tokio::join!(
            Self::control_writer(write_half, token, cs_rx, watch_rx),
            Self::control_reader(read_half, token_copy, cr_tx, watch_tx, &scid, main)
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
