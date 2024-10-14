use std::path::Path;

struct Track {}

pub fn build_library() {
    // allow to be swapped from config option or file
    let directory = format!("{}/Music/", env!("HOME"));
    let dir = Path::new(&directory);

    let mut tracks: Vec<Track> = vec![];

    compile_library(&dir, &mut tracks);
    // recurse through directory
    // open all tracks with symphonia
    // if track codec is supported, add to library
    // if not, skip to next
}

fn compile_library(path: &Path, tracks: &mut Vec<Track>) {
    // change unwraps to handle all exceptions
    if path.is_dir() {
        for path in path.read_dir().unwrap() {
            let path = path.unwrap().path();
            if path.is_dir() {
                compile_library(&path, tracks);
            } else if path.is_file() {
                if let Some(track) = process_track(&path) {
                    tracks.push(track);
                }
            }
        }
    }
}

fn process_track(path: &Path) -> Option<Track> {
    todo!("compile track meta/data");
}

fn _check_cache() {
    todo!("check if cache lock has changed. if yes, reload, else load");
}

fn _cache_library() {
    todo!("cache current library");
}
