pub mod config;
pub mod tap;
pub mod utils;

use bevy::prelude::*;
use bevy_ineffable::prelude::*;

use crate::{
    mapping::{
        config::{ActiveMappingConfig, MappingAction, default_mapping_config, save_mapping_config},
        tap::{handle_repeat_tap, handle_single_tap},
    },
    utils::relate_to_root_path,
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
    mut ineffable: IneffableCommands,
    mut next_state: ResMut<NextState<MappingState>>,
) {
    let config_path = relate_to_root_path(["local", "default.ron"]);

    if !config_path.exists() {
        save_mapping_config(&default_mapping_config(), &config_path).unwrap();
    }

    if let Ok((mapping_config, input_config)) = config::load_mapping_config(&config_path) {
        ineffable.set_config(&input_config);
        commands.insert_resource(ActiveMappingConfig(mapping_config));
        next_state.set(MappingState::Normal);
    } else {
        log::error!("Failed to load mapping config")
    }
}

#[derive(Resource, Debug, Clone)]
pub struct HotkeyMap {
    pub bindings: Vec<(String, KeyCode)>,
}

#[derive(Event, Debug, Clone)]
pub struct HotkeyMapChanged(pub HotkeyMap);
