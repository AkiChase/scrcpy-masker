/// Bevy basic plugin that creates a window with a transparent background and a border.
use bevy::prelude::*;

pub struct MappingLabelPlugin;

impl Plugin for MappingLabelPlugin {
    fn build(&self, _app: &mut App) {
        // app.add_systems(Startup, temp);
    }
}

// TODO 渲染 active mapping

// fn temp(mut commands: Commands) {
//     const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);

//     commands.spawn((
//         Node {
//             width: Val::Percent(100.),
//             height: Val::Percent(100.),
//             justify_content: JustifyContent::Center,
//             align_items: AlignItems::Center,
//             column_gap: Val::Px(20.),
//             ..default()
//         },
//         BackgroundColor(Color::NONE),
//         children![(
//             Button,
//             Node {
//                 width: Val::Px(150.0),
//                 height: Val::Px(65.0),
//                 justify_content: JustifyContent::Center,
//                 align_items: AlignItems::Center,
//                 ..default()
//             },
//             BorderColor(Color::BLACK),
//             BorderRadius::MAX,
//             BackgroundColor(NORMAL_BUTTON),
//             children![(
//                 Text::new("Button1"),
//                 TextFont {
//                     font_size: 12.,
//                     ..default()
//                 },
//                 TextColor(Color::srgb(0.9, 0.9, 0.9)),
//                 TextShadow::default(),
//             )]
//         )],
//     ));
// }
