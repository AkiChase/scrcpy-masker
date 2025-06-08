use std::collections::HashMap;

use bevy::{ecs::resource::Resource, input::keyboard::KeyCode};
use bevy_ineffable::{
    config::InputConfig,
    phantom::IAWrp,
    prelude::{ContinuousBinding, IneffableCommands, InputAction, InputBinding, InputKind},
};
use paste::paste;
use seq_macro::seq;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display};

use crate::mapping::{
    tap::{MappingRepeatTap, MappingSingleTap},
    utils::Size,
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

        pub fn ineff_pulse(self) -> IAWrp<MappingAction, bevy_ineffable::phantom::Pulse> {
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

#[derive(Serialize, Deserialize, Debug)]
pub enum MappingType {
    SingleTap(MappingSingleTap),
    RepeatTap(MappingRepeatTap),
    // MultipleTap(MappingMultipleTap),
    // Swipe(MappingSwipe),
}

impl_mapping_type_methods! {
    MappingType {
        SingleTap => MappingSingleTap,
        RepeatTap => MappingRepeatTap,
        // Swipe => MappingSwipe,
        // MultipleTap => MappingMultipleTap,
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MappingConfig {
    pub title: String,
    pub original_size: Size,
    pub mappings: HashMap<MappingAction, MappingType>,
}

#[derive(Resource)]
pub struct ActiveMappingConfig(pub MappingConfig);

pub fn load_mapping_config(mut ineffable: IneffableCommands) -> MappingConfig {
    // TODO load from file
    let mapping_config = MappingConfig {
        title: "test".to_string(),
        original_size: Size {
            width: 1920,
            height: 1080,
        },
        mappings: HashMap::from([
            (
                MappingAction::SingleTap1,
                MappingType::SingleTap(
                    MappingSingleTap::new(
                        (100, 200).into(),
                        "test",
                        0,
                        3000,
                        false,
                        ContinuousBinding::hold(KeyCode::KeyA).0,
                    )
                    .unwrap(),
                ),
            ),
            (
                MappingAction::SingleTap2,
                MappingType::SingleTap(
                    MappingSingleTap::new(
                        (200, 300).into(),
                        "test",
                        0,
                        0,
                        true,
                        ContinuousBinding::hold(KeyCode::KeyS).0,
                    )
                    .unwrap(),
                ),
            ),
        ]),
    };

    // apply config
    let mut all_bindings: HashMap<String, Vec<InputBinding>> = HashMap::new();

    for (action, mapping) in &mapping_config.mappings {
        all_bindings.insert(action.to_string(), vec![mapping.get_bind()]);
    }

    let binding_config: HashMap<String, HashMap<String, Vec<InputBinding>>> =
        HashMap::from([("MappingAction".to_string(), all_bindings)]);

    let mut config = InputConfig::new();
    config.bindings = binding_config;
    ineffable.set_config(&config);

    mapping_config
}
