//! Tracks global state and performs event handling

use ratatui::widgets::TableState;

pub struct State {
    pub all_tracks: TableState,
}

impl State {
    pub fn new() -> Self {
        Self {
            all_tracks: TableState::default(),
        }
    }
}
