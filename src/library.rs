//! Scans for and builds representative tracks for all audio files in a given directory
//!
//! Once the initial read is completed, groups tracks by artist and album for representation in TUI
//!
//! TODO: cache

use std::{collections::HashMap, fs::File, path::Path};

use symphonia::{
    core::{formats::FormatOptions, io::MediaSourceStream, meta::MetadataOptions, probe::Hint},
    default::get_probe,
};

use crate::artist::{Album, AlbumList, Artist, ArtistList};

// this is a bit much ... need to clean impl
// it would, admittedly, be nice to build artist/album ephemerally
// schema on read, if you will
// but in a way, that's what I'm doing here, isn't it?

/// Base unit representing an audio file with accompanying metadata for playback
#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    /// File path
    path: String,
    /// Track title
    title: String,
    /// Album track belongs to
    album: String,
    /// Album artist(s)
    album_artist: String,
    /// Track artist(s)
    artist: String,
    /// If embedded lyrics, attached here
    lyrics: String,
    /// Duration, in seconds, of the track
    duration: f32,
    /// Track number in the album
    track_num: String,
    /// Total tracks in the parent album
    track_total: String,
    /// Date of track release
    date: String,
}

impl Track {
    pub fn path(&self) -> &str {
        &self.path.as_str()
    }
    pub fn title(&self) -> &str {
        &self.title.as_str()
    }
    pub fn album(&self) -> &str {
        &self.album.as_str()
    }
    pub fn album_artist(&self) -> &str {
        &self.album_artist.as_str()
    }
    pub fn artist(&self) -> &str {
        &self.artist.as_str()
    }
    pub fn lyrics(&self) -> &str {
        &self.lyrics.as_str()
    }
    pub fn duration(&self) -> f32 {
        self.duration
    }
    pub fn track_num(&self) -> &str {
        &self.track_num.as_str()
    }
    pub fn track_total(&self) -> &str {
        &self.track_total.as_str()
    }
    pub fn date(&self) -> &str {
        &self.date.as_str()
    }
}

/// Given a directory, take a vector of resulting Tracks and group them by album and artist
/// On completion, returns an ArtistList, comprising the grouped tracks
pub fn build_library(directory: &str) -> ArtistList {
    let mut tracks: Vec<Track> = Vec::new();
    compile_library(directory, &mut tracks);

    // I can hardly say I like this, but it works
    let mut artists: HashMap<&str, Vec<Album>> = HashMap::new();
    let mut album_names: Vec<&str> = Vec::new();
    for t in tracks.iter() {
        if !artists.contains_key(t.artist()) {
            // if artist doesn't exist, neither does this album
            // unless there's an unaccounted for edge case
            album_names.push(t.album());
            artists.insert(
                t.artist(),
                vec![Album::new(
                    t.album(),
                    t.album_artist(),
                    t.date(),
                    t.track_num(),
                    vec![t.clone()], // hmm
                )],
            );
        } else {
            if !album_names.contains(&t.album()) {
                album_names.push(t.album());
                artists.get_mut(t.artist()).unwrap().push(Album::new(
                    t.album(),
                    t.album_artist(),
                    t.date(),
                    t.track_num(),
                    vec![t.clone()],
                ));
            }
        }
        // what if track has no tags? add to a default artist
    }

    let mut artist_list: ArtistList = ArtistList::default();
    for (k, v) in artists.iter() {
        let albums = AlbumList::from(v);
        artist_list.add_artist(Artist::new(k, albums));
    }

    return artist_list;
}

/// Takes a directory and vector of tracks, iteratively scanning through it for all audio files
/// Once found, audio files are processed into Tracks and added to the input vector
fn compile_library(path: &str, tracks: &mut Vec<Track>) {
    let mut dirs: Vec<String> = vec![path.to_string()];
    // add a check is_audio_file to avoid scanning everything needlessly
    // do I want to rely on ext in file name, or check metadata...

    // too much type conversion going on here, clean it up
    loop {
        match dirs.pop() {
            Some(path_t) => {
                let path_i = Path::new(path_t.as_str());
                if path_i.is_dir() {
                    for path_i in path_i.read_dir().unwrap() {
                        dirs.push(String::from(path_i.unwrap().path().to_str().unwrap()));
                    }
                } else if path_i.is_file() {
                    if let Some(track) = process_track(path_i.to_str().unwrap()) {
                        tracks.push(track);
                    }
                }
            }
            None => break,
        }
    }
}

/// Takes a string slice representing path to an audio file as input, then reads the file and
/// attempts to convert create a representative Track from the audio file
fn process_track(path: &str) -> Option<Track> {
    let source = Box::new(File::open(path).expect("box file error"));
    let mss = MediaSourceStream::new(source, Default::default());

    let mut hint = Hint::new();
    hint.with_extension(Path::new(path).extension()?.to_str().expect("hint error"));

    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();

    let mut probe = match get_probe().format(&hint, mss, &fmt_opts, &meta_opts) {
        Ok(p) => p,
        Err(_e) => return None,
    };

    // intuitively this can be massively optimised
    // to start, function, then optimise
    let mut metadata: [String; 8] = Default::default();
    if let Some(meta) = probe.format.metadata().current() {
        let tags = meta.tags();
        if !tags.is_empty() {
            for tag in tags.iter().filter(|t| t.is_known()) {
                if let Some(key) = tag.std_key {
                    let key = format!("{:?}", key);
                    match key.as_str() {
                        "TrackTitle" => metadata[0] = tag.value.to_string(),
                        "Album" => metadata[1] = tag.value.to_string(),
                        "AlbumArtist" => metadata[2] = tag.value.to_string(),
                        "Artist" => metadata[3] = tag.value.to_string(),
                        "Lyrics" => metadata[4] = tag.value.to_string(),
                        "TrackNumber" => metadata[5] = tag.value.to_string(),
                        "TrackTotal" => metadata[6] = tag.value.to_string(),
                        "Date" => metadata[7] = tag.value.to_string(),
                        _ => {}
                    }
                }
            }
        }
    } else if let Some(meta) = probe.metadata.get().as_ref().and_then(|m| m.current()) {
        let tags = meta.tags();
        if !tags.is_empty() {
            for tag in tags.iter().filter(|t| t.is_known()) {
                if let Some(key) = tag.std_key {
                    let key = format!("{:?}", key);
                    match key.as_str() {
                        "TrackTitle" => metadata[0] = tag.value.to_string(),
                        "Album" => metadata[1] = tag.value.to_string(),
                        "AlbumArtist" => metadata[2] = tag.value.to_string(),
                        "Artist" => metadata[3] = tag.value.to_string(),
                        "Lyrics" => metadata[4] = tag.value.to_string(),
                        "TrackNumber" => metadata[5] = tag.value.to_string(),
                        "TrackTotal" => metadata[6] = tag.value.to_string(),
                        "Date" => metadata[7] = tag.value.to_string(),
                        _ => {}
                    }
                }
            }
        }
    }

    // is to_owned optimal or could this be improved?
    return Some(Track {
        path: path.to_string(),
        title: metadata[0].to_owned(),
        album: metadata[1].to_owned(),
        album_artist: metadata[2].to_owned(),
        artist: metadata[3].to_owned(),
        lyrics: metadata[4].to_owned(),
        duration: 0.0, // todo: set this proper
        track_num: metadata[5].to_owned(),
        track_total: metadata[6].to_owned(),
        date: metadata[7].to_owned(),
    });
}

fn _check_cache() {
    todo!("check if cache lock has changed. if yes, reload, else load");
}

fn _cache_library() {
    todo!("cache current library");
}
