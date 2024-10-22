use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    prelude::*,
    widgets::{Block, List, ListItem, ListState, Paragraph, Widget, Wrap},
    DefaultTerminal,
};

use super::library::Track;

pub struct Player {
    active: bool,
    artist_list: ArtistList,
    playing: Option<Track>,
}

#[derive(Clone, PartialEq)]
pub struct Album {
    name: String,
    album_artist: String,
    date: String,
    num_tracks: String,
    track_list: Vec<Track>,
}

#[derive(Clone, PartialEq)]
pub struct Artist {
    name: String,
    albums: AlbumList,
}

#[derive(Clone, PartialEq)]
pub struct AlbumList {
    albums: Vec<Album>,
    state: ListState,
}

#[derive(Clone, PartialEq)]
pub struct ArtistList {
    artists: Vec<Artist>,
    state: ListState,
}

// curious if compiler would optimise half these impl's out
// seeing as there are multiple near duplicates
// build release and have a look with ghidra? when done

impl
    FromIterator<(
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        Vec<Track>,
    )> for AlbumList
{
    fn from_iter<
        T: IntoIterator<
            Item = (
                &'static str,
                &'static str,
                &'static str,
                &'static str,
                Vec<Track>,
            ),
        >,
    >(
        iter: T,
    ) -> Self {
        let albums = iter
            .into_iter()
            .map(|(name, album_artist, date, num_tracks, track_list)| {
                Album::new(name, album_artist, date, num_tracks, track_list)
            })
            .collect();
        let state = ListState::default();
        Self { albums, state }
    }
}

impl FromIterator<(&'static str, AlbumList)> for ArtistList {
    fn from_iter<T: IntoIterator<Item = (&'static str, AlbumList)>>(iter: T) -> Self {
        let artists = iter
            .into_iter()
            .map(|(name, albums)| Artist::new(name, albums))
            .collect();
        let state = ListState::default();
        Self { artists, state }
    }
}

impl From<&Artist> for ListItem<'_> {
    fn from(value: &Artist) -> Self {
        let line = Line::from(format!("{}", value.name));
        ListItem::new(line)
    }
}

impl From<&Album> for ListItem<'_> {
    fn from(value: &Album) -> Self {
        let line = Line::from(format!("{}", value.name));
        ListItem::new(line)
    }
}

impl Default for Album {
    fn default() -> Self {
        Self {
            album_artist: String::new(),
            date: String::new(),
            name: String::new(),
            num_tracks: String::new(),
            track_list: vec![],
        }
    }
}

impl Album {
    pub fn new(
        name: &str,
        album_artist: &str,
        date: &str,
        num_tracks: &str,
        track_list: Vec<Track>,
    ) -> Self {
        Self {
            name: name.to_string(),
            album_artist: album_artist.to_string(),
            date: date.to_string(),
            num_tracks: num_tracks.to_string(),
            track_list,
        }
    }
}

impl From<&Vec<Album>> for AlbumList {
    fn from(value: &Vec<Album>) -> Self {
        Self {
            albums: value.clone(),
            state: ListState::default(),
        }
    }
}

impl ArtistList {
    pub fn new() -> Self {
        Self {
            artists: vec![],
            state: ListState::default(),
        }
    }
    pub fn add_artist(&mut self, other: Artist) {
        self.artists.push(other);
    }
    pub fn get_artists(&mut self) -> &Vec<Artist> {
        return &self.artists;
    }
}

impl Artist {
    pub fn new(name: &str, albums: AlbumList) -> Self {
        Self {
            name: name.to_string(),
            albums,
        }
    }
}

// work in progress
// impl Default for Player {
//     fn default() -> Self {
//         Self {
//             active: true,
//             artist_list:
//         }
//     }
// }

impl Player {
    // control
    pub fn new(artist_list: ArtistList) -> Self {
        Self {
            active: true,
            artist_list,
            playing: None,
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
            KeyCode::Esc | KeyCode::Char('q') => self.active = false,
            KeyCode::Up => self.select_previous(),
            KeyCode::Down => self.select_next(),
            KeyCode::Enter | KeyCode::Char(' ') => {}
            _ => {}
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
            .albums
            .albums
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
            &mut self.artist_list.artists[index].albums.state,
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
