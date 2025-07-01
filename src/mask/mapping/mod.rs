pub mod config;
pub mod cursor;
pub mod tap;
pub mod utils;

use bevy::prelude::*;
use bevy_ineffable::prelude::*;

use crate::{
    config::LocalConfig,
    mask::mapping::{
        config::{
            ActiveMappingConfig, MappingAction, default_mapping_config, load_mapping_config,
            save_mapping_config,
        },
        cursor::CursorState,
        tap::{handle_repeat_tap, handle_single_tap},
    },
    utils::relate_to_root_path,
};

#[derive(States, Clone, Copy, Default, Eq, PartialEq, Hash, Debug)]
pub enum MappingState {
    #[default]
    Stop,
    Normal,
    // TODO RawInput, // convert all keys to keycodes
}

pub struct MappingPlugins;

impl Plugin for MappingPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins(IneffablePlugin)
            .insert_state(MappingState::Normal)
            .insert_state(CursorState::Normal)
            .insert_resource(ActiveMappingConfig::default())
            .register_input_action::<MappingAction>()
            .add_systems(Startup, init)
            .add_systems(
                Update,
                (handle_repeat_tap, handle_single_tap)
                    .distributive_run_if(in_state(MappingState::Normal)), // mapping
            );
    }
}

fn init(mut ineffable: IneffableCommands, mut active_mapping: ResMut<ActiveMappingConfig>) {
    let config = LocalConfig::get();

    match load_mapping_config(&config.active_mapping_file) {
        Ok((mapping_config, input_config)) => {
            log::info!(
                "[Mask] Using mapping config {}: {}",
                config.active_mapping_file,
                mapping_config.title
            );
            ineffable.set_config(&input_config);
            return;
        }
        Err(e) => log::error!("{}", e),
    };

    log::info!("[Mask] Using default mapping config");
    let config_path = relate_to_root_path(["local", "mapping", "default.ron"]);
    let default_mapping = default_mapping_config();
    if !config_path.exists() {
        save_mapping_config(&default_mapping, &config_path).unwrap();
    }

    active_mapping.0 = default_mapping;
    LocalConfig::set_active_mapping_file("default.ron".to_string());

    let input_config: InputConfig = InputConfig::from(&active_mapping.0);
    ineffable.set_config(&input_config);
}
