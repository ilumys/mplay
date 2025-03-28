//! Tracks global state and performs event handling

use ratatui::widgets::ListState;

pub struct State {
    pub all_tracks: ListState,
}

impl State {
    pub fn new() -> Self {
        Self {
            all_tracks: ListState::default(),
        }
    }
}
