pub mod mapping;
pub mod mask_command;
pub mod ui;

use bevy::{
    app::{App, Plugin, Startup, Update},
    ecs::system::{Commands, Single},
    window::Window,
};

use crate::mask::mask_command::{MaskSize, handle_mask_command};

pub struct MaskPlugins;

impl Plugin for MaskPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((ui::UiPlugins, mapping::MappingPlugins))
            .add_systems(Startup, init_mask_size)
            .add_systems(Update, handle_mask_command);
    }
}

fn init_mask_size(mut commands: Commands, window: Single<&Window>) {
    commands.insert_resource(MaskSize(window.size()));
}
