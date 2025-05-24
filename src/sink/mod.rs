use std::{
    fs::File,
    path::PathBuf,
    result,
    sync::{
        Arc, Mutex,
        mpsc::{Sender, channel},
    },
    thread::{self, JoinHandle},
};

use symphonia::{
    core::{
        audio::AudioBufferRef,
        codecs::DecoderOptions,
        errors::{self, Error},
        formats::{FormatOptions, FormatReader},
        io::MediaSourceStream,
        meta::MetadataOptions,
        probe::Hint,
    },
    default::{get_codecs, get_probe},
};

#[cfg(target_os = "linux")]
mod linux;

// this may need to be set per os as well, unless available libs are matching
const SUPPORTED_CODECS: [&str; 1] = ["flac"];

pub struct Sink {
    // controls: play, pause, skip, seek, clear, ..
    handle: JoinHandle<Result<()>>,
    audio_tx: Sender<PathBuf>,
    // pause_tx: Sender<bool>, // stub for testing
    // bitflags to indicate action to take? but some need to be updated
    is_empty: Arc<Mutex<bool>>,
    is_paused: Arc<Mutex<bool>>,
}

pub enum SinkError {
    ExtensionError,
    NoDefaultTrack,
    UnrecoverableError, // better name? can't do much with this at present
}

type Result<T, E = SinkError> = result::Result<T, E>;

pub trait AudioSink {
    fn write(&mut self, decoded: AudioBufferRef) -> Result<()>;
    fn flush(&mut self);
}

// need to think on desing more. mutexes here are a stub; prefer channels
impl Sink {
    pub fn new() -> Self {
        let (tx, rx) = channel::<PathBuf>();
        // let (ptx, prx) = channel::<bool>();
        let is_empty = Arc::new(Mutex::new(false));
        let is_paused = Arc::new(Mutex::new(false));

        let empty = Arc::clone(&is_empty);
        // let paused = Arc::clone(&is_paused);
        // move this logic elsewhere?
        let handle: JoinHandle<Result<()>> = thread::spawn(move || {
            loop {
                // detect if something is already playing, blocking to wait if not
                // how to tell `Sink` that something is now playing without shared state?
                if *empty.lock().expect("unlock `empty` failed") {
                    if let Ok(k) = rx.recv() {
                        match read_audio_file(&k) {
                            Ok(_) => return Ok(()),
                            Err(_) => todo!("handle errors in playback thread"),
                        }
                    }
                }

                // the below needs to be pushed down to where bitstream playback

                // pass channels through to audio_play to track this and pause state?
                // if let Ok(i) = arx.try_recv() {}

                // // poll for pause being set; block until unset if it is
                // //
                // // this block can be simplified in rust 1.88 with stabilisation
                // // of if-let chains
                // if let Ok(i) = prx.try_recv() {
                //     if i {
                //         while let Ok(k) = prx.recv() {
                //             if !k {
                //                 break;
                //             }
                //         }
                //     }
                // }
            }
        });

        Sink {
            handle,
            audio_tx: tx,
            // pause_tx: ptx,
            is_empty,
            is_paused,
        }
    }

    pub fn wait_for_close(self) {
        self.handle.join().expect("thread termination");
    }

    pub fn append(&self, path: &str) {
        self.audio_tx.send(path.into());
    }

    pub fn toggle_pause(&mut self) {}
}

// consider an API that supports using and passing streams of bytes which
// provide audio playback and may queue

fn read_audio_file(path: &PathBuf) -> Result<()> {
    let mut hint = Hint::new();

    match path.to_str() {
        Some(e) => {
            hint.with_extension(e);
        }
        None => return Err(SinkError::ExtensionError),
    }

    let source = Box::new(File::open(path).expect("failed to open audio file"));
    let mss = MediaSourceStream::new(source, Default::default());
    let meta_opts = MetadataOptions {
        ..Default::default()
    };
    let fmt_opts = FormatOptions {
        enable_gapless: true,
        ..Default::default()
    };

    match get_probe().format(&hint, mss, &fmt_opts, &meta_opts) {
        Ok(mut probe) => {
            let decode_opts = DecoderOptions { verify: true };
            play_audio(&mut probe.format, &decode_opts)
        }
        Err(_) => todo!("handle symphonia errors"),
    }
}

fn play_audio(
    format_reader: &mut Box<dyn FormatReader>,
    decoder_opts: &DecoderOptions,
) -> Result<()> {
    // this does not account for streams with multiple tracks
    let (track, track_id) = match format_reader.default_track() {
        Some(t) => (t, t.id),
        None => return Err(SinkError::NoDefaultTrack),
    };

    let mut audio_output: Option<Box<dyn AudioSink>> = None;
    let mut decoder = get_codecs()
        .make(&track.codec_params, decoder_opts)
        .expect("codec not found");

    // symphonia does not indicate success in any straight-forward manner; only
    // an `end of stream` (from `IoError`) indicates this
    let result: errors::Result<()> = loop {
        let pkt = match format_reader.next_packet() {
            Ok(p) => p,
            Err(e) => match e {
                symphonia::core::errors::Error::ResetRequired => {
                    todo!("recover from ResetRequired")
                }
                _ => break Err(e),
            },
        };

        if pkt.track_id() != track_id {
            continue;
        }

        match decoder.decode(&pkt) {
            Ok(i) => {
                if let Some(n) = audio_output.as_mut() {
                    n.write(i);
                }
            }
            Err(e) => match e {
                Error::DecodeError(_) => todo!("handle decode error"),
                _ => break Err(e),
            },
        }
    };

    if let Some(i) = audio_output.as_mut() {
        i.flush();
    }

    match result {
        Ok(_) => Ok(()),
        Err(Error::IoError(e))
            if e.kind() == std::io::ErrorKind::UnexpectedEof
                && e.to_string() == "end of stream" =>
        {
            Ok(())
        }
        _ => Err(SinkError::UnrecoverableError),
    }
}
