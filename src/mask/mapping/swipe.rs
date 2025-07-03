use std::time::Duration;

use bevy::ecs::system::{Res, ResMut};
use bevy_ineffable::prelude::*;
use bevy_tokio_tasks::TokioTasksRuntime;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

use crate::{
    mask::mapping::{
        config::ActiveMappingConfig,
        utils::{ControlMsgHelper, Position},
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

fn ease_sigmoid_like(t: f32) -> f32 {
    1.0 / (1.0 + (-12.0 * (t - 0.5)).exp())
}

pub fn handle_swipe(
    ineffable: Res<Ineffable>,
    active_mapping: Res<ActiveMappingConfig>,
    cs_tx_res: Res<ChannelSenderCS>,
    runtime: ResMut<TokioTasksRuntime>,
) {
    if let Some(active_mapping) = &active_mapping.0 {
        for (action, mapping) in &active_mapping.mappings {
            if action.as_ref().starts_with("Swipe") {
                let original_size_pair = active_mapping.original_size.into_u32_pair();
                let mapping = mapping.as_ref_swipe();
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
                            original_size_pair,
                            potions[0].into_f32_pair(),
                        );
                        let mut cur_pos = &potions[0];
                        let min_interval = 50; // 50ms is the minimum interval
                        for i in 1..potions.len() {
                            let next_pos = &potions[i];

                            let (dx, dy) = (next_pos.sub(cur_pos)).into_f32_pair();
                            let steps = std::cmp::max(1, interval / min_interval);
                            let step_duration = interval / steps;

                            for step in 1..=steps {
                                let linear_t = step as f32 / steps as f32;
                                let eased_t = ease_sigmoid_like(linear_t);

                                let interp_x = cur_pos.x as f32 + eased_t * dx;
                                let interp_y = cur_pos.y as f32 + eased_t * dy;
                                ControlMsgHelper::send_touch(
                                    &cs_tx,
                                    MotionEventAction::Move,
                                    pointer_id,
                                    original_size_pair,
                                    (interp_x, interp_y),
                                );
                                sleep(Duration::from_millis(step_duration as u64)).await;
                            }

                            cur_pos = next_pos;
                        }
                        ControlMsgHelper::send_touch(
                            &cs_tx,
                            MotionEventAction::Up,
                            pointer_id,
                            original_size_pair,
                            cur_pos.into_f32_pair(),
                        );
                    });
                }
            }
        }
    }
}
