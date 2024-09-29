use std::{env, path::Path};

mod media;
mod output;

// todo: change all panic calls to an exit code
// or maybe something else? still deciding
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        println!("here we open ui")
    } else if args.len() > 1 {
        println!("track: {}", args[1]);
        let path = Path::new(&args[1]);
        if path.is_dir() {
            for file in path.read_dir().expect("unable to read dir") {
                if let Ok(track) = file {
                    println!("current track: {}", &track.path().display());
                    media::run_track(&track.path());
                }
            }
        } else if path.is_file() {
            media::run_track(path);
        }
    }
}
