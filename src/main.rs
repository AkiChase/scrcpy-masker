use std::{env, fs::File, net::SocketAddrV4, process::exit, sync::OnceLock};

use bevy::{
    log::{BoxedLayer, LogPlugin, tracing_subscriber::Layer},
    prelude::*,
    window::{CompositeAlphaMode, PresentMode},
};
use scrcpy_masker::{
    config::LocalConfig,
    mask::{MaskPlugins, mask_command::MaskCommand},
    scrcpy::{
        control_msg::ScrcpyControlMsg,
        controller::{self, ControllerCommand},
    },
    update,
    utils::{
        ChannelReceiverA, ChannelReceiverM, ChannelReceiverV, ChannelSenderCS, relate_to_root_path,
    },
    web::{self, ws::WebSocketNotification},
};
use tokio::sync::{broadcast, mpsc, oneshot};
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

fn main() {
    let default_language = "en-US";
    rust_i18n::set_locale(default_language);

    let args: Vec<String> = env::args().collect();
    if args.contains(&"--self_update".to_string()) {
        update::self_update();
        exit(0);
    }

    if let Err(e) = LocalConfig::load() {
        println!("LocalConfig load failed. {}", e);
    }
    // update language
    let language = LocalConfig::get().language;
    match language.as_str() {
        "zh-CN" | "en-US" => rust_i18n::set_locale(&language),
        _ => {
            rust_i18n::set_locale(default_language);
            LocalConfig::set_language(default_language.to_string());
        }
    }
    // update config file
    LocalConfig::save().unwrap();

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    custom_layer: log_custom_layer,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        has_shadow: false,
                        transparent: true,
                        decorations: false,
                        present_mode: PresentMode::AutoVsync,
                        resizable: false,
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
        .add_plugins(bevy_tokio_tasks::TokioTasksPlugin::default())
        .add_plugins(MaskPlugins)
        .add_systems(Startup, start_servers)
        .run();
}

fn start_servers(mut commands: Commands) {
    let config = LocalConfig::get();
    let web_addr: SocketAddrV4 = format!("127.0.0.1:{}", config.web_port).parse().unwrap();
    let controller_addr: SocketAddrV4 = format!("127.0.0.1:{}", config.controller_port)
        .parse()
        .unwrap();

    let (cs_tx, _) = broadcast::channel::<ScrcpyControlMsg>(1000);
    let (ws_tx, _) = broadcast::channel::<WebSocketNotification>(1000);
    let (v_tx, v_rx) = flume::unbounded::<Vec<u8>>();
    let (a_tx, a_rx) = flume::unbounded::<Vec<u8>>();
    let (m_tx, m_rx) = flume::unbounded::<(MaskCommand, oneshot::Sender<Result<String, String>>)>();
    let (d_tx, d_rx) = mpsc::unbounded_channel::<ControllerCommand>();

    commands.insert_resource(ChannelSenderCS(cs_tx.clone()));
    commands.insert_resource(ChannelReceiverV(v_rx));
    commands.insert_resource(ChannelReceiverA(a_rx));
    commands.insert_resource(ChannelReceiverM(m_rx));
    web::Server::start(web_addr, cs_tx.clone(), d_tx, m_tx.clone(), ws_tx.clone());
    controller::Controller::start(controller_addr, cs_tx, v_tx, a_tx, d_rx, m_tx, ws_tx);
}

// TODO 构建跨平台压缩包即可，设置命令行启动方式(bat、sh)
