use std::io::BufReader;
use std::{fs::File, path::Path};

use rodio::{Decoder, OutputStream, Sink};

mod metadata;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = Path::new(args.get(1).expect("expected file path"));

    println!("path: `{}`", path.to_str().unwrap());

    if let Some(tags) = metadata::get_tags(path) {
        for tag in tags.iter() {
            println!("{}: {}", tag.key, tag.value);
        }
    }

    let (_stream, stream_handle) = OutputStream::try_default().expect("stream error");
    let sink = Sink::try_new(&stream_handle).expect("sink error");

    let data = BufReader::new(File::open(path).unwrap());
    let source = Decoder::new(data).expect("decode error");

    sink.append(source);
    sink.sleep_until_end();
}
