//! Contains all rendering configuration

// TODO tabs
// queue: listing all tracks in queue and their position
// all: display all tracks
// artist: display all artists, selecing provides view of their albums and tracks
// album: display all albums, selecting provides view of their tracks

use std::time::Duration;

use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    prelude::Stylize,
    style::{Modifier, Style},
    widgets::{Block, Cell, Paragraph, Row, Table},
};

use crate::library::{AudioTrack, Player};

mod state;

use state::State;

pub struct UserInterface {
    active: bool,
    player: Player,
    state: state::State,
    tracks: Box<[AudioTrack]>, // reference to slice? but it's heap, so.. doesn't matter?
}

impl UserInterface {
    pub fn new(track_list: Box<[AudioTrack]>) -> Self {
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
        // `key.kind` not always set. keep an eye out for events failing at this step
        // unlikely to occur for me, given kb/os used
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.active = false,
            KeyCode::Down => self.state.all_tracks.select_next(),
            KeyCode::Up => self.state.all_tracks.select_previous(),
            KeyCode::PageDown => self.state.all_tracks.select_last(), // select bottom of visible page?
            KeyCode::PageUp => self.state.all_tracks.select_first(),  // top of visible page...
            KeyCode::Enter => {
                let path = match self.state.all_tracks.selected() {
                    Some(i) => match &self.tracks[i] {
                        AudioTrack::Extended(a) => &a.path,
                        AudioTrack::Limited(a) => &a.path,
                    },
                    None => unreachable!(), // index out of bounds
                };
                self.player.append(path);
                self.state.all_tracks.select_next();
            }
            KeyCode::Char(' ') => self.player.toggle_pause(),
            KeyCode::Char('c') => self.player.clear_queue(),
            KeyCode::Char('>') => self.player.next(),
            _ => (),
        }
    }

    // fn render_search(&mut self, area: Rect, buf: &mut Buffer) {
    //     todo!("render stateful search, accepting user input");
    // }

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
            .map(|v| match v {
                AudioTrack::Extended(x) => Row::new([
                    Cell::new(x.path.clone()),
                    Cell::new(x.title.clone()),
                    Cell::new(x.artists.clone()),
                    Cell::new(x.album.clone()),
                    Cell::new(x.date.clone()),
                ]),
                AudioTrack::Limited(x) => {
                    Row::new([Cell::new(x.path.clone()), Cell::new(x.title.clone())])
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

    // fn render_status(&mut self, area: Rect, buf: &mut Buffer) {
    //     todo!("render status of currently playing track");
    // }
}
