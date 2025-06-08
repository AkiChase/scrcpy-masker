/// Bevy basic plugin that creates a window with a transparent background and a border.

#[cfg(any(target_os = "macos", target_os = "linux"))]
use bevy::window::CompositeAlphaMode;
use bevy::{prelude::*, window::PresentMode};

pub struct BasicPlugin;

impl Plugin for BasicPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
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
        }))
        // ClearColor resource: The color used to clear the screen at the beginning of each frame
        .insert_resource(ClearColor(Color::NONE))
        .add_systems(Startup, (setup, border.after(setup), show.after(border)));
    }
}

fn setup(mut commands: Commands, mut window: Single<&mut Window>) {
    let scale = window.scale_factor();
    window.resolution = (800.0 * scale, 600.0 * scale).into();
    commands.spawn(Camera2d);
}

fn border(mut commands: Commands, window: Single<&mut Window>) {
    let scale = window.scale_factor();
    let logical_width = window.resolution.width() / scale;
    let logical_height = window.resolution.height() / scale;
    let border_thickness = 1.0; // logical size
    let border_color = Color::linear_rgba(0.0, 0.0, 0.0, 255.0);

    commands.spawn((
        Node {
            width: Val::Px(logical_width),
            height: Val::Px(logical_height),
            border: UiRect::all(Val::Px(border_thickness)),
            ..default()
        },
        BackgroundColor(Color::NONE),
        BorderColor(border_color),
    ));
}

fn show(mut window: Single<&mut Window>) {
    window.visible = true;
}
