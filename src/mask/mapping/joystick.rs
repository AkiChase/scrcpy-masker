use std::{collections::HashMap, time::Duration};

use bevy::{
    ecs::{
        resource::Resource,
        system::{Commands, Res, ResMut},
    },
    math::Vec2,
};
use bevy_ineffable::prelude::*;
use bevy_tokio_tasks::TokioTasksRuntime;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

use crate::{
    mask::mapping::{
        config::ActiveMappingConfig,
        utils::{ControlMsgHelper, Position, ease_sigmoid_like},
    },
    scrcpy::constant::MotionEventAction,
    utils::ChannelSenderCS,
};

pub fn joystick_init(mut commands: Commands) {
    commands.insert_resource(JoystickStates::default());
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingJoystick {
    pub note: String,
    pub pointer_id: u64,
    pub position: Position,
    pub initial_duration: u64,
    pub max_offset_x: f32,
    pub max_offset_y: f32,
    pub bind: InputBinding,
}

impl MappingJoystick {
    pub fn new(
        note: &str,
        pointer_id: u64,
        position: Position,
        initial_duration: u64,
        max_offset_x: f32,
        max_offset_y: f32,
        bind: InputBinding,
    ) -> Result<Self, String> {
        // check binding
        if let InputBinding::DualAxis { x: _, y: _ } = bind {
            Ok(Self {
                note: note.to_string(),
                pointer_id,
                position,
                initial_duration,
                max_offset_x,
                max_offset_y,
                bind,
            })
        } else {
            Err("Joystick's binding must be DualAxis".to_string())
        }
    }
}

#[derive(Resource, Default)]
pub struct JoystickStates(HashMap<String, Vec2>);

fn scale_direction_2d_state(d_state: Vec2, mapping: &MappingJoystick) -> Vec2 {
    if d_state.x == 0.0 && d_state.y == 0.0 {
        return d_state;
    }

    let max_x = mapping.max_offset_x;
    let max_y = mapping.max_offset_y;

    let scaled = Vec2 {
        x: d_state.x * max_x,
        y: d_state.y * max_y,
    };

    let ellipse_norm = (scaled.x / max_x).powi(2) + (scaled.y / max_y).powi(2);

    if ellipse_norm > 1.0 {
        let norm = (d_state.x.powi(2) + d_state.y.powi(2)).sqrt();
        let unit = Vec2 {
            x: d_state.x / norm,
            y: d_state.y / norm,
        };
        Vec2 {
            x: unit.x * max_x,
            y: unit.y * max_y,
        }
    } else {
        scaled
    }
}

const MIN_STEP: u64 = 25; // 25 is the minimum step length
const MIN_INTERVAL: u64 = 50; // 50ms is the minimum interval

pub fn handle_joystick(
    ineffable: Res<Ineffable>,
    active_mapping: Res<ActiveMappingConfig>,
    cs_tx_res: Res<ChannelSenderCS>,
    runtime: ResMut<TokioTasksRuntime>,
    mut joystick_states: ResMut<JoystickStates>,
) {
    if let Some(active_mapping) = &active_mapping.0 {
        for (action, mapping) in &active_mapping.mappings {
            if action.as_ref().starts_with("Joystick") {
                let mapping = mapping.as_ref_joystick();
                let key = action.to_string();
                let state = scale_direction_2d_state(
                    ineffable.direction_2d(action.ineff_dual_axis()),
                    mapping,
                );
                if joystick_states.0.contains_key(&key) {
                    let last_state = joystick_states.0.get(&key).unwrap().clone();
                    let position: Vec2 = mapping.position.into();
                    if state.x == 0.0 && state.y == 0.0 {
                        // touch up and remove state
                        ControlMsgHelper::send_touch(
                            &cs_tx_res.0,
                            MotionEventAction::Up,
                            mapping.pointer_id,
                            active_mapping.original_size.into(),
                            position + last_state,
                        );
                        joystick_states.0.remove(&key);
                    } else if state != last_state {
                        // record new state
                        joystick_states.0.insert(key, state);
                        // move to new state
                        let delta = state - last_state;
                        let steps = std::cmp::max(1, f32::max(delta.x, delta.y) as u64 / MIN_STEP);
                        for step in 1..=steps {
                            let linear_t = step as f32 / steps as f32;
                            let interp_x = position.x + last_state.x + delta.x * linear_t;
                            let interp_y = position.y + last_state.y + delta.y * linear_t;
                            ControlMsgHelper::send_touch(
                                &cs_tx_res.0,
                                MotionEventAction::Move,
                                mapping.pointer_id,
                                active_mapping.original_size.into(),
                                (interp_x, interp_y).into(),
                            );
                        }
                    }
                } else if state.x != 0.0 || state.y != 0.0 {
                    // record state
                    joystick_states.0.insert(key, state);
                    // touch down
                    let cs_tx = cs_tx_res.0.clone();
                    let pointer_id = mapping.pointer_id;
                    let original_size: Vec2 = active_mapping.original_size.into();
                    let position: Vec2 = mapping.position.into();
                    ControlMsgHelper::send_touch(
                        &cs_tx,
                        MotionEventAction::Down,
                        pointer_id,
                        original_size,
                        position,
                    );
                    // move to state
                    let delta = state.clone();
                    let steps: u64 = std::cmp::max(1, mapping.initial_duration / MIN_INTERVAL);

                    runtime.spawn_background_task(move |_ctx| async move {
                        for step in 1..=steps {
                            let linear_t = step as f32 / steps as f32;
                            let eased_t = ease_sigmoid_like(linear_t);

                            let interp_x = position.x + eased_t * delta.x;
                            let interp_y = position.y + eased_t * delta.y;
                            ControlMsgHelper::send_touch(
                                &cs_tx,
                                MotionEventAction::Move,
                                pointer_id,
                                original_size,
                                (interp_x, interp_y).into(),
                            );
                            sleep(Duration::from_millis(MIN_INTERVAL)).await;
                        }
                    });
                }
            }
        }
    }
}
