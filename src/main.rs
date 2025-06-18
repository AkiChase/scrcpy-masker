use std::{env, fs::File, net::SocketAddrV4, process::exit, sync::OnceLock};

use bevy::{
    log::{BoxedLayer, LogPlugin, tracing_subscriber::Layer},
    prelude::*,
    window::{CompositeAlphaMode, PresentMode},
};
use scrcpy_masker::{
    mapping,
    scrcpy::{
        control_msg::ControlSenderMsg,
        controller::{self, ControllerCommand},
    },
    ui, update,
    utils::{ChannelReceiver, ChannelSender, relate_to_root_path},
    web,
};
use tokio::sync::mpsc;
use tracing_appender::non_blocking::WorkerGuard;

static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

fn log_custom_layer(_app: &mut App) -> Option<BoxedLayer> {
    let file = File::create(relate_to_root_path(["app.log"])).unwrap_or_else(|e| {
        panic!("Failed to create log file: {}", e);
    });
    let (non_blocking, guard) = tracing_appender::non_blocking(file);
    let _ = LOG_GUARD.set(guard);
    Some(
        bevy::log::tracing_subscriber::fmt::layer()
            .with_writer(non_blocking)
            .with_file(false)
            .with_line_number(true)
            .with_ansi(false)
            .boxed(),
    )
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.contains(&"--self_update".to_string()) {
        update::self_update();
        exit(0);
    }

    // TODO 加载配置（包括adb路径等等）

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    custom_layer: log_custom_layer,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        transparent: true,
                        decorations: false,
                        present_mode: PresentMode::AutoVsync,
                        visible: false,
                        #[cfg(target_os = "macos")]
                        composite_alpha_mode: CompositeAlphaMode::PostMultiplied,
                        #[cfg(target_os = "linux")]
                        composite_alpha_mode: CompositeAlphaMode::PreMultiplied,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((ui::UiPlugins, mapping::HotKeyPlugins))
        .add_systems(Startup, start_servers)
        .run();
}

fn start_servers(mut commands: Commands) {
    let web_addr: SocketAddrV4 = "127.0.0.1:27799".parse().unwrap();
    let controller_addr: SocketAddrV4 = "127.0.0.1:27798".parse().unwrap();

    let (cs_tx, cs_rx) = flume::unbounded::<ControlSenderMsg>();
    let (v_tx, v_rx) = flume::unbounded::<Vec<u8>>();
    let (a_tx, a_rx) = flume::unbounded::<Vec<u8>>();

    let (d_tx, d_rx) = mpsc::unbounded_channel::<ControllerCommand>();
    commands.insert_resource(ChannelSender(cs_tx.clone()));
    commands.insert_resource(ChannelReceiver(v_rx));
    commands.insert_resource(ChannelReceiver(a_rx));
    web::Server::start(web_addr, cs_tx, d_tx);
    controller::Controller::start(controller_addr, cs_rx, v_tx, a_tx, d_rx);
}
