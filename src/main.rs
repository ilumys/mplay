mod artist;
mod library;
mod player;

fn main() {
    let music_dir = format!("{}/Music/", env!("HOME"));
    // todo: lazy load library
    // start display with only artists to allow more time/space to load all
    let library = library::build_library(music_dir.as_str());
    let terminal = ratatui::init();

    // strong want to optimise player
    let player = player::Player::new(library);
    player.run(terminal);
    ratatui::restore();
}
