use bevy::prelude::*;

use bevy_terminal::{TerminalBundle, TerminalPlugin, render::TerminalRendererFont};

const FONTS: [&str;2] = ["alloy_curses_12x12.png", "zx_evolution_8x8.png"];
#[derive(Default)]
struct FontIndex(pub usize);

fn spawn_terminal(
    mut commands: Commands
) {
    let mut term_bundle = TerminalBundle::with_size(20,3);

    term_bundle.terminal.draw_border_single();
    term_bundle.terminal.put_string(1,1, "Press spacebar");
    commands.spawn_bundle(term_bundle);
    
    let mut cam = PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.0, 0.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    };
    cam.transform.translation += Vec3::new(10.0,1.5,0.0);

    commands.spawn_bundle(cam);
}

fn change_font(
    keys: Res<Input<KeyCode>>,
    mut font_index: ResMut<FontIndex>,
    mut q: Query<&mut TerminalRendererFont>,
) {
    if keys.just_pressed(KeyCode::Space) {
        for mut font in q.iter_mut() {
            font_index.0 = 1 - font_index.0;
            font.0 = String::from(FONTS[font_index.0]);
        }
    }
}

fn main() {
    App::build()
    .init_resource::<FontIndex>()
    .add_plugins(DefaultPlugins)
    .add_plugin(TerminalPlugin)
    .add_startup_system(spawn_terminal.system())
    .add_system(change_font.system())
    .run()
}