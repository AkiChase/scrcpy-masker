pub mod basic;

use basic::BasicPlugin;
use bevy::app::{App, Plugin};

pub struct UiPlugins;

impl Plugin for UiPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins(BasicPlugin);
    }
}
