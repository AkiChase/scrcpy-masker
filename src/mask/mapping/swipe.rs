use std::time::Duration;

use bevy::{
    ecs::system::{Res, ResMut},
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingSwipe {
    pub note: String,
    pub pointer_id: u64,
    pub positions: Vec<Position>,
    pub interval: u64,
    pub bind: InputBinding,
}

impl MappingSwipe {
    pub fn new(
        note: &str,
        pointer_id: u64,
        positions: Vec<Position>,
        interval: u64,
        bind: InputBinding,
    ) -> Result<Self, String> {
        // check binding
        if let InputBinding::Pulse(_) = bind {
            Ok(Self {
                note: note.to_string(),
                pointer_id,
                positions,
                interval,
                bind,
            })
        } else {
            Err("Swipe's binding must be Pulse".to_string())
        }
    }
}

const MIN_INTERVAL: u64 = 50; // 50ms is the minimum interval

pub fn handle_swipe(
    ineffable: Res<Ineffable>,
    active_mapping: Res<ActiveMappingConfig>,
    cs_tx_res: Res<ChannelSenderCS>,
    runtime: ResMut<TokioTasksRuntime>,
) {
    if let Some(active_mapping) = &active_mapping.0 {
        for (action, mapping) in &active_mapping.mappings {
            if action.as_ref().starts_with("Swipe") {
                let mapping = mapping.as_ref_swipe();
                let original_size: Vec2 = active_mapping.original_size.into();
                if ineffable.just_pulsed(action.ineff_pulse()) {
                    let cs_tx = cs_tx_res.0.clone();
                    let pointer_id = mapping.pointer_id;
                    let potions = mapping.positions.clone();
                    let interval = mapping.interval;
                    runtime.spawn_background_task(move |_ctx| async move {
                        ControlMsgHelper::send_touch(
                            &cs_tx,
                            MotionEventAction::Down,
                            pointer_id,
                            original_size,
                            potions[0].into(),
                        );
                        let mut cur_pos: Vec2 = potions[0].into();
                        for i in 1..potions.len() {
                            let next_pos: Vec2 = potions[i].into();

                            let delta = next_pos - cur_pos;
                            let steps = std::cmp::max(1, interval / MIN_INTERVAL);
                            let step_duration = interval / steps;

                            for step in 1..=steps {
                                let linear_t = step as f32 / steps as f32;
                                let eased_t = ease_sigmoid_like(linear_t);

                                let interp_x = cur_pos.x + eased_t * delta.x;
                                let interp_y = cur_pos.y + eased_t * delta.y;
                                ControlMsgHelper::send_touch(
                                    &cs_tx,
                                    MotionEventAction::Move,
                                    pointer_id,
                                    original_size,
                                    (interp_x, interp_y).into(),
                                );
                                sleep(Duration::from_millis(step_duration as u64)).await;
                            }

                            cur_pos = next_pos;
                        }
                        ControlMsgHelper::send_touch(
                            &cs_tx,
                            MotionEventAction::Up,
                            pointer_id,
                            original_size,
                            cur_pos.into(),
                        );
                    });
                }
            }
        }
    }
}
