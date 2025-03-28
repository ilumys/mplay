//! Contains all rendering configuration

// TODO tabs
// queue: listing all tracks in queue and their position
// all: display all tracks
// artist: display all artists, selecing provides view of their albums and tracks
// album: display all albums, selecting provides view of their tracks

use std::time::Duration;

use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    prelude::Stylize,
    widgets::{Block, List, ListItem, Paragraph},
};

use crate::library::{AudioTrack, TrackList};

mod state;

use state::State;

pub struct UserInterface {
    active: bool,
    state: state::State,
    tracks: TrackList,
}

impl UserInterface {
    pub fn new(track_list: TrackList) -> Self {
        UserInterface {
            active: true,
            state: State::new(),
            tracks: track_list,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) {
        while self.active {
            terminal.draw(|frame| self.draw(frame)).unwrap();

            // turn `read` call into `poll` to not block input
            if event::poll(Duration::from_millis(100)).is_ok_and(|r| r) {
                if let Event::Key(k) = event::read().expect("event read") {
                    self.handle_key(k);
                }
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let [search_area, body_area, status_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Percentage(90),
            Constraint::Length(6),
        ])
        .areas(frame.area());

        let search_data = Paragraph::new("todo: implement search by artist, album, or track")
            .italic()
            .gray()
            .block(Block::bordered().title("search").bold());

        let status_data = Paragraph::new("todo: implement status for current track")
            .block(Block::bordered().title("status").bold());

        frame.render_widget(search_data, search_area);
        frame.render_widget(status_data, status_area);
        self.render_all_tracks(body_area, frame);
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Esc | event::KeyCode::Char('q') => self.active = false,
            KeyCode::Down => self.state.all_tracks.select_next(),
            KeyCode::Up => self.state.all_tracks.select_previous(),
            _ => (),
        }
    }

    fn render_search(&mut self, area: Rect, buf: &mut Buffer) {
        todo!("render stateful search, accepting user input");
    }

    fn render_all_tracks(&mut self, area: Rect, frame: &mut Frame) {
        let block = Block::bordered().title("tracks");

        let track_list: Vec<ListItem> = self
            .tracks
            .iter()
            .map(|i| match i {
                AudioTrack::Full(x) => ListItem::from(x.title.clone()),
                AudioTrack::Limited(x) => ListItem::from(x.title.clone()),
            })
            .collect();

        let track_list = List::new(track_list).block(block).highlight_symbol(">");

        frame.render_stateful_widget(track_list, area, &mut self.state.all_tracks);
    }

    fn render_status(&mut self, area: Rect, buf: &mut Buffer) {
        todo!("render status of currently playing track");
    }
}
