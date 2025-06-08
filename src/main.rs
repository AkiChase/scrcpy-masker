use bevy::prelude::*;
use scrcpy_masker::{mapping, ui};

fn main() {
    App::new()
        .add_plugins((ui::UiPlugins, mapping::HotKeyPlugins))
        .run();
}
