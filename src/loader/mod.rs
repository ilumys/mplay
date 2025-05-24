//! shun_loader

use std::{
    //collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
    rc::Rc,
};

use symphonia::{
    core::{
        errors::Error as SymphoniaError, formats::FormatOptions, io::MediaSourceStream,
        meta::MetadataOptions, meta::Tag, probe::Hint,
    },
    default::get_probe,
};

// TODO: add a log file
// TODO: add caching

#[derive(Default)]
pub struct FullAudioTrack {
    /// Album the track belongs to
    pub album: String,
    /// Track artists
    pub artists: String,
    /// File path
    pub path: String,
    /// Track title
    pub title: String,
    // album: &Album,
    // artist: &Artist,
    /// Date of track release
    pub date: String,
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

pub struct AudioLibrary {
    pub tracks: LibraryCollection,
}

pub type LibraryCollection = Box<[Rc<AudioTrack>]>; // reference to slice? but it's heap, so.. doesn't matter?

// TODO: accept config file path, or use a default config file
// pub fn initialise_config(music_dir: Option<PathBuf>) {}

impl AudioTrack {
    fn new_full(path: &Path, metadata: &[Tag]) -> Self {
        let mut track: FullAudioTrack = Default::default();

        track.path = path.to_string_lossy().into_owned();

        for tag in metadata.iter().filter(|t| t.is_known()) {
            if let Some(key) = tag.std_key {
                let key = format!("{:?}", key);
                match key.as_str() {
                    "Album" => track.album = tag.value.to_string(),
                    "Artist" => track.artists = tag.value.to_string(),
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

impl AudioLibrary {
    /// Given a directory, take a vector of resulting `AudioTrack`s and group them by album and artist
    pub fn from_directory(directory: PathBuf) -> Self {
        let mut tracks: Vec<Rc<AudioTrack>> = Vec::with_capacity(256);

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
                                        Ok(ok) => tracks.push(Rc::new(ok)),
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

        Self {
            tracks: tracks.into_boxed_slice(),
        }
    }
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
