//! Tracks global state and performs event handling

use ratatui::widgets::TableState;

pub struct State {
    pub all_tracks: TableState,
    pub mode: Mode,
    pub search: Search,
}

pub struct Search {
    // casting `cursor_index` back and forth between `usize` and `u32` displeases
    // me but I am similarly distressed at the thought of storing `usize` in
    // the struct to index with. no winning here
    pub cursor_index: usize,
    pub query: String,
}

// relate to selected panel?
pub enum Mode {
    Playback,
    Search,
}

impl State {
    pub fn new() -> Self {
        Self {
            all_tracks: TableState::default(),
            mode: Mode::Playback,
            search: Search::new(),
        }
    }
}

impl Search {
    fn new() -> Self {
        Self {
            cursor_index: 0,
            query: String::new(),
        }
    }

    fn cursor_byte_index(&self) -> usize {
        self.query
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.cursor_index)
            .unwrap_or(self.query.len())
    }

    pub fn move_cursor_left(&mut self) {
        let n = self.cursor_index.saturating_sub(1);
        self.cursor_index = n.clamp(0, self.query.chars().count());
    }

    pub fn move_cursor_right(&mut self) {
        let n = self.cursor_index.saturating_add(1);
        self.cursor_index = n.clamp(0, self.query.chars().count());
    }

    pub fn new_char(&mut self, ch: char) {
        let i = self.cursor_byte_index();
        self.query.insert(i, ch);
        self.move_cursor_right()
    }

    pub fn delete_char(&mut self) {
        let i = self.cursor_byte_index();
        if i != 0 && self.query.is_char_boundary(i) {
            // since this is 'backspace', not 'delete', remove char preceding
            // the cursor. i != 0 guards statement
            self.query.remove(i - 1);
            self.move_cursor_left()
        }
    }
}
