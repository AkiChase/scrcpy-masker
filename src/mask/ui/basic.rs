/// Bevy basic plugin that creates a window with a transparent background and a border.
use bevy::prelude::*;

pub struct BasicPlugin;

impl Plugin for BasicPlugin {
    fn build(&self, app: &mut App) {
        app
            // ClearColor resource: The color used to clear the screen at the beginning of each frame
            .insert_resource(ClearColor(Color::NONE))
            .add_systems(Startup, (setup, border.after(setup)));
    }
}

fn setup(mut commands: Commands, mut window: Single<&mut Window>) {
    window.resolution.set(800., 600.);
    commands.spawn(Camera2d);
}

fn border(mut commands: Commands) {
    let border_thickness = 1.0; // logical size
    let border_color = Color::linear_rgba(0.0, 0.0, 0.0, 255.0);

    commands.spawn((
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            border: UiRect::all(Val::Px(border_thickness)),
            box_sizing: BoxSizing::BorderBox,
            ..default()
        },
        BackgroundColor(Color::NONE),
        BorderColor(border_color),
    ));
}
