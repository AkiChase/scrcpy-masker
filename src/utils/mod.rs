pub mod share;

use std::{
    env,
    path::{Path, PathBuf},
};

use bevy::ecs::resource::Resource;
use flume::{Receiver, Sender};

pub fn relate_to_root_path<P>(segments: P) -> PathBuf
where
    P: IntoIterator,
    P::Item: AsRef<Path>,
{
    let root = get_base_root();
    segments.into_iter().fold(root, |acc, seg| acc.join(seg))
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
pub struct ChannelSender<T>(pub Sender<T>);

#[derive(Resource)]
pub struct ChannelReceiver<T>(pub Receiver<T>);
