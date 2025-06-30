pub mod mapping;
pub mod mask_command;
pub mod ui;

use bevy::app::{App, Plugin, Update};

use crate::mask::mask_command::{MaskSize, handle_mask_command};

pub struct MaskPlugins;

impl Plugin for MaskPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((ui::UiPlugins, mapping::MappingPlugins))
            .insert_resource(MaskSize(0, 0))
            .add_systems(Update, handle_mask_command);
    }
}
