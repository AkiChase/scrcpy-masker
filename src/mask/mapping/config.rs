use std::{
    collections::HashMap,
    fs::{File, create_dir_all},
    io::Write,
    path::Path,
};

use bevy::{
    ecs::resource::Resource,
    input::{gamepad::GamepadAxis, keyboard::KeyCode, mouse::MouseButton},
    math::Vec2,
};
use bevy_ineffable::{
    config::InputConfig,
    phantom::IAWrp,
    prelude::{InputAction, InputBinding},
};
use paste::paste;
use seq_macro::seq;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use strum_macros::{AsRefStr, Display};

use crate::{
    mask::mapping::{
        binding::{ButtonBinding, DirectionBinding},
        cast_spell::{
            MappingCancelCast, MappingMouseCastSpell, MappingPadCastSpell, MouseCastReleaseMode,
            PadCastReleaseMode,
        },
        direction_pad::MappingDirectionPad,
        fire::{MappingFire, MappingFps},
        observation::MappingObservation,
        raw_input::MappingRawInput,
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
            PadCastDirection~N,
            #[ineffable(pulse)]
            CancelCast~N,
            #[ineffable(continuous)]
            Observation~N,
            #[ineffable(pulse)]
            Fps~N,
            #[ineffable(continuous)]
            Fire~N,
            #[ineffable(pulse)]
            RawInput~N,
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
                    MappingAction::Observation~N => self.clone()._observation~N(),
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
                    MappingAction::RawInput~N => self.clone()._rawinput~N(),
                )*
                _ => panic!("ineff_pulse called on non-pulse variant"),
            }
        }

        pub fn ineff_dual_axis(&self) -> IAWrp<MappingAction, bevy_ineffable::phantom::DualAxis> {
            match self {
                #(
                    MappingAction::DirectionPad~N => self.clone()._directionpad~N(),
                    MappingAction::PadCastDirection~N => self.clone()._padcastdirection~N(),
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
            pub fn get_input_binding(&self) -> InputBinding {
                match self {
                    $(
                        $enum_name::$variant(inner) => inner.input_binding.clone(),
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
    Observation(MappingObservation),
    Fps(MappingFps),
    Fire(MappingFire),
    RawInput(MappingRawInput),
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
        Observation => MappingObservation,
        Fps => MappingFps,
        Fire => MappingFire,
        RawInput => MappingRawInput,
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MappingConfig {
    pub version: String,
    pub title: String,
    pub original_size: Size,
    pub mappings: HashMap<MappingAction, MappingType>,
}

impl MappingConfig {
    pub fn get_mapping_label_info(&self) -> Vec<(&MappingType, String, Vec2, Vec2)> {
        let size: Vec2 = self.original_size.into();
        self.mappings
            .iter()
            .map(|(_, mapping)| {
                let (binding, pos): (String, Vec2) = match mapping {
                    MappingType::SingleTap(m) => (m.bind.to_string(), m.position.into()),
                    MappingType::RepeatTap(m) => (m.bind.to_string(), m.position.into()),
                    MappingType::MultipleTap(m) => (m.bind.to_string(), m.items[0].position.into()),
                    MappingType::Swipe(m) => (m.bind.to_string(), m.positions[0].into()),
                    MappingType::DirectionPad(m) => (String::new(), m.position.into()),
                    MappingType::MouseCastSpell(m) => (m.bind.to_string(), m.position.into()),
                    MappingType::PadCastSpell(m) => (String::new(), m.position.into()),
                    MappingType::CancelCast(m) => (m.bind.to_string(), m.position.into()),
                    MappingType::Observation(m) => (m.bind.to_string(), m.position.into()),
                    MappingType::Fps(m) => (m.bind.to_string(), m.position.into()),
                    MappingType::Fire(m) => (m.bind.to_string(), m.position.into()),
                    MappingType::RawInput(m) => (m.bind.to_string(), m.position.into()),
                };
                (mapping, binding, pos, size)
            })
            .collect()
    }
}

impl From<&MappingConfig> for InputConfig {
    fn from(mapping_config: &MappingConfig) -> Self {
        let mut all_bindings: HashMap<String, Vec<InputBinding>> = HashMap::new();

        for (action, mapping) in &mapping_config.mappings {
            if let MappingType::PadCastSpell(m) = mapping {
                all_bindings.insert(action.to_string(), vec![m.input_binding.clone()]);
                all_bindings.insert(m.pad_action.to_string(), vec![m.pad_input_binding.clone()]);
            } else {
                all_bindings.insert(action.to_string(), vec![mapping.get_input_binding()]);
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
                        ButtonBinding::new(vec![KeyCode::Digit1.into()]),
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
                        ButtonBinding::new(vec![KeyCode::Digit2.into()]),
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
                        ButtonBinding::new(vec![KeyCode::Digit3.into()]),
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
                        ButtonBinding::new(vec![
                            KeyCode::ControlLeft.into(),
                            KeyCode::Digit3.into(),
                        ]),
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
                        ButtonBinding::new(vec![KeyCode::Digit4.into()]),
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
                        ButtonBinding::new(vec![KeyCode::Digit5.into()]),
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
                        DirectionBinding::Button {
                            up: KeyCode::KeyW.into(),
                            down: KeyCode::KeyS.into(),
                            left: KeyCode::KeyA.into(),
                            right: KeyCode::KeyD.into(),
                        },
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
                        (500, 1000).into(),
                        300,
                        100.,
                        100.,
                        DirectionBinding::JoyStick {
                            x: GamepadAxis::LeftStickX,
                            y: GamepadAxis::LeftStickY,
                        },
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
                        ButtonBinding::new(vec![KeyCode::Backquote.into()]),
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
                        ButtonBinding::new(vec![KeyCode::KeyE.into()]),
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
                        ButtonBinding::new(vec![KeyCode::KeyQ.into()]),
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
                        ButtonBinding::new(vec![KeyCode::AltLeft.into()]),
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
                        ButtonBinding::new(vec![MouseButton::Back.into()]),
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
                        MappingAction::PadCastDirection1,
                        DirectionBinding::Button {
                            up: KeyCode::KeyW.into(),
                            down: KeyCode::KeyS.into(),
                            left: KeyCode::KeyA.into(),
                            right: KeyCode::KeyD.into(),
                        },
                        ButtonBinding::new(vec![KeyCode::KeyJ.into()]),
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
                        ButtonBinding::new(vec![KeyCode::Space.into()]),
                    )
                    .unwrap(),
                ),
            ),
            (
                MappingAction::Observation1,
                MappingType::Observation(
                    MappingObservation::new(
                        "Observation",
                        4,
                        (2000, 300).into(),
                        0.5,
                        0.5,
                        ButtonBinding::new(vec![MouseButton::Forward.into()]),
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
                        ButtonBinding::new(vec![MouseButton::Right.into()]),
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
                        ButtonBinding::new(vec![MouseButton::Left.into()]),
                    )
                    .unwrap(),
                ),
            ),
            (
                MappingAction::RawInput1,
                MappingType::RawInput(
                    MappingRawInput::new(
                        "RawInput",
                        (2000, 300).into(),
                        ButtonBinding::new(vec![KeyCode::Enter.into()]),
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
    let config_string = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read mapping config file: {}", e))?;
    let mapping_config: MappingConfig = serde_json::from_str(&config_string)
        .map_err(|e| format!("Cannot deserialize mapping config: {}", e))?;

    let input_config: InputConfig = InputConfig::from(&mapping_config);

    Ok((mapping_config, input_config))
}

pub fn save_mapping_config(config: &MappingConfig, path: &Path) -> Result<(), String> {
    let json_string =
        to_string_pretty(config).map_err(|e| format!("Cannot serialize mapping config: {}", e))?;
    if let Some(parent) = path.parent() {
        create_dir_all(parent)
            .map_err(|e| format!("Cannot create directory for config file: {}", e))?;
    }

    let mut file =
        File::create(path).map_err(|e| format!("Cannot create mapping config file: {}", e))?;
    file.write_all(json_string.as_bytes())
        .map_err(|e| format!("Cannot write to mapping config file: {}", e))?;

    Ok(())
}
