use std::{
    collections::HashMap,
    fs::{File, create_dir_all},
    io::Write,
    path::Path,
};

use bevy::{
    ecs::resource::Resource,
    input::{gamepad::GamepadAxis, keyboard::KeyCode, mouse::MouseButton},
};
use bevy_ineffable::{
    bindings::AnalogInput,
    config::InputConfig,
    phantom::IAWrp,
    prelude::{
        ContinuousBinding, DualAxisBinding, InputAction, InputBinding, PulseBinding,
        SingleAxisBinding,
    },
};
use paste::paste;
use ron::ser::{PrettyConfig, to_string_pretty};
use seq_macro::seq;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display};

use crate::{
    mask::mapping::{
        cast_spell::{
            MappingCancelCast, MappingMouseCastSpell, MappingPadCastSpell, MouseCastReleaseMode,
            PadCastReleaseMode,
        },
        direction_pad::MappingDirectionPad,
        fire::{MappingFire, MappingFps},
        swipe::MappingSwipe,
        tap::{MappingMultipleTap, MappingMultipleTapItem, MappingRepeatTap, MappingSingleTap},
        utils::Size,
    },
    utils::relate_to_root_path,
};

// declare 32 actions for each kind of key mapping
seq!(N in 1..=32 {
    #[derive(InputAction, Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone, AsRefStr, Display)]
    pub enum MappingAction {
        #(
            #[ineffable(continuous)]
            SingleTap~N,
            #[ineffable(continuous)]
            RepeatTap~N,
            #[ineffable(pulse)]
            MultipleTap~N,
            #[ineffable(pulse)]
            Swipe~N,
            #[ineffable(dual_axis)]
            DirectionPad~N,
            #[ineffable(continuous)]
            MouseCastSpell~N,
            #[ineffable(continuous)]
            PadCastSpell~N,
            #[ineffable(dual_axis)]
            PadCastSpellDirection~N,
            #[ineffable(pulse)]
            CancelCast~N,
            #[ineffable(pulse)]
            Fps~N,
            #[ineffable(continuous)]
            Fire~N,
        )*
    }

    impl MappingAction {
        pub fn ineff_continuous(&self) -> IAWrp<MappingAction, bevy_ineffable::phantom::Continuous> {
            match self {
                #(
                    MappingAction::SingleTap~N => self.clone()._singletap~N(),
                    MappingAction::RepeatTap~N => self.clone()._repeattap~N(),
                    MappingAction::MouseCastSpell~N => self.clone()._mousecastspell~N(),
                    MappingAction::PadCastSpell~N => self.clone()._padcastspell~N(),
                    MappingAction::Fire~N => self.clone()._fire~N(),
                )*
                _ => panic!("ineff_continuous called on non-continuous variant"),
            }
        }

        pub fn ineff_pulse(&self) -> IAWrp<MappingAction, bevy_ineffable::phantom::Pulse> {
            match self {
                #(
                    MappingAction::MultipleTap~N => self.clone()._multipletap~N(),
                    MappingAction::Swipe~N => self.clone()._swipe~N(),
                    MappingAction::CancelCast~N => self.clone()._cancelcast~N(),
                    MappingAction::Fps~N => self.clone()._fps~N(),
                )*
                _ => panic!("ineff_pulse called on non-pulse variant"),
            }
        }

        pub fn ineff_dual_axis(&self) -> IAWrp<MappingAction, bevy_ineffable::phantom::DualAxis> {
            match self {
                #(
                    MappingAction::DirectionPad~N => self.clone()._directionpad~N(),
                    MappingAction::PadCastSpellDirection~N => self.clone()._padcastspelldirection~N(),
                )*
                _ => panic!("ineff_dual_axis called on non-dual_axis variant"),
            }
        }
    }
});

macro_rules! impl_mapping_type_methods {
    (
        $enum_name:ident {
            $($variant:ident => $type_name:ident),* $(,)?
        }
    ) => {
        impl $enum_name {
            pub fn get_bind(&self) -> InputBinding {
                match self {
                    $(
                        $enum_name::$variant(inner) => inner.bind.clone(),
                    )*
                }
            }

            $(
                paste! {
                    pub fn [<as_ref_ $variant:lower>](&self) -> &$type_name {
                        match self {
                            $enum_name::$variant(inner) => inner,
                            _ => panic!(concat!("Not a ", stringify!($variant), " mapping")),
                        }
                    }
                }
            )*
        }
    };
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MappingType {
    SingleTap(MappingSingleTap),
    RepeatTap(MappingRepeatTap),
    MultipleTap(MappingMultipleTap),
    Swipe(MappingSwipe),
    DirectionPad(MappingDirectionPad),
    MouseCastSpell(MappingMouseCastSpell),
    PadCastSpell(MappingPadCastSpell),
    CancelCast(MappingCancelCast),
    Fps(MappingFps),
    Fire(MappingFire),
}

impl_mapping_type_methods! {
    MappingType {
        SingleTap => MappingSingleTap,
        RepeatTap => MappingRepeatTap,
        MultipleTap => MappingMultipleTap,
        Swipe => MappingSwipe,
        DirectionPad => MappingDirectionPad,
        MouseCastSpell => MappingMouseCastSpell,
        PadCastSpell => MappingPadCastSpell,
        CancelCast => MappingCancelCast,
        Fps => MappingFps,
        Fire => MappingFire,
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingConfig {
    pub version: String,
    pub title: String,
    pub original_size: Size,
    pub mappings: HashMap<MappingAction, MappingType>,
}

impl From<&MappingConfig> for InputConfig {
    fn from(mapping_config: &MappingConfig) -> Self {
        let mut all_bindings: HashMap<String, Vec<InputBinding>> = HashMap::new();

        for (action, mapping) in &mapping_config.mappings {
            if let MappingType::PadCastSpell(m) = mapping {
                all_bindings.insert(action.to_string(), vec![m.bind.clone()]);
                all_bindings.insert(m.pad_action.to_string(), vec![m.pad_bind.clone()]);
            } else {
                all_bindings.insert(action.to_string(), vec![mapping.get_bind()]);
            }
        }

        let binding_config: HashMap<String, HashMap<String, Vec<InputBinding>>> =
            HashMap::from([("MappingAction".to_string(), all_bindings)]);
        let mut input_config = InputConfig::new();
        input_config.bindings = binding_config;
        input_config
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ActiveMappingConfig(pub Option<MappingConfig>);

pub fn default_mapping_config() -> MappingConfig {
    MappingConfig {
        version: "0.0.1".to_string(),
        title: "Default".to_string(),
        original_size: Size {
            width: 2560,
            height: 1440,
        },
        mappings: HashMap::from([
            (
                MappingAction::SingleTap1,
                MappingType::SingleTap(
                    MappingSingleTap::new(
                        (100, 100).into(),
                        "SingleTap (duration)",
                        1,
                        1000,
                        false,
                        ContinuousBinding::hold(KeyCode::Digit1).0,
                    )
                    .unwrap(),
                ),
            ),
            (
                MappingAction::SingleTap2,
                MappingType::SingleTap(
                    MappingSingleTap::new(
                        (200, 100).into(),
                        "SingleTap (sync)",
                        1,
                        0,
                        true,
                        ContinuousBinding::hold(KeyCode::Digit2).0,
                    )
                    .unwrap(),
                ),
            ),
            (
                MappingAction::RepeatTap1,
                MappingType::RepeatTap(
                    MappingRepeatTap::new(
                        (250, 200).into(),
                        "RepeatTap",
                        1,
                        30,
                        100,
                        ContinuousBinding::hold(KeyCode::Digit3).0,
                    )
                    .unwrap(),
                ),
            ),
            (
                MappingAction::RepeatTap2,
                MappingType::RepeatTap(
                    MappingRepeatTap::new(
                        (250, 250).into(),
                        "RepeatTap (multi-binding)",
                        2,
                        30,
                        100,
                        ContinuousBinding::hold((KeyCode::ControlLeft, KeyCode::Digit3)).0,
                    )
                    .unwrap(),
                ),
            ),
            (
                MappingAction::MultipleTap1,
                MappingType::MultipleTap(
                    MappingMultipleTap::new(
                        "MultipleTap",
                        1,
                        vec![
                            MappingMultipleTapItem {
                                position: (100, 100).into(),
                                duration: 500,
                                wait: 0,
                            },
                            MappingMultipleTapItem {
                                position: (200, 200).into(),
                                duration: 500,
                                wait: 1000,
                            },
                            MappingMultipleTapItem {
                                position: (300, 300).into(),
                                duration: 500,
                                wait: 1000,
                            },
                        ],
                        PulseBinding::just_pressed(KeyCode::Digit4).0,
                    )
                    .unwrap(),
                ),
            ),
            (
                MappingAction::Swipe1,
                MappingType::Swipe(
                    MappingSwipe::new(
                        "Swipe",
                        1,
                        vec![(100, 100).into(), (200, 200).into(), (300, 300).into()],
                        1000,
                        PulseBinding::just_pressed(KeyCode::Digit5).0,
                    )
                    .unwrap(),
                ),
            ),
            (
                MappingAction::DirectionPad1,
                MappingType::DirectionPad(
                    MappingDirectionPad::new(
                        "DirectionPad",
                        9,
                        (300, 1000).into(),
                        100,
                        100.,
                        100.,
                        DualAxisBinding::builder()
                            .set_x(
                                SingleAxisBinding::hold()
                                    .set_negative(KeyCode::KeyA)
                                    .set_positive(KeyCode::KeyD)
                                    .build(),
                            )
                            .set_y(
                                SingleAxisBinding::hold()
                                    .set_negative(KeyCode::KeyW)
                                    .set_positive(KeyCode::KeyS)
                                    .build(),
                            )
                            .build()
                            .0,
                    )
                    .unwrap(),
                ),
            ),
            (
                MappingAction::DirectionPad2,
                MappingType::DirectionPad(
                    MappingDirectionPad::new(
                        "DirectionPad gamepad",
                        9,
                        (300, 1000).into(),
                        300,
                        100.,
                        100.,
                        DualAxisBinding::builder()
                            .set_x(
                                SingleAxisBinding::analog(AnalogInput::GamePad(
                                    GamepadAxis::LeftStickX,
                                ))
                                .set_sensitivity(1.0)
                                .build(),
                            )
                            .set_y(
                                SingleAxisBinding::analog(AnalogInput::GamePad(
                                    GamepadAxis::LeftStickY,
                                ))
                                .set_sensitivity(1.0)
                                .build(),
                            )
                            .build()
                            .0,
                    )
                    .unwrap(),
                ),
            ),
            (
                MappingAction::Fps1,
                MappingType::Fps(
                    MappingFps::new(
                        "FPS",
                        0,
                        (1280, 720).into(),
                        2.,
                        1.,
                        PulseBinding::just_pressed(MouseButton::Right).0,
                    )
                    .unwrap(),
                ),
            ),
            (
                MappingAction::MouseCastSpell1,
                MappingType::MouseCastSpell(
                    MappingMouseCastSpell::new(
                        "MouseCastSpell (no direction)",
                        3,
                        (1900, 1150).into(),
                        (1280, 815).into(),
                        1.,
                        0.7,
                        150.,
                        625.,
                        MouseCastReleaseMode::OnRelease,
                        true,
                        ContinuousBinding::hold(KeyCode::KeyR).0,
                    )
                    .unwrap(),
                ),
            ),
            (
                MappingAction::MouseCastSpell2,
                MappingType::MouseCastSpell(
                    MappingMouseCastSpell::new(
                        "MouseCastSpell (press to release)",
                        3,
                        (1900, 1150).into(),
                        (1280, 815).into(),
                        1.,
                        0.7,
                        150.,
                        625.,
                        MouseCastReleaseMode::OnPress,
                        false,
                        ContinuousBinding::hold(KeyCode::KeyQ).0,
                    )
                    .unwrap(),
                ),
            ),
            (
                MappingAction::MouseCastSpell3,
                MappingType::MouseCastSpell(
                    MappingMouseCastSpell::new(
                        "MouseCastSpell (second press to release)",
                        3,
                        (2100, 1030).into(),
                        (1280, 815).into(),
                        1.,
                        0.7,
                        150.,
                        625.,
                        MouseCastReleaseMode::OnSecondPress,
                        true,
                        ContinuousBinding::hold(MouseButton::Back).0,
                    )
                    .unwrap(),
                ),
            ),
            (
                MappingAction::MouseCastSpell4,
                MappingType::MouseCastSpell(
                    MappingMouseCastSpell::new(
                        "MouseCastSpell",
                        3,
                        (2250, 900).into(),
                        (1280, 815).into(),
                        1.,
                        0.7,
                        150.,
                        625.,
                        MouseCastReleaseMode::OnRelease,
                        false,
                        ContinuousBinding::hold(KeyCode::KeyE).0,
                    )
                    .unwrap(),
                ),
            ),
            (
                MappingAction::PadCastSpell1,
                MappingType::PadCastSpell(
                    MappingPadCastSpell::new(
                        "PadCastSpell",
                        3,
                        (2000, 750).into(),
                        PadCastReleaseMode::OnRelease,
                        150.,
                        true,
                        MappingAction::PadCastSpellDirection1,
                        DualAxisBinding::builder()
                            .set_x(
                                SingleAxisBinding::hold()
                                    .set_negative(KeyCode::KeyA)
                                    .set_positive(KeyCode::KeyD)
                                    .build(),
                            )
                            .set_y(
                                SingleAxisBinding::hold()
                                    .set_negative(KeyCode::KeyW)
                                    .set_positive(KeyCode::KeyS)
                                    .build(),
                            )
                            .build()
                            .0,
                        ContinuousBinding::hold(KeyCode::KeyJ).0,
                    )
                    .unwrap(),
                ),
            ),
            (
                MappingAction::CancelCast1,
                MappingType::CancelCast(
                    MappingCancelCast::new(
                        "CancelCast",
                        (2200, 175).into(),
                        PulseBinding::just_pressed(KeyCode::Space).0,
                    )
                    .unwrap(),
                ),
            ),
            (
                MappingAction::Fps1,
                MappingType::Fps(
                    MappingFps::new(
                        "FPS",
                        0,
                        (1280, 720).into(),
                        2.,
                        1.,
                        PulseBinding::just_pressed(MouseButton::Right).0,
                    )
                    .unwrap(),
                ),
            ),
            (
                MappingAction::Fire1,
                MappingType::Fire(
                    MappingFire::new(
                        "Fire",
                        1,
                        (2000, 1000).into(),
                        1.,
                        0.5,
                        ContinuousBinding::hold(MouseButton::Left).0,
                    )
                    .unwrap(),
                ),
            ),
        ]),
    }
}

pub fn load_mapping_config(
    file_name: impl AsRef<str>,
) -> Result<(MappingConfig, InputConfig), String> {
    // load from file
    let path = relate_to_root_path(["local", "mapping", file_name.as_ref()]);
    let ron_string = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read mapping config file: {}", e))?;
    let mapping_config: MappingConfig = ron::from_str(&ron_string)
        .map_err(|e| format!("Cannot deserialize mapping config: {}", e))?;

    let input_config: InputConfig = InputConfig::from(&mapping_config);

    Ok((mapping_config, input_config))
}

pub fn save_mapping_config(config: &MappingConfig, path: &Path) -> Result<(), String> {
    let pretty = PrettyConfig::default();
    let ron_string = to_string_pretty(config, pretty)
        .map_err(|e| format!("Cannot serialize mapping config: {}", e))?;
    if let Some(parent) = path.parent() {
        create_dir_all(parent)
            .map_err(|e| format!("Cannot create directory for config file: {}", e))?;
    }
    let mut file =
        File::create(path).map_err(|e| format!("Cannot create mapping config file: {}", e))?;
    file.write_all(ron_string.as_bytes())
        .map_err(|e| format!("Cannot write to mapping config file: {}", e))?;
    Ok(())
}
