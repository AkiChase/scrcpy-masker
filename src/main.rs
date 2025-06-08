use std::{env, process::exit};

use bevy::prelude::*;
use scrcpy_masker::{mapping, ui, update};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.contains(&"--self_update".to_string()) {
        update::self_update();
        exit(0);
    }

    App::new()
        .add_plugins((ui::UiPlugins, mapping::HotKeyPlugins))
        .run();
}
