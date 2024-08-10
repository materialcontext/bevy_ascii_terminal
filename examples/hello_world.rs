use bevy::prelude::*;
use bevy::color::palettes::css::{BLUE, GREEN, RED, WHITE, GRAY, LIMEGREEN};
use bevy_ascii_terminal::{prelude::*, TerminalPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, TerminalPlugin))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, spawn_terminal)
        .add_systems(Update, hello_world)
        .run();
}

fn spawn_terminal(mut commands: Commands) {
    let mut term = Terminal::new([20, 1]).with_border(Border::single_line());

    term.put_string([0, 0], "Press spacebar".bg(Color::Srgba(LIMEGREEN)));

    commands.spawn((TerminalBundle::from(term), AutoCamera));
}

fn hello_world(keys: Res<ButtonInput<KeyCode>>, mut q: Query<&mut Terminal>) {
    if keys.just_pressed(KeyCode::Space) {
        for mut term in q.iter_mut() {
            term.clear();
            term.put_char([0, 0], 'H'.fg(Color::Srgba(BLUE)).bg(Color::Srgba(GREEN)));
            term.put_char([1, 0], 'e'.fg(Color::Srgba(BLUE)).bg(Color::Srgba(WHITE)));
            term.put_char([2, 0], 'l'.fg(Color::Srgba(GREEN)).bg(Color::Srgba(BLUE)));
            term.put_char([3, 0], 'l'.fg(Color::Srgba(RED)).bg(Color::Srgba(GREEN)));
            term.put_char([4, 0], 'o'.fg(Color::Srgba(GREEN)).bg(Color::Srgba(GRAY)));

            term.put_string([6, 0], "World!");
        }
    }
}
