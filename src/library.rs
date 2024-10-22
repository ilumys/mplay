use std::{collections::HashMap, fs::File, path::Path};

use symphonia::{
    core::{formats::FormatOptions, io::MediaSourceStream, meta::MetadataOptions, probe::Hint},
    default::get_probe,
};

use super::player::{Album, AlbumList, Artist, ArtistList};

// this is a bit much ... need to clean impl
// it would, admittedly, be nice to build artist/album ephemerally
// schema on read, if you will
// but in a way, that's what I'm doing here, isn't it?
#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    path: String,
    title: String,
    album: String,
    album_artist: String,
    artist: String,
    lyrics: String,
    duration: f32,
    track_num: String,
    track_total: String,
    date: String,
}

impl Track {
    pub fn get_path(&self) -> &str {
        return &self.path;
    }
    pub fn get_title(&self) -> &str {
        return &self.title;
    }
    pub fn get_album(&self) -> &str {
        return &self.album;
    }
    pub fn get_album_artist(&self) -> &str {
        return &self.album_artist;
    }
    pub fn get_artist(&self) -> &str {
        return &self.artist;
    }
    pub fn get_lyrics(&self) -> &str {
        return &self.lyrics;
    }
    pub fn get_duration(&self) -> f32 {
        return self.duration;
    }
    pub fn get_track_num(&self) -> &str {
        return &self.track_num;
    }
    pub fn get_track_total(&self) -> &str {
        return &self.track_total;
    }
    pub fn get_date(&self) -> &str {
        return &self.date;
    }
}

pub fn build_library(directory: &str) -> ArtistList {
    let mut tracks: Vec<Track> = vec![];
    compile_library(directory, &mut tracks);

    // I can hardly say I like this, but it works
    let mut artists: HashMap<&str, Vec<Album>> = HashMap::new();
    let mut album_names: Vec<&str> = vec![];
    for t in tracks.iter() {
        if !artists.contains_key(t.get_artist()) {
            // if artist doesn't exist, neither does this album
            // unless there's an unaccounted for edge case
            album_names.push(t.get_album());
            artists.insert(
                t.get_artist(),
                vec![Album::new(
                    t.get_album(),
                    t.get_album_artist(),
                    t.get_date(),
                    t.get_track_num(),
                    vec![t.clone()], // hmm
                )],
            );
        } else {
            if !album_names.contains(&t.get_album()) {
                album_names.push(t.get_album());
                artists.get_mut(t.get_artist()).unwrap().push(Album::new(
                    t.get_album(),
                    t.get_album_artist(),
                    t.get_date(),
                    t.get_track_num(),
                    vec![t.clone()],
                ));
            }
        }
        // what if track has no tags? add to a default artist
    }

    let mut artist_list: ArtistList = ArtistList::new();
    for (k, v) in artists.iter() {
        let albums = AlbumList::from(v);
        artist_list.add_artist(Artist::new(k, albums));
    }

    return artist_list;
}

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
