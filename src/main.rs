use std::io::BufReader;
use std::{fs::File, path::Path};

use crossterm::event::{self, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Layout, Margin},
    style::Stylize,
    widgets::{block::Title, Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

//mod metadata;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let terminal = ratatui::init();
    run(terminal, args.get(1).expect("expected file path"));
    ratatui::restore();
}

fn run(mut term: ratatui::DefaultTerminal, arg: &str) {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().expect("stream error");
    let sink = rodio::Sink::try_new(&stream_handle).expect("sink error");

    // printing is very bad
    // each line starts at last column of previous line
    let path = Path::new(arg);
    // need to return metadata from this fn, to populate tui frame
    selecter(path, &sink);

    // want to track state of artists and albums, for scroll and other display
    // also want to make artists and albums into a struct, with strong/weak references to each other

    // add a button
    // selecting the button appends a song
    let _result = loop {
        let pr = format!("list of artists");
        let pl = format!("list of albums");
        term.draw(|frame| {
            let layout = Layout::vertical([
                Constraint::Length(3),
                Constraint::Percentage(90),
                Constraint::Min(6),
            ])
            .split(frame.area());

            let library =
                Layout::horizontal([Constraint::Percentage(28), Constraint::Percentage(72)])
                    .split(layout[1]);

            // print current dir in search?
            let search = Block::bordered().title(Title::from("search".bold()));

            let artists = Paragraph::new(pr).block(Block::bordered().title("artists".bold()));
            let albums = Paragraph::new(pl).block(Block::bordered().title("albums".bold()));

            // how to preload library?
            let playing = Block::bordered().title(Title::from("playing".bold()));

            frame.render_widget(search, layout[0]);
            frame.render_widget(artists, library[0]);
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight),
                library[0].inner(Margin {
                    horizontal: 0,
                    vertical: 1,
                }),
                &mut ScrollbarState::new(28).position(0),
            );
            frame.render_widget(albums, library[1]);
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight),
                library[1].inner(Margin {
                    horizontal: 0,
                    vertical: 1,
                }),
                &mut ScrollbarState::new(28).position(0),
            );
            frame.render_widget(playing, layout[2]);
        })
        .unwrap();

        if let event::Event::Key(key) = event::read().unwrap() {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char(' ') => {
                        if sink.is_paused() {
                            sink.play();
                        } else {
                            sink.pause();
                        }
                    }
                    KeyCode::Char('q') => return,
                    _ => continue,
                }
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
    /*
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
    }*/
    let data = BufReader::new(File::open(path).expect("bufreader error"));
    let source = rodio::Decoder::new(data).expect("decode error");
    sink.append(source);
}
