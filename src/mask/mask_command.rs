use bevy::prelude::*;
use bevy_ineffable::prelude::IneffableCommands;

use crate::{
    mask::mapping::{
        MappingState,
        config::{
            ActiveMappingConfig, MappingConfig, load_mapping_config, validate_mapping_config,
        },
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
    GetActiveMapping,
    ValidateMappingConfig {
        config: MappingConfig,
    },
    LoadAndActivateMappingConfig {
        file_name: String,
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
                    log::info!("[Mapping] Enter normal mapping mode");
                    window.visible = true;
                    format!("main device connection connected")
                } else {
                    next_cursor_state.set(CursorState::Normal);
                    next_mapping_state.set(MappingState::Stop);
                    log::info!("[Mapping] Stop mapping");
                    window.visible = false;
                    format!("main device connection disconnected")
                };
                log::info!("[Mask] {}", msg);
                oneshot_tx.send(Ok(msg)).unwrap();
            }
            MaskCommand::GetActiveMapping => {
                oneshot_tx.send(Ok(active_mapping.1.clone())).unwrap();
            }
            MaskCommand::ValidateMappingConfig { config } => {
                match validate_mapping_config(&config) {
                    Ok(_) => {
                        oneshot_tx.send(Ok(String::new())).unwrap();
                    }
                    Err(err) => {
                        oneshot_tx.send(Err(err)).unwrap();
                    }
                }
            }
            MaskCommand::LoadAndActivateMappingConfig { file_name } => {
                log::info!("[Mapping] Load and activate mapping config: {}", file_name);
                match load_mapping_config(&file_name) {
                    Ok((mapping_config, input_config)) => {
                        ineffable.set_config(&input_config);
                        active_mapping.0 = Some(mapping_config);
                        active_mapping.1 = file_name;
                        oneshot_tx.send(Ok(String::new())).unwrap();
                    }
                    Err(e) => {
                        oneshot_tx.send(Err(e)).unwrap();
                    }
                }
            }
        }
    }
}
