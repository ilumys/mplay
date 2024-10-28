use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    prelude::*,
    widgets::{Block, List, ListItem, Paragraph, Widget},
    DefaultTerminal,
};

use crate::artist::ArtistList;

pub struct Player {
    active: bool,
    artist_list: ArtistList,
    // implement queue
}

impl Player {
    // control
    pub fn new(artist_list: ArtistList) -> Self {
        Self {
            active: true,
            artist_list,
        }
    }
    pub fn run(mut self, mut terminal: DefaultTerminal) {
        while self.active {
            terminal
                .draw(|frame| {
                    frame.render_widget(&mut self, frame.area());
                    if let Event::Key(key) = event::read().expect("read event") {
                        self.handle_key(key);
                    }
                })
                .unwrap();
        }
    }
    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        match key.code {
            // somehow, different key strokes move the list. very odd
            KeyCode::Esc | KeyCode::Char('q') => self.active = false,
            KeyCode::Up => self.select_previous(),
            KeyCode::Down => self.select_next(),
            KeyCode::Enter | KeyCode::Char(' ') => (),
            _ => (),
        }
    }
    fn select_next(&mut self) {
        self.artist_list.state.select_next();
    }
    fn select_previous(&mut self) {
        self.artist_list.state.select_previous();
    }

    // rendering
    fn render_artist(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered().title("artists");
        let artists: Vec<ListItem> = self
            .artist_list
            .artists
            .iter()
            .map(|artist| ListItem::from(artist))
            .collect();
        let list = List::new(artists)
            .block(block)
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">");
        StatefulWidget::render(list, area, buf, &mut self.artist_list.state);
    }

    fn render_album(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered().title("albums");
        let index = match self.artist_list.state.selected() {
            Some(i) => i,
            None => 0,
        };

        let albums: Vec<ListItem> = self.artist_list.artists[index]
            .album_list
            .albums()
            .iter()
            .map(|album| ListItem::from(album))
            .collect();

        let list = List::new(albums)
            .block(block)
            .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
            .highlight_symbol(">");

        StatefulWidget::render(
            list,
            area,
            buf,
            &mut self.artist_list.artists[index].album_list.state,
        );
    }

    fn render_search(area: Rect, buf: &mut Buffer) {
        Paragraph::new("search by artist, album, or track")
            .italic()
            .gray()
            .block(Block::bordered().title("search").bold())
            .render(area, buf);
    }

    fn render_playing(area: Rect, buf: &mut Buffer) {
        Paragraph::new("current track")
            .block(Block::bordered().title("currently playing".bold()))
            .render(area, buf);
    }
}

impl Widget for &mut Player {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let [search, body, status] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Percentage(90),
            Constraint::Length(6),
        ])
        .areas(area);

        let [lartist, lalbum] =
            Layout::horizontal([Constraint::Percentage(28), Constraint::Percentage(72)])
                .areas(body);

        Player::render_search(search, buf);
        self.render_artist(lartist, buf);
        self.render_album(lalbum, buf);
        Player::render_playing(status, buf);
    }
}
