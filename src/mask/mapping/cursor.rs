use bevy::{input::mouse::MouseMotion, prelude::*};

#[derive(States, Clone, Copy, Default, Eq, PartialEq, Hash, Debug)]
pub enum CursorState {
    #[default]
    Normal,
    Bounded,
    Fire,
}

#[derive(Resource)]
pub struct CursorPosition(Vec2);

pub struct MappingPlugins;

impl Plugin for MappingPlugins {
    fn build(&self, app: &mut App) {
        app.insert_resource(CursorPosition([0., 0.].into()))
            .add_systems(
                Update,
                (
                    cursor_position_normal.run_if(in_state(CursorState::Normal)),
                    cursor_position_bounded.run_if(in_state(CursorState::Bounded)),
                    cursor_position_fire.run_if(in_state(CursorState::Fire)),
                ),
            )
            .add_systems(Startup, test);
    }
}

// TODO 不同模式
// 1. 正常模式：正常鼠标，基于window来更新当前鼠标坐标，None（离开窗口）时不更新
// 2. 有界模式：可视的虚拟鼠标，移动虚拟鼠标到中心，基于MouseMotion更新虚拟鼠标坐标，到达窗口边缘时，表现为无法移出窗口
// 3. 开火模式：隐形的虚拟鼠标，移动虚拟鼠标到中心，基于MouseMotion更新虚拟鼠标坐标，并持续发送move事件，并在达到边缘时抬起，在中心时再放下

fn test(mut window: Single<&mut Window>, mut cursor_pos: ResMut<CursorPosition>) {
    // window.cursor_options.grab_mode = bevy::window::CursorGrabMode::Locked;
    window.visible = true;
    window.decorations = true;
    let size = window.size();
    cursor_pos.0 = [size.x / 2.0, size.y / 2.0].into();
}

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
