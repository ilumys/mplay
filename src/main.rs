use std::io::BufReader;
use std::{fs::File, path::Path};

use ratatui::crossterm::event::{self, KeyCode, KeyEventKind};

mod metadata;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let terminal = ratatui::init();
    run(terminal, args.get(1).expect("expected file path"));
}

fn run(mut term: ratatui::DefaultTerminal, arg: &str) {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().expect("stream error");
    let sink = rodio::Sink::try_new(&stream_handle).expect("sink error");

    // printing is very bad
    // each line starts at last column of previous line
    let path = Path::new(arg);
    selecter(path, &sink);

    let _result = loop {
        let p = format!("playing: {arg}");
        term.draw(|frame| {
            let msg = ratatui::widgets::Paragraph::new(p);
            frame.render_widget(msg, frame.area());
        })
        .unwrap();

        if let event::Event::Key(key) = event::read().unwrap() {
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('p') {
                if sink.is_paused() {
                    sink.play();
                } else {
                    sink.pause();
                }
            }
            if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return;
            }
        }
    };
}

fn selecter(path: &Path, sink: &rodio::Sink) {
    if path.is_dir() {
        for file in path.read_dir().expect("dir error") {
            let file = file.expect("file extraction error").path();
            if file.is_dir() {
                selecter(&file, sink);
            } else if file.is_file() {
                player(&file, sink);
            }
        }
    } else if path.is_file() {
        player(&path, sink);
    }
}

fn player(path: &Path, sink: &rodio::Sink) {
    let supported = ["flac"];
    // ridiculous
    if let Some(ext) = path.extension() {
        if let Some(str) = ext.to_str() {
            if !supported.contains(&str) {
                return;
            }
        }
    } else {
        return;
    }

    let keys = [
        "TrackTitle",
        "Album",
        "AlbumArtist",
        "Artist",
        "TrackNumber",
        "TrackTotal",
        "Date",
    ];

    if let Some(tags) = metadata::get_tags(path) {
        for tag in tags.iter() {
            if keys.contains(&tag.0.as_str()) {
                println!("{}: {}", tag.0, tag.1);
            }
        }
    }
    let data = BufReader::new(File::open(path).expect("bufreader error"));
    let source = rodio::Decoder::new(data).expect("decode error");
    sink.append(source);
}
