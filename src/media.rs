use std::{fs::File, path::Path};
use symphonia::core::{
    codecs::DecoderOptions,
    errors,
    formats::{FormatOptions, FormatReader},
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
};

use crate::output;

// read the provided track, get hints and options, then play
pub fn run_track(path: &Path) {
    let mut hint = Hint::new();

    if let Some(extension) = path.extension() {
        if let Some(ext_str) = extension.to_str() {
            hint.with_extension(ext_str);
        }
    }

    let source = Box::new(File::open(path).expect("cannot open file"));
    let mss = MediaSourceStream::new(source, Default::default());
    let format_opts = FormatOptions {
        enable_gapless: true,
        ..Default::default()
    };
    let metadata_opts: MetadataOptions = Default::default();

    // probe mss for metadata and get format reader
    match symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts) {
        Ok(mut probe) => {
            let decode_opts = DecoderOptions { verify: true };
            play_track(&mut probe.format, &decode_opts).unwrap();
        }
        Err(err) => panic!("error code: {}", err),
    }
}

// playback a provided track
fn play_track(
    reader: &mut Box<dyn FormatReader>,
    decode_opts: &DecoderOptions,
) -> errors::Result<()> {
    // media formats may contain more than one track in bitstream
    // need to account for this or will cause unhappiness
    // todo: pass this up as an option
    let track = reader.default_track().unwrap();
    let track_id = track.id;

    // note the mut. this will change, obviously
    let mut audio_output: Option<Box<dyn output::AudioOutput>> = None;

    // create track decoder
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, decode_opts)
        .unwrap();

    // get base time for track and elapsed duration, for display purposes
    //let time_base = track.codec_params.time_base;
    //let duration = track.codec_params.n_frames.unwrap() + track.codec_params.start_ts;

    // errors may propagate out; retrieve them
    let result = loop {
        // todo: recover from ResetRequired
        let packet = match reader.next_packet() {
            Ok(pkt) => pkt,
            Err(e) => break Err(e),
        };

        // ensure packet belongs to current track
        if packet.track_id() != track_id {
            continue;
        }

        // decode packet to audio samples
        match decoder.decode(&packet) {
            Ok(decode) => {
                if let Some(audio_output) = audio_output.as_mut() {
                    audio_output.write(decode).unwrap();
                } else {
                    let spec = *decode.spec();
                    let duration = decode.capacity() as u64;
                    audio_output.replace(output::open(spec, duration).unwrap());
                }
            }
            Err(errors::Error::DecodeError(e)) => {
                // non-fatal error, print and continue
                eprintln!("decode error: {}", e);
            }
            Err(e) => break Err(e),
        }
    };

    if let Some(audio_output) = audio_output.as_mut() {
        audio_output.flush();
    }

    match result {
        Err(errors::Error::IoError(e))
            if e.kind() == std::io::ErrorKind::UnexpectedEof
                && e.to_string() == "end of stream" =>
        {
            return Ok(())
        }
        _ => return result,
    }
}
