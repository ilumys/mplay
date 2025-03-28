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

// struct Album {
//     artist: String,
//     name: String,
//     num_tracks: i32,
//     tracks: Vec<&AudioTrack>
//       identifier for track number
// }

// struct Artist {
//     name: String,
//     albums: Vec<Album>,
//     tracks: Vec<&AudioTrack>
// }

/// Audio track with extended metadata present
///
/// An AudioTrack is either known or unknown, and this is determined by the presence of metadata in the audio file
/// If it is known, it SHOULD belong to an album, and it SHOULD have an artist
/// If it is unknown, then it will only store enough data to identify and locate it
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FullAudioTrack {
    // four `String`s, and two references: 24x4=96 + 8x2=16 = 112 bytes
    path: String,
    pub title: String,
    // album: &Album,
    // artist: &Artist,
    date: String,
    lyrics: String, // add visuals ?
}

/// Audio track with no detected metadata
#[derive(Clone, Debug, PartialEq)]
pub struct LimitedAudioTrack {
    // 56 bytes
    path: String,
    pub title: String,
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

pub type TrackList = Vec<AudioTrack>;

/// Given a directory, take a vector of resulting `AudioTrack`s and group them by album and artist
/// On completion, returns an `ArtistList`, comprising the grouped tracks
pub fn build_library(directory: PathBuf) -> TrackList {
    let mut tracks: Vec<AudioTrack> = Vec::new();

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
    tracks
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
                "TrackTitle" => track.title = tag.value.to_string(),
                "Date" => track.date = tag.value.to_string(),
                "Lyrics" => track.lyrics = tag.value.to_string(),
                // "TrackNumber" => match tag.value.to_string().parse::<i32>() {
                //     Ok(i) => track.track_num = i,
                //     Err(e) => {
                //         // indicative that a tag has been incorrectly set on the audio file
                //         eprintln!("failed track build: {e}");
                //         track.track_num = 0;
                //     }
                // },
                // "TrackTotal" => match tag.value.to_string().parse::<i32>() {
                //     Ok(i) => track.track_total = i,
                //     Err(e) => {
                //         eprintln!("failed track build: {e}");
                //         track.track_total = 0;
                //     }
                // },
                // "Album" => {
                //     track.album = tag.value.to_string();
                // }
                // "AlbumArtist" => {
                //     track.album_artist = tag.value.to_string();
                // }
                // "Artist" => {
                //     track.artist = tag.value.to_string();
                // }
                _ => (),
            }
        }
    }
    track
}

fn build_track_without_metadata(path: &Path) -> LimitedAudioTrack {
    LimitedAudioTrack {
        path: path.to_string_lossy().into_owned(),
        title: path
            .file_name()
            .expect("build_track_without_metadata: filename from path")
            .to_string_lossy()
            .into_owned(),
    }
}

fn _check_cache() {
    todo!("check if cache lock has changed. if yes, reload, else load");
}

fn _cache_library() {
    todo!("cache current library");
}
