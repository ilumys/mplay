//! Scans for and builds representative tracks for all audio files in a given directory
//!
//! Once the initial read is completed, groups tracks by artist and album for representation in TUI
//!
//! TODO: cache

use std::{fs::File, path::Path};

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

// consider: split AudioTrack into other file, turn below into method(s) for Library
// how to group AudioTrack::Known if it has missing tags?

pub struct Library {}

/// Given a directory, take a vector of resulting `AudioTrack`s and group them by album and artist
/// On completion, returns an `ArtistList`, comprising the grouped tracks
pub fn build_library(directory: &str) -> Library {
    let mut tracks: Vec<AudioTrack> = Vec::new();

    let supported_extensions: [&str; 1] = ["flac"];

    // base capacity is arbitrary in size
    // don't need to optimise too greatly (pay this once), but don't want to spam realloc either
    let mut dirs: Vec<&Path> = Vec::with_capacity(256);
    dirs.push(Path::new(directory));

    // iteratively loop through given directory
    loop {
        match dirs.pop() {
            Some(path) => {
                if path.is_dir() {
                    // todo: on error, write to stderr and continue
                    for entry in path.read_dir().expect("failed to read directory") {
                        dirs.push(
                            entry
                                .expect("failed to push entry to dirs")
                                .path()
                                .as_path(),
                        );
                    }
                } else if path.is_file() {
                    // prefer a more robust solution, but this has advantage of requiring handle
                    // scenarios to consider:
                    match path.extension() {
                        Some(p) => {
                            if supported_extensions.contains(&p.to_str().expect("ext to str")) {
                                // load track and check for metadata
                                match read_audio_file(path) {
                                    Ok(ok) => tracks.push(ok),
                                    Err(e) => eprintln!("{e}") // ignore and continue
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

    return;
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
