use std::{fs::File, io::BufReader};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

pub struct Player {
    sink: Sink,
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
}

impl Player {
    pub fn new() -> Self {
        // wrap in result and propagate
        let (stream, stream_handle) = OutputStream::try_default().expect("output stream init");
        let sink = Sink::try_new(&stream_handle).expect("sink init");

        Self {
            sink,
            _stream: stream,
            _stream_handle: stream_handle,
        }
    }

    /// Appends a source to the sink, queuing it for playback
    pub fn append(&mut self, path: &str) {
        let file = BufReader::new(File::open(path).expect("read audio file"));
        let source = Decoder::new(file).expect("decode audio file");

        self.sink.append(source);
    }

    /// Toggles the sink between a `play` and `pause` state
    pub fn toggle_pause(&mut self) {
        match self.sink.is_paused() {
            true => self.sink.play(),
            false => self.sink.pause(),
        }
    }

    /// Clears all sources from the sink and its queue, terminating playback
    pub fn clear_queue(&mut self) {
        self.sink.stop();
    }

    /// Skips to the next source in the sink queue
    pub fn next(&mut self) {
        self.sink.skip_one();
    }
}
