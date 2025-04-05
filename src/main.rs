//! mplay: music player based in the terminal

use std::path::PathBuf;

mod library;
mod ui;

use library::AudioLibrary;

fn main() {
    // todo: config file
    let music_dir = format!("{}/Music/", env!("HOME"));
    let library = AudioLibrary::from_directory(PathBuf::from(music_dir));

    ui::UserInterface::new(library.tracks).run(ratatui::init());
    ratatui::restore();
}
