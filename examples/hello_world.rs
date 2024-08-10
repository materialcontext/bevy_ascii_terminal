use bevy::prelude::*;
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

    term.put_string([0, 0], "Press spacebar".bg(Color::LIME_GREEN));

    commands.spawn((TerminalBundle::from(term), AutoCamera));
}

fn hello_world(keys: Res<ButtonInput<KeyCode>>, mut q: Query<&mut Terminal>) {
    if keys.just_pressed(KeyCode::Space) {
        for mut term in q.iter_mut() {
            term.clear();
            term.put_char([0, 0], 'H'.fg(Color::BLUE).bg(Color::GREEN));
            term.put_char([1, 0], 'e'.fg(Color::BLUE).bg(Color::WHITE));
            term.put_char([2, 0], 'l'.fg(Color::GREEN).bg(Color::BLUE));
            term.put_char([3, 0], 'l'.fg(Color::RED).bg(Color::GREEN));
            term.put_char([4, 0], 'o'.fg(Color::GREEN).bg(Color::GRAY));

            term.put_string([6, 0], "World!");
        }
    }
}
