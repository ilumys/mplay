//use std::io::BufReader;
//use std::{fs::File, path::Path};

use ratatui::{
    crossterm::event::{self, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Margin, Rect},
    style::Stylize,
    widgets::{block::Title, Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

pub fn run(mut term: ratatui::DefaultTerminal) {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().expect("stream error");
    let sink = rodio::Sink::try_new(&stream_handle).expect("sink error");

    // printing is very bad
    // each line starts at last column of previous line
    //let path = Path::new(arg);
    // need to return metadata from this fn, to populate tui frame
    //selecter(path, &sink);

    // want to track state of artists and albums, for scroll and other display
    // also want to make artists and albums into a struct, with strong/weak references to each other

    // add a button
    // selecting the button appends a song
    let _result = loop {
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

            let search = Block::bordered().title(Title::from("search".bold()));
            let playing = Paragraph::new("track").block(Block::bordered().title("playin".bold()));

            frame.render_widget(search, layout[0]);
            draw_artists(frame, library[0]);
            draw_albums(frame, library[1]);

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

fn draw_artists(frame: &mut Frame, container: Rect) {
    let pr = format!("list of artists");
    let artists = Paragraph::new(pr).block(Block::bordered().title("artists".bold()));

    frame.render_widget(artists, container);
    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight),
        container.inner(Margin {
            horizontal: 0,
            vertical: 1,
        }),
        &mut ScrollbarState::new(28).position(0),
    );
}

fn draw_albums(frame: &mut Frame, container: Rect) {
    let pl = format!("list of albums");
    let albums = Paragraph::new(pl).block(Block::bordered().title("albums".bold()));

    frame.render_widget(albums, container);
    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight),
        container.inner(Margin {
            horizontal: 0,
            vertical: 1,
        }),
        &mut ScrollbarState::new(28).position(0),
    );
}
