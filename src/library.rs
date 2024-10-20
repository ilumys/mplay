use std::{fs::File, path::Path};

use symphonia::{
    core::{formats::FormatOptions, io::MediaSourceStream, meta::MetadataOptions, probe::Hint},
    default::get_probe,
};

struct Track {
    path: String,
    title: String,
    album: String,
    album_artist: String,
    artist: String,
    lyrics: String,
    duration: f32,
}

pub fn build_library() {
    // add option for dir in args or file
    let path = format!("{}/Music/", env!("HOME"));
    let mut tracks: Vec<Track> = vec![];
    compile_library(&path, &mut tracks);
}

fn compile_library(path: &str, tracks: &mut Vec<Track>) {
    let mut dirs: Vec<String> = vec![path.to_string()];
    // add a check is_audio_file to avoid scanning everything needlessly
    loop {
        match dirs.pop() {
            Some(path_t) => {
                let path_i = Path::new(path_t.as_str());
                if path_i.is_file() {
                    if let Some(track) = process_track(path) {
                        tracks.push(track);
                    }
                } else if path_i.is_dir() {
                    for path_i in path_i.read_dir().unwrap() {
                        let path_i = path_i.unwrap().path();
                        if path_i.is_dir() {
                            dirs.push(String::from(path_i.to_str().unwrap()));
                        } else if path_i.is_file() {
                            if let Some(track) = process_track(path) {
                                tracks.push(track);
                            }
                        }
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
    let mut metadata: [String; 5] = Default::default();
    if let Some(meta) = probe.format.metadata().current() {
        let tags = meta.tags();
        if !tags.is_empty() {
            for tag in tags.iter().filter(|t| t.is_known()) {
                if let Some(key) = tag.std_key {
                    let key = format!("{:?}", key);
                    match key.as_str() {
                        "TrackTitle" => metadata[0] = key,
                        "Album" => metadata[1] = key,
                        "AlbumArtist" => metadata[2] = key,
                        "Artist" => metadata[3] = key,
                        "Lyrics" => metadata[4] = key,
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
                        "TrackTitle" => metadata[0] = key,
                        "Album" => metadata[1] = key,
                        "AlbumArtist" => metadata[2] = key,
                        "Artist" => metadata[3] = key,
                        "Lyrics" => metadata[4] = key,
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
        duration: 0.0,
    });
}

fn _check_cache() {
    todo!("check if cache lock has changed. if yes, reload, else load");
}

fn _cache_library() {
    todo!("cache current library");
}
