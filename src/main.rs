use std::io::BufReader;
use std::{fs::File, path::Path};

mod metadata;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = Path::new(args.get(1).expect("expected file path"));

    println!("path: `{}`", path.to_str().expect("path to str error"));
    selecter(path);
}

fn selecter(path: &Path) {
    if let Some(tags) = metadata::get_tags(path) {
        for tag in tags.iter() {
            println!("{}: {}", tag.key, tag.value);
        }
    }
    let (_stream, stream_handle) = rodio::OutputStream::try_default().expect("stream error");
    let sink = rodio::Sink::try_new(&stream_handle).expect("sink error");

    if path.is_dir() {
        for file in path.read_dir().expect("dir error") {
            let file = file.expect("file extraction error").path();
            if file.is_dir() {
                selecter(&file);
            } else if file.is_file() {
                player(&file, &sink);
            }
        }
    } else if path.is_file() {
        player(&path, &sink);
    }
}

fn player(path: &Path, sink: &rodio::Sink) {
    if let Some(tags) = metadata::get_tags(path) {
        for tag in tags.iter() {
            println!("{}: {}", tag.key, tag.value);
        }
    }
    let data = BufReader::new(File::open(path).expect("bufreader error"));
    let source = rodio::Decoder::new(data).expect("decode error");
    sink.append(source);
    sink.sleep_until_end();
}
