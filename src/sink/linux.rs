//! sink implementation on linux

use pulse::{
    channelmap::{Map, Position},
    sample::{Format, Spec},
    stream::Direction::Playback,
};
use pulse_simple::Simple;
use symphonia::core::{
    audio::{AudioBufferRef, Channels, RawSampleBuffer, SignalSpec},
    units::Duration,
};

use super::AudioSink;

// pipewire?

struct PulseAudio {
    pulse: pulse_simple::Simple,
    buffer: RawSampleBuffer<f32>,
}

impl PulseAudio {
    /// Creates a connection to the PulseAudio server
    fn open(
        spec: SignalSpec,
        duration: Duration,
        app_name: &str,
        stream_name: &str,
    ) -> super::Result<Box<dyn AudioSink>> {
        let buffer = RawSampleBuffer::<f32>::new(duration, spec);
        let pulse_spec = Spec {
            format: Format::FLOAT32NE,
            rate: spec.rate,
            channels: spec.channels.count() as u8,
        };
        debug_assert!(pulse_spec.is_valid());

        let ch_map = map_channels(spec.channels);
        let conn = Simple::new(
            None,
            app_name,
            Playback,
            None,
            stream_name,
            &pulse_spec,
            ch_map.as_ref(),
            None,
        );

        match conn {
            Ok(pulse) => return Ok(Box::new(PulseAudio { pulse, buffer })),
            Err(_) => todo!("handle pulseaudio errors"),
        }
    }
}

impl AudioSink for PulseAudio {
    /// Writes `decoded` to `self`s PulseAudio server
    fn write(&mut self, decoded: AudioBufferRef) -> super::Result<()> {
        if decoded.frames() == 0 {
            return Ok(());
        }

        self.buffer.copy_interleaved_ref(decoded);
        match self.pulse.write(self.buffer.as_bytes()) {
            Ok(_) => return Ok(()),
            Err(_) => todo!("handle pulseaudio errors"),
        }
    }

    /// Wait until all data is drained from the server then return, ignoring
    /// any errors
    fn flush(&mut self) {
        let _ = self.pulse.drain();
    }
}

/// Given a set of symphonia `Channels`, map them to PulseAudio `Position`s
fn map_channels(ch: Channels) -> Option<Map> {
    let mut map: Map = Default::default();
    map.init().set_len(ch.count() as u8);
    let is_mono = ch.count() == 1;

    for (i, ch) in ch.iter().enumerate() {
        map.get_mut()[i] = match ch {
            Channels::FRONT_LEFT if is_mono => Position::Mono,
            Channels::FRONT_LEFT => Position::FrontLeft,
            Channels::FRONT_RIGHT => Position::FrontRight,
            Channels::FRONT_CENTRE => Position::FrontCenter,
            Channels::LFE1 => Position::Lfe,
            Channels::REAR_LEFT => Position::RearLeft,
            Channels::REAR_CENTRE => Position::RearCenter,
            Channels::REAR_RIGHT => Position::RearRight,
            Channels::FRONT_LEFT_CENTRE => Position::FrontLeftOfCenter,
            Channels::FRONT_RIGHT_CENTRE => Position::FrontRightOfCenter,
            Channels::SIDE_LEFT => Position::SideLeft,
            Channels::SIDE_RIGHT => Position::SideRight,
            Channels::TOP_CENTRE => Position::TopCenter,
            Channels::TOP_FRONT_LEFT => Position::TopFrontLeft,
            Channels::TOP_FRONT_CENTRE => Position::TopFrontCenter,
            Channels::TOP_FRONT_RIGHT => Position::TopFrontRight,
            Channels::TOP_REAR_LEFT => Position::TopRearLeft,
            Channels::TOP_REAR_CENTRE => Position::TopRearCenter,
            Channels::TOP_REAR_RIGHT => Position::TopRearRight,
            _ => {
                // no remaining pulse channels that symphonia can map to
                // TODO: log reporting
                return None;
            }
        }
    }
    Some(map)
}
