use std::{env, fs::File, process::exit, sync::OnceLock};

use bevy::{
    log::{BoxedLayer, LogPlugin, tracing_subscriber::Layer},
    prelude::*,
    window::{CompositeAlphaMode, PresentMode},
};
use scrcpy_masker::{mapping, server::Server, ui, update, utils::relate_to_root_path};
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
        .add_systems(Startup, Server::start_default) // TODO move to ui button event, where we need to set adb path and check adb available and finally start server
        .run();
}
