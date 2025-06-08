// use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*, window::CursorGrabMode};

// fn lock_cursor_with_virtual_mouse(
//     mut input: ResMut<ButtonInput<KeyCode>>,
//     mut window: Single<&mut Window>,
// ) {
//     if input.clear_just_pressed(KeyCode::Enter) {
//         window.cursor_options.grab_mode = bevy::window::CursorGrabMode::Locked;
//         println!("Locked")
//     }
// }

// TODO lock and unlock mouse
// fn unlock(mut input: ResMut<ButtonInput<KeyCode>>, mut window: Single<&mut Window>) {
//     if input.clear_just_pressed(KeyCode::Escape) {
//         window.cursor_options.grab_mode = bevy::window::CursorGrabMode::None;
//         println!("Unlocked")
//     }
// }

// pub fn lock_cursor(mut window: Single<&mut Window>) {
//     window.cursor_options.grab_mode = CursorGrabMode::Locked;

//     // also hide the cursor
//     // window.cursor.visible = false;
// }

// pub fn print_accumulated_mouse_motion(mouse_move: Res<AccumulatedMouseMotion>) {
//     println!("acc mouse motion: {:?}", mouse_move.delta)
// }

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl From<(u32, u32)> for Size {
    fn from((width, height): (u32, u32)) -> Self {
        Size { width, height }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

impl From<(u32, u32)> for Position {
    fn from((x, y): (u32, u32)) -> Self {
        Position { x, y }
    }
}
