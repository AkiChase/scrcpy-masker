use bevy::prelude::*;
use bevy_ineffable::{config::InputConfig, prelude::IneffableCommands};

use crate::{
    mask::mapping::{
        MappingState,
        config::{ActiveMappingConfig, MappingConfig},
        cursor::CursorState,
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
pub struct MaskSize(pub Vec2);

pub fn handle_mask_command(
    m_rx: Res<ChannelReceiverM>,
    mut window: Single<&mut Window>,
    mut next_mapping_state: ResMut<NextState<MappingState>>,
    mut next_cursor_state: ResMut<NextState<CursorState>>,
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
                window.position.set((left, top).into());

                mask_size.0 = window.resolution.size();

                let msg = format!(
                    "Window moved to ({},{}) and resize to {}x{}",
                    left, top, mask_size.0.x, mask_size.0.y
                );

                log::info!("[Mask] {}", msg);
                oneshot_tx.send(Ok(msg)).unwrap();
            }
            MaskCommand::DeviceConnectionChange { connect } => {
                let msg = if connect {
                    next_mapping_state.set(MappingState::Normal);
                    window.visible = true;
                    format!("main device connection connected")
                } else {
                    next_cursor_state.set(CursorState::Normal);
                    next_mapping_state.set(MappingState::Stop);
                    window.visible = false;
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
                    active_mapping.0 = Some(mapping);
                    oneshot_tx
                        .send(Ok("Successfully change active mapping".to_string()))
                        .unwrap();
                }
            }
        }
    }
}
