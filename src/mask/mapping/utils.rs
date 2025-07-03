use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::scrcpy::{
    constant::{MotionEventAction, MotionEventButtons},
    control_msg::ScrcpyControlMsg,
};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl Size {
    pub fn into_u32_pair(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

impl From<(u32, u32)> for Size {
    fn from((width, height): (u32, u32)) -> Self {
        Size { width, height }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl From<(i32, i32)> for Position {
    fn from((x, y): (i32, i32)) -> Self {
        Position { x, y }
    }
}

impl Position {
    pub fn into_f32_pair(&self) -> (f32, f32) {
        (self.x as f32, self.y as f32)
    }

    pub fn sub(&self, other: &Position) -> Position {
        Position {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

pub struct ControlMsgHelper;

impl ControlMsgHelper {
    pub fn send_touch(
        cs_tx: &broadcast::Sender<ScrcpyControlMsg>,
        action: MotionEventAction,
        pointer_id: u64,
        mask_size: (u32, u32),
        mask_pos: (f32, f32),
    ) {
        cs_tx
            .send(ScrcpyControlMsg::InjectTouchEvent {
                action,
                pointer_id,
                x: mask_pos.0 as i32,
                y: mask_pos.1 as i32,
                w: mask_size.0 as u16,
                h: mask_size.1 as u16,
                pressure: half::f16::from_f32_const(1.0),
                action_button: MotionEventButtons::PRIMARY,
                buttons: MotionEventButtons::PRIMARY,
            })
            .unwrap();
    }
}


pub fn ease_sigmoid_like(t: f32) -> f32 {
    1.0 / (1.0 + (-12.0 * (t - 0.5)).exp())
}