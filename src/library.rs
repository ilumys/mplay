//! Scans for and builds representative tracks for all audio files in a given directory
//!
//! Once the initial read is completed, groups tracks by artist and album for representation in TUI
//!
//! TODO: cache

use std::{collections::HashMap, fs::File, path::{Path, PathBuf}};

use symphonia::{
    core::{
        errors::Error as SymphoniaError, formats::FormatOptions, io::MediaSourceStream,
        meta::Tag, meta::MetadataOptions, probe::Hint,
    },
    default::get_probe,
};

// an AudioTrack is either known or unknown
// if it is known, it SHOULD belong to an album
// an album has an album_artist; and a track has an artist
// albums belong to the album_artist

/// Audio track with extended metadata present
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FullAudioTrack {
    path: String,
    title: String,
    album: String,
    album_artist: String,
    artist: String,
    track_num: i32,
    track_total: i32,
    duration: f64, // calculate on opening the file. remove from here, or preempt?
    date: String,
    lyrics: String // visuals ?
}

/// Audio track with no detected metadata
#[derive(Clone, Debug, PartialEq)]
pub struct LimitedAudioTrack {
    path: String,
    title: String
}

/// Base unit representing an audio file with accompanying metadata for playback
///
/// Variants:
///   Full: where extended audio metadata is detected
///   Limited: where no additional metadata is found
#[derive(Clone, Debug, PartialEq)]
pub enum AudioTrack {
    Full(FullAudioTrack),
    Limited(LimitedAudioTrack),
}

#[non_exhaustive]
struct Album {
    name: String,
    artist: String,
    tracks: Vec<AudioTrack>
    // num tracks? date rel?
}

type ArtistCollection = HashMap<String, Vec<Album>>;

/// Given a directory, take a vector of resulting `AudioTrack`s and group them by album and artist
/// On completion, returns an `ArtistList`, comprising the grouped tracks
pub fn build_library(directory: PathBuf) -> ArtistCollection {
    let mut tracks: Vec<AudioTrack> = Vec::new();

    let supported_extensions: [&str; 1] = ["flac"];

    // base capacity is arbitrary in size
    // don't need to optimise too greatly (pay this once), but don't want to spam realloc either
    let mut dirs: Vec<PathBuf> = Vec::with_capacity(256);
    dirs.push(directory);

    // iteratively loop through given directory
    // if item is directory, iterate over its children, pushing all to vec
    // if item is file, check extension is supported then pass off to build track
    loop {
        match dirs.pop() {
            Some(path) => {
                if path.is_dir() {
                    for entry in path.read_dir().unwrap() {
                        match entry {
                            Ok(i) => {
                                dirs.push(i.path());
                            }
                            Err(e) => eprintln!("{e}")
                        }
                    }
                } else if path.is_file() {
                    // prefer a more robust solution, but this has advantage of requiring handle
                    match path.extension() {
                        Some(p) => {
                            if supported_extensions.contains(&p.to_str().expect("ext to str")) {
                                match read_audio_file(path.as_path()) {
                                    Ok(ok) => tracks.push(ok),
                                    Err(e) => eprintln!("{e}")
                                }
                            }
                        }
                        None => continue,
                    }
                }
            }
            None => break,
        }
    }

    // compile library from vec of tracks
    let mut artistc: ArtistCollection = HashMap::with_capacity(128);
    
    for trk in tracks.iter() {
        // below does not perform explicit handling for absent metadata
        // e.g., no album_artist tag on file, thus it is left blank, and so is the entry here
        // it is preferable that this case is handled. todo
        match trk {
            AudioTrack::Full(i_t) => {
                // entry instead? but it consumes string. shouldn't be a problem but is
                // todo: review hash_map source to determine material difference
                match artistc.get_mut(&i_t.album_artist) {
                    Some(i_aa) => {
                        // check for album
                        match i_aa.into_iter().find(|a| a.name == i_t.album) {
                            Some(i_a) => {
                                i_a.tracks.push(AudioTrack::Full(i_t.clone())); // todo: investigate safe move instead of clone
                            }
                            None => {
                                // the artist exists, but not this album
                                // push new album to vec with track
                                i_aa.push(
                                    Album {
                                        name: String::from(&i_t.album),
                                        artist: String::from(&i_t.album_artist),
                                        tracks: vec![AudioTrack::Full(i_t.clone())]
                                    }
                                );
                            }
                        }
                    },
                    None => {
                        // no entry for album_artist
                        // push new to vec and initialise album
                        artistc.insert(
                            String::from(&i_t.album_artist),
                            vec![Album {
                                name: String::from(&i_t.album),
                                artist: String::from(&i_t.artist),
                                tracks: vec![AudioTrack::Full(i_t.clone())]
                            }]
                        );
                    }
                }
            }
            AudioTrack::Limited(i_t) => {
                // search for default entry
                // if absent, create
                match artistc.get_mut("no_artist") {
                    Some(i_aa) => {
                        // if we hit this, we know the default album exists as well
                        match i_aa.first_mut() {
                            // push(AudioTrack::Limited(i_t.clone())),
                            Some(i_a) => i_a.tracks.push(AudioTrack::Limited(i_t.clone())),
                            None => unreachable!("no_artist exists with no album") // prefer assert or handle
                        }
                    }
                    None => {
                        // initialise default 'no_artist' entry
                        artistc.insert(
                            String::from("no_artist"),
                            vec![Album {
                                name: String::from("default"),
                                artist: String::from("no_artist"),
                                tracks: vec![AudioTrack::Limited(i_t.clone())]
                            }]
                        );
                    }
                }
            }
        }
    }

    artistc
}

/// Takes a string slice representing path to an audio file as input, then reads the file and
/// attempts to convert create a representative Track from the audio file
fn read_audio_file(path: &Path) -> std::result::Result<AudioTrack, SymphoniaError> {
    let source = Box::new(File::open(path).expect("box file error"));
    let mss = MediaSourceStream::new(source, Default::default());

    let mut hint = Hint::new();
    hint.with_extension(
        Path::new(path)
            .extension()
            .expect("hint extension")
            .to_str()
            .expect("hint to str"),
    );

    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();

    let mut probe = match get_probe().format(&hint, mss, &fmt_opts, &meta_opts) {
        Ok(p) => p,
        Err(e) => return Err(e),
    };

    if let Some(meta) = probe.format.metadata().current() {
        let tags = meta.tags();
        if !tags.is_empty() {
            Ok(AudioTrack::Full(build_track_with_metadata(path, tags)))
        } else {
            Ok(AudioTrack::Limited(build_track_without_metadata(path)))
        }
    } else if let Some(meta) = probe.metadata.get().as_ref().and_then(|m| m.current()) {
        let tags = meta.tags();
        if !tags.is_empty() {
            Ok(AudioTrack::Full(build_track_with_metadata(path, tags)))
        } else {
            Ok(AudioTrack::Limited(build_track_without_metadata(path)))
        }
    } else {
        Ok(AudioTrack::Limited(build_track_without_metadata(path)))
    }
}

fn build_track_with_metadata(path: &Path, metadata: &[Tag]) -> FullAudioTrack {
    let mut track: FullAudioTrack = Default::default();

    track.path = path.to_string_lossy().into_owned();
    
    for tag in metadata.iter().filter(|t| t.is_known()) {
        if let Some(key) = tag.std_key {
            let key = format!("{:?}", key);
            match key.as_str() {
                "TrackTitle" => {
                    track.title = tag.value.to_string();
                },
                "Album" => {
                    track.album = tag.value.to_string();
                }
                "AlbumArtist" => {
                    track.album_artist = tag.value.to_string();
                }
                "Artist" => {
                    track.artist = tag.value.to_string();
                }
                "TrackNumber" => {
                    match tag.value.to_string().parse::<i32>() {
                        Ok(i) => track.track_num = i,
                        Err(e) => eprintln!("failed track build: {e}")
                    }
                }
                "TrackTotal" => {
                    match tag.value.to_string().parse::<i32>() {
                        Ok(i) => track.track_total = i,
                        Err(e) => eprintln!("failed track build: {e}")
                    }
                }
                "Date" => {
                    track.date = tag.value.to_string();
                }
                "Lyrics" => {
                    track.lyrics = tag.value.to_string();
                }
                _ => ()
            }
        }
    }
    track
}

fn build_track_without_metadata(path: &Path) -> LimitedAudioTrack {
    LimitedAudioTrack {
        path: path.to_string_lossy().into_owned(),
        title: path.file_name().expect("filename from path").to_string_lossy().into_owned()
    }
}

fn _check_cache() {
    todo!("check if cache lock has changed. if yes, reload, else load");
}

fn _cache_library() {
    todo!("cache current library");
}
