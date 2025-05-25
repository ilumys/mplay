//! mplay: music player based in the terminal

use std::path::PathBuf;

mod loader;
mod playback;
mod term;

fn main() {
    // todo: config file
    let music_dir = format!("{}/Music/", env!("HOME"));
    let library = loader::AudioLibrary::from_directory(PathBuf::from(music_dir));

    term::UserInterface::new(library.tracks).run(ratatui::init());
    ratatui::restore();
}
