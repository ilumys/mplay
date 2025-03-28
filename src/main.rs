//! mplay: music player based in the terminal
//!
//! Leverages ratatui to draw a terminal user interface and symphonia for audio playback

use std::path::PathBuf;

use ui::UserInterface;

mod library;
mod ui;

fn main() {
    // todo: config file
    let music_dir = format!("{}/Music/", env!("HOME"));
    // todo: lazy load library
    // start display with only artists to allow more time/space to load all
    let library = library::build_library(PathBuf::from(music_dir));
    let terminal = ratatui::init();

    // strong want to optimise player
    UserInterface::new(library).run(terminal);
    ratatui::restore();
}
