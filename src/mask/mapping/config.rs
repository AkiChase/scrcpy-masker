use std::{
    collections::HashMap,
    fs::{File, create_dir_all},
    io::Write,
    path::Path,
};

use bevy::{ecs::resource::Resource, input::keyboard::KeyCode};
use bevy_ineffable::{
    config::InputConfig,
    phantom::IAWrp,
    prelude::{ContinuousBinding, InputAction, InputBinding, InputKind, PulseBinding},
};
use paste::paste;
use ron::ser::{PrettyConfig, to_string_pretty};
use seq_macro::seq;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display};

use crate::{
    mask::mapping::{
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
        )*
    }

    impl MappingAction {
        pub fn input_kind(&self)->InputKind{
            match self {
                #(
                    MappingAction::SingleTap~N => InputKind::Continuous,
                    MappingAction::RepeatTap~N => InputKind::Continuous,
                    MappingAction::MultipleTap~N => InputKind::Pulse,
                    MappingAction::Swipe~N => InputKind::Pulse,
                )*
            }
        }

        pub fn ineff_continuous(&self) -> IAWrp<MappingAction, bevy_ineffable::phantom::Continuous> {
            match self {
                #(
                    MappingAction::SingleTap~N => self.clone()._singletap~N(),
                    MappingAction::RepeatTap~N => self.clone()._repeattap~N(),
                )*
                _ => panic!("ineff_continuous called on non-continuous variant"),
            }
        }

        pub fn ineff_pulse(&self) -> IAWrp<MappingAction, bevy_ineffable::phantom::Pulse> {
            match self {
                #(
                    MappingAction::MultipleTap~N => self.clone()._multipletap~N(),
                    MappingAction::Swipe~N => self.clone()._swipe~N(),
                )*
                _ => panic!("ineff_pulse called on non-pulse variant"),
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
    // Swipe(MappingSwipe),
}

impl_mapping_type_methods! {
    MappingType {
        SingleTap => MappingSingleTap,
        RepeatTap => MappingRepeatTap,
        MultipleTap => MappingMultipleTap,
        // Swipe => MappingSwipe,
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
            all_bindings.insert(action.to_string(), vec![mapping.get_bind()]);
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
            width: 1280,
            height: 720,
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
