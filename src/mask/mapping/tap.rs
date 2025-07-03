use std::{collections::HashMap, time::Duration};

use bevy::{
    ecs::{
        resource::Resource,
        system::{Commands, Res, ResMut},
    },
    time::{Time, Timer, TimerMode},
};
use bevy_ineffable::prelude::*;
use bevy_tokio_tasks::TokioTasksRuntime;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

use crate::{
    mask::{
        mapping::{
            config::ActiveMappingConfig,
            utils::{ControlMsgHelper, Position},
        },
        mask_command::MaskSize,
    },
    scrcpy::constant::MotionEventAction,
    utils::ChannelSenderCS,
};

pub fn tap_init(mut commands: Commands) {
    commands.insert_resource(RepeatTapTimers::default());
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingSingleTap {
    pub position: Position,
    pub note: String,
    pub pointer_id: u64,
    pub duration: u64,
    pub sync: bool,
    pub bind: InputBinding,
}

impl MappingSingleTap {
    pub fn new(
        position: Position,
        note: &str,
        pointer_id: u64,
        duration: u64,
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
}

pub fn handle_single_tap(
    ineffable: Res<Ineffable>,
    active_mapping: Res<ActiveMappingConfig>,
    cs_tx_res: Res<ChannelSenderCS>,
    mask_size: Res<MaskSize>,
    runtime: ResMut<TokioTasksRuntime>,
) {
    if let Some(active_mapping) = &active_mapping.0 {
        for (action, mapping) in &active_mapping.mappings {
            if action.as_ref().starts_with("SingleTap") {
                let mapping = mapping.as_ref_singletap();
                if ineffable.just_activated(action.ineff_continuous()) {
                    if mapping.sync {
                        // Tap down sync
                        ControlMsgHelper::send_touch(
                            &cs_tx_res.0,
                            MotionEventAction::Down,
                            mapping.pointer_id,
                            mask_size.into_u32_pair(),
                            mapping.position.into_f32_pair(),
                        );
                    } else {
                        let cs_tx = cs_tx_res.0.clone();
                        let mask_size_pair = mask_size.into_u32_pair();
                        let pointer_id = mapping.pointer_id;
                        let mask_pos = mapping.position.into_f32_pair();
                        let duration = Duration::from_millis(mapping.duration as u64);
                        // Tap down
                        ControlMsgHelper::send_touch(
                            &cs_tx,
                            MotionEventAction::Down,
                            pointer_id,
                            mask_size_pair,
                            mask_pos,
                        );
                        // wait and Tap up
                        runtime.spawn_background_task(move |_ctx| async move {
                            sleep(duration).await;
                            ControlMsgHelper::send_touch(
                                &cs_tx,
                                MotionEventAction::Up,
                                pointer_id,
                                mask_size_pair,
                                mask_pos,
                            );
                        });
                    }
                } else if mapping.sync && ineffable.just_deactivated(action.ineff_continuous()) {
                    // Tap up sync
                    ControlMsgHelper::send_touch(
                        &cs_tx_res.0,
                        MotionEventAction::Up,
                        mapping.pointer_id,
                        mask_size.into_u32_pair(),
                        mapping.position.into_f32_pair(),
                    );
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingRepeatTap {
    pub position: Position,
    pub note: String,
    pub pointer_id: u64,
    pub duration: u64,
    pub interval: u32,
    pub bind: InputBinding,
}

impl MappingRepeatTap {
    pub fn new(
        position: Position,
        note: &str,
        pointer_id: u64,
        duration: u64,
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

#[derive(Resource, Default)]
pub struct RepeatTapTimers(HashMap<String, RepeatTapTimer>);

struct RepeatTapTimer {
    timer: Timer,
    pointer_id: u64,
    mask_pos: (f32, f32),
    duration: Duration,
}

pub fn handle_repeat_tap_trigger(
    time: Res<Time>,
    mut timers: ResMut<RepeatTapTimers>,
    cs_tx_res: Res<ChannelSenderCS>,
    mask_size: Res<MaskSize>,
    runtime: ResMut<TokioTasksRuntime>,
) {
    for (_, timer) in timers.0.iter_mut() {
        if timer.timer.tick(time.delta()).just_finished() {
            let cs_tx = cs_tx_res.0.clone();
            let mask_size_pair = mask_size.into_u32_pair();
            let pointer_id = timer.pointer_id;
            let mask_pos = timer.mask_pos;
            let duration = timer.duration;
            // Tap down
            ControlMsgHelper::send_touch(
                &cs_tx,
                MotionEventAction::Down,
                pointer_id,
                mask_size_pair,
                mask_pos,
            );
            // wait and Tap up
            runtime.spawn_background_task(move |_ctx| async move {
                sleep(duration).await;
                ControlMsgHelper::send_touch(
                    &cs_tx,
                    MotionEventAction::Up,
                    pointer_id,
                    mask_size_pair,
                    mask_pos,
                );
            });
        }
    }
}

pub fn handle_repeat_tap(
    ineffable: Res<Ineffable>,
    active_mapping: Res<ActiveMappingConfig>,
    mut timers: ResMut<RepeatTapTimers>,
) {
    if let Some(active_mapping) = &active_mapping.0 {
        for (action, mapping) in &active_mapping.mappings {
            if action.as_ref().starts_with("RepeatTap") {
                let mapping = mapping.as_ref_repeattap();
                let key = action.to_string();
                if ineffable.just_activated(action.ineff_continuous()) {
                    let interval = Duration::from_millis(mapping.interval as u64);
                    let mut timer = Timer::new(interval, TimerMode::Repeating);
                    timer.tick(interval);
                    timers.0.insert(
                        key,
                        RepeatTapTimer {
                            timer,
                            pointer_id: mapping.pointer_id,
                            mask_pos: mapping.position.into_f32_pair(),
                            duration: Duration::from_millis(mapping.duration as u64),
                        },
                    );
                } else if ineffable.just_deactivated(action.ineff_continuous()) {
                    timers.0.remove(&key);
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingMultipleTapItem {
    pub position: Position,
    pub duration: u64,
    pub wait: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingMultipleTap {
    pub note: String,
    pub pointer_id: u64,
    pub items: Vec<MappingMultipleTapItem>,
    pub bind: InputBinding,
}

impl MappingMultipleTap {
    pub fn new(
        note: &str,
        pointer_id: u64,
        items: Vec<MappingMultipleTapItem>,
        bind: InputBinding,
    ) -> Result<Self, String> {
        // check binding
        if let InputBinding::Pulse(_) = bind {
            Ok(Self {
                note: note.to_string(),
                pointer_id,
                items,
                bind,
            })
        } else {
            Err("MultipleTap's binding must be Pulse".to_string())
        }
    }
}

pub fn handle_multiple_tap(
    ineffable: Res<Ineffable>,
    active_mapping: Res<ActiveMappingConfig>,
    cs_tx_res: Res<ChannelSenderCS>,
    mask_size: Res<MaskSize>,
    runtime: ResMut<TokioTasksRuntime>,
) {
    if let Some(active_mapping) = &active_mapping.0 {
        for (action, mapping) in &active_mapping.mappings {
            if action.as_ref().starts_with("MultipleTap") {
                let mapping = mapping.as_ref_multipletap();
                if ineffable.just_pulsed(action.ineff_pulse()) {
                    let cs_tx = cs_tx_res.0.clone();
                    let mask_size_pair = mask_size.into_u32_pair();
                    let pointer_id = mapping.pointer_id;
                    let items = mapping.items.clone();
                    runtime.spawn_background_task(move |_ctx| async move {
                        for item in items {
                            sleep(Duration::from_millis(item.wait)).await;
                            ControlMsgHelper::send_touch(
                                &cs_tx,
                                MotionEventAction::Down,
                                pointer_id,
                                mask_size_pair,
                                item.position.into_f32_pair(),
                            );
                            sleep(Duration::from_millis(item.duration)).await;
                            ControlMsgHelper::send_touch(
                                &cs_tx,
                                MotionEventAction::Up,
                                pointer_id,
                                mask_size_pair,
                                item.position.into_f32_pair(),
                            );
                        }
                    });
                }
            }
        }
    }
}
