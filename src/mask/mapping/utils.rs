use flume::Sender;
use serde::{Deserialize, Serialize};

use crate::scrcpy::{
    constant::{MotionEventAction, MotionEventButtons},
    control_msg::ScrcpyControlMsg,
};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Size {
    pub width: u32,
    pub height: u32,
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

pub struct ControlMsgHelper;

impl ControlMsgHelper {
    pub fn send_touch(
        cs_tx: &Sender<ScrcpyControlMsg>,
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
