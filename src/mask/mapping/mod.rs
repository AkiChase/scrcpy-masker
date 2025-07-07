pub mod cast_spell;
pub mod config;
pub mod cursor;
pub mod direction_pad;
pub mod fire;
pub mod observation;
pub mod swipe;
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
        cursor::{CursorPlugins, CursorState},
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
        app.add_plugins((IneffablePlugin, CursorPlugins))
            .insert_state(MappingState::Stop)
            .insert_resource(ActiveMappingConfig(None))
            .register_input_action::<MappingAction>()
            .add_systems(
                Startup,
                (
                    init,
                    tap::tap_init,
                    direction_pad::direction_pad_init,
                    fire::fire_init,
                    cast_spell::cast_spell_init,
                    observation::init_observation,
                ),
            )
            .add_systems(
                Update,
                (
                    tap::handle_single_tap,
                    tap::handle_repeat_tap,
                    tap::handle_repeat_tap_trigger,
                    tap::handle_multiple_tap,
                    swipe::handle_swipe,
                    direction_pad::handle_direction_pad,
                    cast_spell::handle_mouse_cast_spell,
                    cast_spell::handle_mouse_cast_spell_trigger,
                    cast_spell::handle_cancel_cast,
                    cast_spell::handle_pad_cast_spell,
                    cast_spell::handle_pad_cast_spell_trigger,
                    observation::handle_observation,
                    observation::handle_observation_trigger,
                    fire::handle_fps,
                    (fire::handle_fire, fire::handle_fire_trigger)
                        .run_if(in_state(CursorState::Fps)),
                )
                    .run_if(in_state(MappingState::Normal)), // mapping
            );
    }
}

fn init(mut ineffable: IneffableCommands, mut active_mapping: ResMut<ActiveMappingConfig>) {
    let config = LocalConfig::get();

    let (mapping_config, input_config) = match load_mapping_config(&config.active_mapping_file) {
        Ok((mapping_config, input_config)) => {
            log::info!(
                "[Mask] Using mapping config {}: {}",
                config.active_mapping_file,
                mapping_config.title
            );
            (mapping_config, input_config)
        }
        Err(e) => {
            log::error!("{}", e);
            log::info!("[Mask] Using default mapping config");
            let default_mapping = default_mapping_config();
            let config_path = relate_to_root_path(["local", "mapping", "default.ron"]);
            save_mapping_config(&default_mapping, &config_path).unwrap();
            LocalConfig::set_active_mapping_file("default.ron".to_string());
            let input_config: InputConfig = InputConfig::from(&default_mapping);
            (default_mapping, input_config)
        }
    };

    active_mapping.0 = Some(mapping_config);
    ineffable.set_config(&input_config);
}
