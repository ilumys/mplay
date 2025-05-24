//! mplay: music player based in the terminal

use std::path::PathBuf;

mod loader;
mod sink;
mod term;

use loader::AudioLibrary;

// TODO: make async where I can
fn main() {
    let music_dir = format!("{}/Music/", env!("HOME"));
    let library = AudioLibrary::from_directory(PathBuf::from(music_dir));

    term::UserInterface::new(library.tracks).run(ratatui::init());
    ratatui::restore();
}
