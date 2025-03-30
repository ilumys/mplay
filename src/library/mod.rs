//! Scans for and builds representative tracks for all audio files in a given directory
//!
//! Once the initial read is completed, groups tracks by artist and album for representation in UI
//!
//! TODO: cache

use std::{
    //collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
};

use symphonia::{
    core::{
        errors::Error as SymphoniaError, formats::FormatOptions, io::MediaSourceStream,
        meta::MetadataOptions, meta::Tag, probe::Hint,
    },
    default::get_probe,
};

mod player;

pub(crate) use player::Player;

/// Audio track with extended metadata present
#[derive(Default)]
pub struct FullAudioTrack {
    /// File path
    pub path: String,
    /// Track title
    pub title: String,
    // album: &Album,
    // artist: &Artist,
    /// Date of track release
    date: String,
    /// Track lyrics
    lyrics: String, // add visuals ?
                    // duration
                    // track num ?
}

/// Audio track with no detected metadata
pub struct LimitedAudioTrack {
    /// File path
    pub path: String,
    /// Track title, assumed from the file name
    pub title: String,
}

/// Base unit representing an audio file with accompanying metadata for playback
/// Each instance is 113 bytes and 8-byte aligned
///
/// Variants:
/// - Full: where extended audio metadata is detected
/// - Limited: where no additional metadata is found
pub enum AudioTrack {
    Extended(FullAudioTrack),
    Limited(LimitedAudioTrack),
}

impl AudioTrack {
    fn new_full(path: &Path, metadata: &[Tag]) -> Self {
        let mut track: FullAudioTrack = Default::default();

        track.path = path.to_string_lossy().into_owned();

        for tag in metadata.iter().filter(|t| t.is_known()) {
            if let Some(key) = tag.std_key {
                let key = format!("{:?}", key);
                match key.as_str() {
                    "TrackTitle" => track.title = tag.value.to_string(),
                    "Date" => track.date = tag.value.to_string(),
                    "Lyrics" => track.lyrics = tag.value.to_string(),
                    _ => (),
                }
            }
        }
        AudioTrack::Extended(track)
    }

    fn new_limited(path: &Path) -> Self {
        AudioTrack::Limited(LimitedAudioTrack {
            path: path.to_string_lossy().into_owned(),
            title: path
                .file_name()
                .expect("failed to extract filename from path")
                .to_string_lossy()
                .into_owned(),
        })
    }
}

/// Given a directory, take a vector of resulting `AudioTrack`s and group them by album and artist
/// On completion, returns an `ArtistList`, comprising the grouped tracks
pub fn build_library(directory: PathBuf) -> Box<[AudioTrack]> {
    let mut tracks: Vec<AudioTrack> = Vec::with_capacity(256);

    let supported_extensions: [&str; 1] = ["flac"];

    // base capacity is arbitrary in size
    // don't need to optimise too greatly (pay this once), but don't want to spam realloc either
    let mut dirs: Vec<PathBuf> = Vec::with_capacity(256);
    dirs.push(directory);

    // iterate through given directory
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
                            Err(e) => eprintln!("{e}"),
                        }
                    }
                } else if path.is_file() {
                    // a rather unscientific method for determining file type
                    // prefer a more robust solution, but this has advantage of requiring handle
                    match path.extension() {
                        Some(p) => {
                            if supported_extensions.contains(&p.to_str().expect("ext to str")) {
                                match read_audio_file(path.as_path()) {
                                    Ok(ok) => tracks.push(ok),
                                    Err(e) => eprintln!("{e}"),
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

    // convert to and return a boxed slice to make it obvious that this:
    // (a) is not expected to change, and;
    // (b) to save that tiny little bit on stack size
    return tracks.into_boxed_slice();
}

/// Takes a string slice representing path to an audio file as input, then reads the file and
/// attempts to convert create a representative Track from the audio file
fn read_audio_file(path: &Path) -> Result<AudioTrack, SymphoniaError> {
    let source = Box::new(File::open(path).expect("box file error"));
    let mss = MediaSourceStream::new(source, Default::default());

    let mut hint = Hint::new();
    hint.with_extension(
        Path::new(path)
            .extension()
            .expect("failed to extract file extension")
            .to_str()
            .expect("extension to str conversion failed"),
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
            Ok(AudioTrack::new_full(path, tags))
        } else {
            Ok(AudioTrack::new_limited(path))
        }
    } else if let Some(meta) = probe.metadata.get().as_ref().and_then(|m| m.current()) {
        let tags = meta.tags();
        if !tags.is_empty() {
            Ok(AudioTrack::new_full(path, tags))
        } else {
            Ok(AudioTrack::new_limited(path))
        }
    } else {
        Ok(AudioTrack::new_limited(path))
    }
}

// fn _check_cache() {
//     todo!("check if cache lock has changed. if yes, reload, else load");
// }

// fn _cache_library() {
//     todo!("cache current library");
// }
