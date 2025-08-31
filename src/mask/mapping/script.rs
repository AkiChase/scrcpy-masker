use bevy_ineffable::prelude::{ContinuousBinding, InputBinding};
use serde::{Deserialize, Serialize};

use crate::mask::mapping::{
    binding::{ButtonBinding, ValidateMappingConfig},
    utils::Position,
};

// ControlMsgHelper::send_touch(
//     &cs_tx_res.0,
//     MotionEventAction::Down,
//     mapping.pointer_id,
//     original_size,
//     mapping.position.into(),
// );

// tap，swipe,  进入和退出raw input模式, 发送按键、粘贴文本、获取鼠标坐标
// 直接在system中刷新吧，handle和trigger均是首个执行前刷新

// pub fn register_dynamic_functions(
//     engine: &mut ScriptEngine,
//     cs_tx_res: &ChannelSenderCS,
//     mask_size: &MaskSize,
// ) {
//     let tap = {
//         let cs_tx_clone = cs_tx_res.0.clone();
//         let mask_size_clone = mask_size.0;

//         move |action: String,
//               pointer_id: u64,
//               mask_x: f32,
//               mask_y: f32|
//               -> Result<(), Box<EvalAltResult>> {
//             let action = match action.as_str() {
//                 "up" => MotionEventAction::Up,
//                 "down" => MotionEventAction::Down,
//                 "move" => MotionEventAction::Move,
//                 _ => return Err("Tap action must be one of: up, down, move".into()),
//             };

//             ControlMsgHelper::send_touch(
//                 &cs_tx_clone,
//                 action, // 用匹配出来的 action
//                 pointer_id,
//                 mask_size_clone,
//                 (mask_x, mask_y).into(),
//             );

//             Ok(())
//         }
//     };
//     engine.0.register_fn("tap", tap);
// }

#[derive(Debug, Clone)]
pub struct BindMappingScript {
    pub position: Position,
    pub note: String,
    pub activated_script: String,
    pub deactivated_script: String,
    pub activating_script: String,
    pub interval: u64,
    pub bind: ButtonBinding,
    pub input_binding: InputBinding,
}

impl From<MappingScript> for BindMappingScript {
    fn from(value: MappingScript) -> Self {
        Self {
            position: value.position,
            note: value.note,
            activated_script: value.activated_script,
            deactivated_script: value.deactivated_script,
            activating_script: value.activating_script,
            interval: value.interval,
            bind: value.bind.clone(),
            input_binding: ContinuousBinding::hold(value.bind).0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingScript {
    pub position: Position,
    pub note: String,
    pub activated_script: String,
    pub deactivated_script: String,
    pub activating_script: String,
    pub interval: u64,
    pub bind: ButtonBinding,
}

// TODO 验证语法是否错误
impl ValidateMappingConfig for MappingScript {}
