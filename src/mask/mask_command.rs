use bevy::prelude::*;
use bevy_ineffable::{config::InputConfig, prelude::IneffableCommands};

use crate::{
    mask::mapping::{
        MappingState,
        config::{ActiveMappingConfig, MappingConfig},
    },
    utils::ChannelReceiverM,
};

#[derive(Debug)]
pub enum MaskCommand {
    WinMove {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    },
    DeviceConnectionChange {
        connect: bool,
    },
    ActiveMappingChange {
        mapping: MappingConfig,
        input: InputConfig,
    },
}

#[derive(Resource)]
pub struct MaskSize(pub u32, pub u32);

pub fn handle_mask_command(
    m_rx: Res<ChannelReceiverM>,
    mut window: Single<&mut Window>,
    mut next_state: ResMut<NextState<MappingState>>,
    mut ineffable: IneffableCommands,
    mut active_mapping: ResMut<ActiveMappingConfig>,
    mut mask_size: ResMut<MaskSize>,
) {
    for (msg, oneshot_tx) in m_rx.0.try_iter() {
        match msg {
            MaskCommand::WinMove {
                left,
                top,
                right,
                bottom,
            } => {
                // logical size and position
                let width = (right - left) as f32;
                let height = (bottom - top) as f32;

                window.resolution.set(width, height);
                window.position.set([left, top].into());

                let new_size = &window.resolution;
                let mask_w = new_size.width().round() as u32;
                let mask_h = new_size.height().round() as u32;

                mask_size.0 = mask_w;
                mask_size.1 = mask_h;

                let msg = format!(
                    "Window moved to ({},{}) and resize to {}x{}",
                    left, top, mask_w, mask_h
                );

                log::info!("[Mask] {}", msg);
                oneshot_tx.send(Ok(msg)).unwrap();
            }
            MaskCommand::DeviceConnectionChange { connect } => {
                let msg = if connect {
                    next_state.set(MappingState::Normal);
                    format!("main device connection connected")
                } else {
                    next_state.set(MappingState::Stop);
                    format!("main device connection disconnected")
                };
                log::info!("[Mask] {}", msg);
                oneshot_tx.send(Ok(msg)).unwrap();
            }
            MaskCommand::ActiveMappingChange { mapping, input } => {
                let report = ineffable.validate(&input);
                if !report.is_empty() {
                    report.dump_to_log();
                    oneshot_tx.send(Err("Key mapping configuration failed validation. Please check the logs for details.".to_string())).unwrap();
                } else {
                    ineffable.set_config(&input);
                    active_mapping.0 = mapping;
                    oneshot_tx
                        .send(Ok("Successfully change active mapping".to_string()))
                        .unwrap();
                }
            }
        }
    }
}
