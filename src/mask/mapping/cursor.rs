use bevy::{input::mouse::MouseMotion, prelude::*};

use crate::{
    mask::{
        mapping::{MappingState, utils::ControlMsgHelper},
        mask_command::MaskSize,
    },
    scrcpy::constant::MotionEventAction,
    utils::ChannelSenderCS,
};

#[derive(States, Clone, Copy, Default, Eq, PartialEq, Hash, Debug)]
pub enum CursorState {
    #[default]
    Normal,
    Bounded,
    Fire,
}

#[derive(Resource)]
pub struct CursorPosition(Vec2);

pub struct CursorPlugins;

impl Plugin for CursorPlugins {
    fn build(&self, app: &mut App) {
        app.insert_resource(CursorPosition([0., 0.].into()))
            .insert_resource(MouseLeftClickDown(false))
            .add_systems(
                Update,
                (
                    cursor_position_normal.run_if(in_state(CursorState::Normal)),
                    cursor_position_bounded.run_if(in_state(CursorState::Bounded)),
                    cursor_position_fire.run_if(in_state(CursorState::Fire)),
                    handle_left_click.run_if(not(in_state(CursorState::Fire))),
                )
                    .run_if(in_state(MappingState::Normal)),
            );
    }
}

// TODO 不同模式
// 1. 正常模式：正常鼠标，基于window来更新当前鼠标坐标，None（离开窗口）时不更新
// 2. 有界模式：可视的虚拟鼠标，移动虚拟鼠标到中心，基于MouseMotion更新虚拟鼠标坐标，到达窗口边缘时，表现为无法移出窗口
// 3. 开火模式：隐形的虚拟鼠标，移动虚拟鼠标到中心，基于MouseMotion更新虚拟鼠标坐标，并持续发送move事件，并在达到边缘时抬起，在中心时再放下

fn cursor_position_normal(window: Single<&mut Window>, mut cursor_pos: ResMut<CursorPosition>) {
    if let Some(pos) = window.cursor_position() {
        cursor_pos.0 = pos;
    }
}

fn cursor_position_bounded(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut cursor_pos: ResMut<CursorPosition>,
) {
    for event in mouse_motion_events.read() {
        cursor_pos.0 += event.delta;
        println!("{:?}", cursor_pos.0);
    }
}
fn cursor_position_fire(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut cursor_pos: ResMut<CursorPosition>,
) {
    for event in mouse_motion_events.read() {
        cursor_pos.0 += event.delta;
        println!("{:?}", cursor_pos.0);
    }
}

#[derive(Resource)]
struct MouseLeftClickDown(bool);

fn handle_left_click(
    mut click_down: ResMut<MouseLeftClickDown>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    cursor_pos: Res<CursorPosition>,
    cs_tx_res: Res<ChannelSenderCS>,
    mask_size: Res<MaskSize>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        click_down.0 = true;
        ControlMsgHelper::send_touch(
            &cs_tx_res.0,
            MotionEventAction::Down,
            0,
            mask_size.into_u32_pair(),
            cursor_pos.0.into(),
        );
    }

    if click_down.0 && mouse_button_input.pressed(MouseButton::Left) {
        ControlMsgHelper::send_touch(
            &cs_tx_res.0,
            MotionEventAction::Move,
            0,
            mask_size.into_u32_pair(),
            cursor_pos.0.into(),
        );
    }

    if mouse_button_input.just_released(MouseButton::Left) && click_down.0 {
        click_down.0 = false;
        ControlMsgHelper::send_touch(
            &cs_tx_res.0,
            MotionEventAction::Up,
            0,
            mask_size.into_u32_pair(),
            cursor_pos.0.into(),
        );
    }
}
