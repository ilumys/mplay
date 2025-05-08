//! Contains all rendering configuration

// TODO tabs
// queue: listing all tracks in queue and their position
// all: display all tracks
// artist: display all artists, selecing provides view of their albums and tracks
// album: display all albums, selecting provides view of their tracks

use std::{ops::Deref, time::Duration};

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Position, Rect},
    prelude::Stylize,
    style::{Modifier, Style},
    widgets::{Block, Cell, Paragraph, Row, Table},
};

use crate::library::{AudioTrack, LibraryCollection, Player};

mod state;

use state::State;

pub struct UserInterface {
    active: bool,
    player: Player,
    state: state::State,
    tracks: LibraryCollection,
}

impl UserInterface {
    pub fn new(track_list: LibraryCollection) -> Self {
        UserInterface {
            active: true,
            player: Player::new(),
            state: State::new(),
            tracks: track_list,
        }
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) {
        while self.active {
            terminal.draw(|frame| self.draw(frame)).unwrap();

            self.player.try_next();

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

        self.render_search(search_area, frame);
        self.render_all_tracks(body_area, frame);
        self.render_status(status_area, frame);
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match self.state.mode {
            state::Mode::Playback => {
                if key.kind == KeyEventKind::Press {
                    // toggle between selected panels (widgets)?
                    match key.code {
                        KeyCode::Char('q') => self.active = false,
                        KeyCode::Down => self.state.all_tracks.select_next(),
                        KeyCode::Up => self.state.all_tracks.select_previous(),
                        KeyCode::PageDown => self.state.all_tracks.select_last(),
                        KeyCode::PageUp => self.state.all_tracks.select_first(),
                        KeyCode::Enter => {
                            match self.state.all_tracks.selected() {
                                Some(i) => self.player.append_queue(self.tracks[i].clone()),
                                None => unreachable!(), // index out of bounds
                            };
                            self.state.all_tracks.select_next();
                        }
                        KeyCode::Char(' ') => self.player.toggle_pause(),
                        KeyCode::Char('c') => self.player.clear_queue(),
                        KeyCode::Char('>') => self.player.next(),
                        KeyCode::Char('/') => self.state.mode = state::Mode::Search,
                        _ => (),
                    }
                }
            }
            state::Mode::Search => {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc => {
                            self.state.search.query = String::new();
                            self.state.search.cursor_index = 0;
                            self.state.mode = state::Mode::Playback;
                        }
                        KeyCode::Enter => self.state.mode = state::Mode::Playback,
                        KeyCode::Char(c) => self.state.search.new_char(c),
                        KeyCode::Left => self.state.search.move_cursor_left(),
                        KeyCode::Right => self.state.search.move_cursor_right(),
                        KeyCode::Backspace | KeyCode::Delete => self.state.search.delete_char(),
                        _ => (),
                    }
                }
            }
        }
    }

    fn render_search(&mut self, area: Rect, frame: &mut Frame) {
        let x = self.state.search.query.as_str();
        let w = Paragraph::new(x).block(
            Block::bordered()
                .title("search")
                .title_style(Modifier::BOLD),
        );

        match self.state.mode {
            state::Mode::Playback => (),
            state::Mode::Search => frame.set_cursor_position(Position::new(
                area.x + self.state.search.cursor_index as u16 + 1,
                area.y + 1,
            )),
        }

        frame.render_widget(w, area);
    }

    fn render_all_tracks(&mut self, area: Rect, frame: &mut Frame) {
        let header = Row::new([
            Cell::new(""),
            Cell::new("title"),
            Cell::new("artist(s)"),
            Cell::new("album"),
            Cell::new("release date"),
        ])
        .bold();
        let rows: Vec<Row> = self
            .tracks
            .iter()
            .filter_map(|v| {
                let q = self.state.search.query.to_lowercase();
                match v.deref() {
                    AudioTrack::Extended(x)
                        if x.title.to_lowercase().contains(&q)
                            || x.album.to_lowercase().contains(&q)
                            || x.artists.to_lowercase().contains(&q) =>
                    {
                        Some(Row::new([
                            Cell::new(x.path.clone()),
                            Cell::new(x.title.clone()),
                            Cell::new(x.artists.clone()),
                            Cell::new(x.album.clone()),
                            Cell::new(x.date.clone()),
                        ]))
                    }
                    AudioTrack::Limited(x) if x.title.to_lowercase().contains(&q) => {
                        Some(Row::new([
                            Cell::new(x.path.clone()),
                            Cell::new(x.title.clone()),
                        ]))
                    }
                    _ => None,
                }
            })
            .collect();

        let tbl = Table::new(
            rows,
            [
                Constraint::Max(0),
                Constraint::Fill(4),
                Constraint::Fill(2),
                Constraint::Fill(2),
                Constraint::Fill(1),
            ],
        )
        .block(Block::bordered().border_set(ratatui::symbols::border::ROUNDED))
        .header(header)
        .column_spacing(2)
        .row_highlight_style(Style::new().add_modifier(Modifier::REVERSED));

        frame.render_stateful_widget(tbl, area, &mut self.state.all_tracks);
    }

    /// Render the status panel
    fn render_status(&mut self, area: Rect, frame: &mut Frame) {
        // TODO: duration, play/pause
        let last_played: String = match &self.player.last_played {
            Some(s) => match s.deref() {
                AudioTrack::Extended(i) => {
                    format!("{0}\n{1}\n{2} - {3}", i.title, i.artists, i.album, i.date)
                }
                AudioTrack::Limited(i) => i.title.clone(),
            },
            None => String::from("none"),
        };
        let title = Block::bordered()
            .title("currently playing")
            .title_style(Modifier::BOLD);
        let widget = Paragraph::new(last_played).block(title);
        frame.render_widget(widget, area);
    }
}
