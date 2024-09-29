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
        if path.try_exists().expect("cannot read file") {
            media::run_track(path);
        } else {
            println!("unable to play track");
        }
    }
}
