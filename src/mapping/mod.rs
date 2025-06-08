pub mod config;
pub mod tap;
pub mod utils;

use bevy::prelude::*;
use bevy_ineffable::prelude::*;

use crate::mapping::{
    config::{ActiveMappingConfig, MappingAction},
    tap::{handle_repeat_tap, handle_single_tap},
};

#[derive(States, Clone, Copy, Default, Eq, PartialEq, Hash, Debug)]
enum MappingState {
    #[default]
    Stop,
    Normal,
    // RawInput, // convert all keys to keycodes
}

#[derive(States, Clone, Copy, Default, Eq, PartialEq, Hash, Debug)]
enum MouseLockState {
    #[default]
    Free,
    // LockedCenterWithoutMouse,
    // LockedWithVirtualMouse,
}

pub struct HotKeyPlugins;

impl Plugin for HotKeyPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins(IneffablePlugin)
            .insert_state(MappingState::Normal)
            .register_input_action::<MappingAction>()
            .add_systems(Startup, init)
            .add_systems(
                Update,
                (handle_repeat_tap, handle_single_tap)
                    .distributive_run_if(in_state(MappingState::Normal)), // mapping
            );
    }
}

fn init(
    mut commands: Commands,
    ineffable: IneffableCommands,
    mut next_state: ResMut<NextState<MappingState>>,
) {
    let mapping_config = config::load_mapping_config(ineffable);
    commands.insert_resource(ActiveMappingConfig(mapping_config));
    next_state.set(MappingState::Normal);
}

#[derive(Resource, Debug, Clone)]
pub struct HotkeyMap {
    pub bindings: Vec<(String, KeyCode)>,
}

#[derive(Event, Debug, Clone)]
pub struct HotkeyMapChanged(pub HotkeyMap);
