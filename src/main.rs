//! mplay: music player based in the terminal

use std::path::PathBuf;

mod library;
mod ui;

fn main() {
    // todo: config file
    let music_dir = format!("{}/Music/", env!("HOME"));
    let library = library::build_library(PathBuf::from(music_dir));

    ui::UserInterface::new(library).run(ratatui::init());
    ratatui::restore();
}
