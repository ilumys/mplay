use std::{collections::VecDeque, fs::File, io::BufReader, rc::Rc};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

use crate::loader::AudioTrack;

pub struct Player {
    pub last_played: Option<Rc<AudioTrack>>, // think on implementation of this more
    sink: Sink,
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
    queue: VecDeque<Rc<AudioTrack>>, // remove element from queue on completion or not? and what smart pointer can I use here to avoid unsafe
}

impl Player {
    pub fn new() -> Self {
        // wrap in result and propagate
        let (stream, stream_handle) = OutputStream::try_default().expect("output stream init");
        let sink = Sink::try_new(&stream_handle).expect("sink init");

        Self {
            last_played: None,
            sink,
            _stream: stream,
            _stream_handle: stream_handle,
            queue: VecDeque::with_capacity(8),
        }
    }

    /// Appends a source to the sink, queuing it for playback
    pub fn append_queue(&mut self, track: Rc<AudioTrack>) {
        self.queue.push_back(track);
    }

    #[inline]
    pub fn try_next(&mut self) {
        if self.sink.empty() {
            self.play_from_queue();
        }
    }

    /// Plays the next track in the queue
    fn play_from_queue(&mut self) {
        match self.queue.pop_front() {
            Some(i) => match &*i {
                AudioTrack::Extended(a) => {
                    let file = BufReader::new(File::open(&a.path).expect("read audio file"));
                    let source = Decoder::new(file).expect("decode audio file");
                    self.sink.append(source);
                    self.last_played = Some(i.clone());
                }
                AudioTrack::Limited(a) => {
                    let file = BufReader::new(File::open(&a.path).expect("read audio file"));
                    let source = Decoder::new(file).expect("decode audio file");
                    self.sink.append(source);
                    self.last_played = Some(i.clone());
                }
            },
            None => self.last_played = None, // curious if this writes every time or `None` abstraction knows not to
        }
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
        self.queue.clear();
        self.last_played = None;
    }

    /// Skips to the next source in the sink queue
    pub fn skip_one(&mut self) {
        self.sink.clear();
        self.toggle_pause(); // `clear` pauses; set to play
        self.play_from_queue();
    }
}
