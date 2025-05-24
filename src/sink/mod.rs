use std::{fs::File, path::PathBuf, result};

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

// implement playback through a method which spawns a thread and runs `play_audio`?
// should this control the queue or should that be a layer up?
pub struct Sink {
    // need a handle!
    // controls: play, pause, skip, seek, clear, ..
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

impl Sink {
    pub async fn append(path: &str) {
        // handle things?
    }
}

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
        // TODO: recover from `ResetRequired`
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
        Err(Error::IoError(e))
            if e.kind() == std::io::ErrorKind::UnexpectedEof
                && e.to_string() == "end of stream" =>
        {
            Ok(())
        }
        _ => Err(SinkError::UnrecoverableError),
    }
}
