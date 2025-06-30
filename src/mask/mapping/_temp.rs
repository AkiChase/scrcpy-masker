// #[derive(Debug, Clone)]
// pub struct KeySteeringWheel {
//     pub note: String,
//     pub pos_x: f32,
//     pub pos_y: f32,
//     pub pointer_id: u32,
//     pub keys: SteeringKeys,
//     pub offset: f32,
// }

// #[derive(Debug, Clone)]
// pub struct SteeringKeys {
//     pub left: String,
//     pub right: String,
//     pub up: String,
//     pub down: String,
// }

// #[derive(Debug, Clone)]
// pub struct KeyDirectionalSkill {
//     pub note: String,
//     pub pos_x: f32,
//     pub pos_y: f32,
//     pub pointer_id: u32,
//     pub key: String,
//     pub range: f32,
// }

// #[derive(Debug, Clone)]
// pub struct KeyDirectionlessSkill {
//     pub note: String,
//     pub pos_x: f32,
//     pub pos_y: f32,
//     pub pointer_id: u32,
//     pub key: String,
// }

// #[derive(Debug, Clone)]
// pub struct KeyCancelSkill {
//     pub note: String,
//     pub pos_x: f32,
//     pub pos_y: f32,
//     pub pointer_id: u32,
//     pub key: String,
// }

// #[derive(Debug, Clone)]
// pub struct KeyTriggerWhenPressedSkill {
//     pub note: String,
//     pub pos_x: f32,
//     pub pos_y: f32,
//     pub pointer_id: u32,
//     pub key: String,
//     pub directional: bool,
//     pub range_or_time: f32,
// }

// #[derive(Debug, Clone)]
// pub struct KeyTriggerWhenDoublePressedSkill {
//     pub note: String,
//     pub pos_x: f32,
//     pub pos_y: f32,
//     pub pointer_id: u32,
//     pub key: String,
//     pub range: f32,
// }

// #[derive(Debug, Clone)]
// pub struct KeyObservation {
//     pub note: String,
//     pub pos_x: f32,
//     pub pos_y: f32,
//     pub pointer_id: u32,
//     pub key: String,
//     pub scale: f32,
// }

// #[derive(Debug, Clone)]
// pub struct KeyTap {
//     pub note: String,
//     pub pos_x: f32,
//     pub pos_y: f32,
//     pub pointer_id: u32,
//     pub key: String,
//     pub time: f32,
// }

// #[derive(Debug, Clone)]
// pub struct KeySwipe {
//     pub note: String,
//     pub pos_x: f32,
//     pub pos_y: f32,
//     pub pointer_id: u32,
//     pub key: String,
//     pub pos: Vec<(f32, f32)>,
//     pub interval_between_pos: f32,
// }

// #[derive(Debug, Clone)]
// pub struct KeyMacro {
//     pub note: String,
//     pub pos_x: f32,
//     pub pos_y: f32,
//     pub key: String,
//     // TODO ...
// }

// #[derive(Debug, Clone)]
// pub struct KeySight {
//     pub note: String,
//     pub pos_x: f32,
//     pub pos_y: f32,
//     pub key: String,
//     pub pointer_id: u32,
//     pub scale_x: f32,
//     pub scale_y: f32,
// }

// #[derive(Debug, Clone)]
// pub struct KeyFire {
//     pub note: String,
//     pub pos_x: f32,
//     pub pos_y: f32,
//     pub drag: bool,
//     pub pointer_id: u32,
//     pub scale_x: f32,
//     pub scale_y: f32,
// }

// // #[derive(Debug, Clone)]
// // pub enum KeyMapping {
// //     SteeringWheel(KeySteeringWheel),
// //     DirectionalSkill(KeyDirectionalSkill),
// //     DirectionlessSkill(KeyDirectionlessSkill),
// //     CancelSkill(KeyCancelSkill),
// //     TriggerWhenPressedSkill(KeyTriggerWhenPressedSkill),
// //     TriggerWhenDoublePressedSkill(KeyTriggerWhenDoublePressedSkill),
// //     Observation(KeyObservation),
// //     Tap(KeyTap),
// //     Swipe(KeySwipe),
// //     Macro(KeyMacro),
// //     Sight(KeySight),
// //     Fire(KeyFire),
// // }
