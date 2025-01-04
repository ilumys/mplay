//! Defines how audio files are represented in mplay, grouping by album and artist

use ratatui::{
    prelude::*,
    widgets::{ListItem, ListState},
};

use crate::library::Track;

#[derive(Clone, Default, PartialEq)]
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
    pub album_list: AlbumList,
}

#[derive(Clone, PartialEq)]
pub struct AlbumList {
    albums: Vec<Album>,
    pub state: ListState,
}

// this is the anchor of track display in the TUI, but it's a bit of a mess
// artists are duplicated as this is built from the 'artists', not 'album_artist' field
// consider restructing this so that tracks are grouped by album and album_artist, for artist
// but even this would not be a fully accurate representation. requires more thought
#[derive(Clone, Default, PartialEq)]
pub struct ArtistList {
    pub artists: Vec<Artist>,
    pub state: ListState,
}

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

impl AlbumList {
    pub fn albums(&self) -> &[Album] {
        &self.albums.as_ref()
    }
}

impl ArtistList {
    pub fn add_artist(&mut self, other: Artist) {
        self.artists.push(other);
    }
}

impl Artist {
    pub fn new(name: &str, album_list: AlbumList) -> Self {
        Self {
            name: name.to_string(),
            album_list,
        }
    }
}
