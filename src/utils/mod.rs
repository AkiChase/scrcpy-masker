pub mod share;

use std::{
    env,
    path::{Path, PathBuf},
};

use bevy::ecs::resource::Resource;
use flume::{Receiver, Sender};
use tokio::sync::{broadcast, oneshot};

use crate::{
    config::LocalConfig, mask::mask_command::MaskCommand, scrcpy::control_msg::ScrcpyControlMsg,
};

pub fn relate_to_root_path<P>(segments: P) -> PathBuf
where
    P: IntoIterator,
    P::Item: AsRef<Path>,
{
    let root = get_base_root();
    segments.into_iter().fold(root, |acc, seg| acc.join(seg))
}

const ILLEGAL_CHARS: [char; 9] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

pub fn is_safe_file_name(name: &str) -> bool {
    !name.contains("..")
        && !name.contains('/')
        && !name.contains('\\')
        && !name.contains('\0')
        && !name.contains("..")
        && !name.chars().any(|c| ILLEGAL_CHARS.contains(&c))
        && Path::new(name).file_name().is_some()
}

fn get_base_root() -> PathBuf {
    #[cfg(debug_assertions)]
    {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }
    #[cfg(not(debug_assertions))]
    {
        env::current_exe()
            .expect("Cannot get current executable path")
            .parent()
            .expect("No parent directory for executable")
            .to_path_buf()
    }
}

#[derive(Resource)]
pub struct ChannelSenderCS(pub broadcast::Sender<ScrcpyControlMsg>);

#[derive(Resource)]
pub struct ChannelReceiverV(pub Receiver<Vec<u8>>);

#[derive(Resource)]
pub struct ChannelReceiverA(pub Receiver<Vec<u8>>);

#[derive(Resource)]
pub struct ChannelReceiverM(pub Receiver<(MaskCommand, oneshot::Sender<Result<String, String>>)>);

pub async fn mask_win_move_helper(
    device_w: u32,
    device_h: u32,
    m_tx: &Sender<(MaskCommand, oneshot::Sender<Result<String, String>>)>,
) -> String {
    let config = LocalConfig::get();
    let (left, top, right, bottom) = {
        if device_w >= device_h {
            // horizontal
            let left = config.horizontal_position.0;
            let top = config.horizontal_position.1;
            let mask_w = config.horizontal_mask_width;
            let mask_h = ((device_h as f32) * (mask_w as f32) / (device_w as f32)).round() as u32;
            (left, top, left + mask_w as i32, top + mask_h as i32)
        } else {
            // vertical
            let left = config.vertical_position.0;
            let top = config.vertical_position.1;
            let mask_h = config.vertical_mask_height;
            let mask_w = ((device_w as f32) * (mask_h as f32) / (device_h as f32)).round() as u32;
            (left, top, left + mask_w as i32, top + mask_h as i32)
        }
    };
    let (oneshot_tx, oneshot_rx) = oneshot::channel::<Result<String, String>>();
    m_tx.send_async((
        MaskCommand::WinMove {
            left,
            top,
            right,
            bottom,
        },
        oneshot_tx,
    ))
    .await
    .unwrap();
    oneshot_rx.await.unwrap().unwrap()
}
