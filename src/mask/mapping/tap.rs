use bevy::ecs::system::Res;
use bevy_ineffable::prelude::*;
use serde::{Deserialize, Serialize};

use crate::mask::mapping::{config::ActiveMappingConfig, utils::Position};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingSingleTap {
    pub position: Position,
    pub note: String,
    pub pointer_id: u8,
    pub duration: u32,
    pub sync: bool,
    pub bind: InputBinding,
}

impl MappingSingleTap {
    pub fn new(
        position: Position,
        note: &str,
        pointer_id: u8,
        duration: u32,
        sync: bool,
        bind: InputBinding,
    ) -> Result<Self, String> {
        // check binding
        if let InputBinding::Continuous(_) = bind {
            Ok(Self {
                position,
                note: note.to_string(),
                pointer_id,
                duration,
                sync,
                bind,
            })
        } else {
            Err("SingleTap's binding must be Continuous".to_string())
        }
    }

    pub fn tap_up(&self) {}

    pub fn tap_down(&self) {}
}

pub fn handle_single_tap(
    ineffable: Res<Ineffable>,
    active_mapping_config: Option<Res<ActiveMappingConfig>>,
) {
    if let Some(active_mapping_config) = active_mapping_config {
        for (action, mapping) in &active_mapping_config.0.mappings {
            if action.as_ref().starts_with("SingleTap") {
                let mapping = mapping.as_ref_singletap();
                if ineffable.just_activated(action.ineff_continuous()) {
                    if mapping.sync {
                        log::warn!(
                            "{} | Tap down sync: {:?}",
                            action.as_ref(),
                            mapping.position
                        );
                    } else {
                        log::warn!(
                            "{} | Tap down: {:?}, wait {} ms, and Tap up", // waiting is handled by tokio task
                            action.as_ref(),
                            mapping.position,
                            mapping.duration
                        );
                    }
                } else if mapping.sync && ineffable.just_deactivated(action.ineff_continuous()) {
                    log::warn!("{} | Tap up sync: {:?}", action.as_ref(), mapping.position);
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingRepeatTap {
    pub position: Position,
    pub note: String,
    pub pointer_id: u8,
    pub duration: u32,
    pub interval: u32,
    pub bind: InputBinding,
}

impl MappingRepeatTap {
    pub fn new(
        position: Position,
        note: &str,
        pointer_id: u8,
        duration: u32,
        interval: u32,
        bind: InputBinding,
    ) -> Result<Self, String> {
        // check binding
        if let InputBinding::Continuous(_) = bind {
            Ok(Self {
                position,
                note: note.to_string(),
                pointer_id,
                duration,
                interval,
                bind,
            })
        } else {
            Err("RepeatTap's binding must be Continuous".to_string())
        }
    }
}

pub fn handle_repeat_tap(
    ineffable: Res<Ineffable>,
    active_mapping_config: Option<Res<ActiveMappingConfig>>,
) {
    if let Some(active_mapping_config) = active_mapping_config {
        for (action, mapping) in &active_mapping_config.0.mappings {
            if action.as_ref().starts_with("RepeatTap") {
                let _mapping = mapping.as_ref_repeattap();
                if ineffable.just_activated(action.ineff_continuous()) {
                    // TODO maybe we need a timmer res here, and we should also add one for handle_single_tap
                } else if ineffable.just_deactivated(action.ineff_continuous()) {
                }
            }
        }
    }
}

// TODO MultipleTap
