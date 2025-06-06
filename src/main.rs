use bevy::prelude::*;
use scrcpy_masker::ui;

fn main() {
    App::new().add_plugins(ui::BasicPlugin).run();
}
