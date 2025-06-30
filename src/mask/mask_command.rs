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
    for msg in m_rx.0.try_iter() {
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

                log::info!(
                    "[Mask] Window moved to {:?} and resize to {}x{}",
                    window.position,
                    mask_w,
                    mask_h
                )
            }
            MaskCommand::DeviceConnectionChange { connect } => {
                next_state.set(if connect {
                    MappingState::Normal
                } else {
                    MappingState::Stop
                });
                log::info!("[Mask] Device connection changed to {}", connect);
            }
            MaskCommand::ActiveMappingChange { mapping, input } => {
                ineffable.set_config(&input);
                active_mapping.0 = mapping;
            }
        }
    }
}
