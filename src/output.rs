use symphonia::core::{
    audio::{AudioBufferRef, SignalSpec},
    units,
};

pub trait AudioOutput {
    fn write(&mut self, decoded: AudioBufferRef<'_>) -> Result<()>;
    fn flush(&mut self);
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum AudioOutputError {
    OpenStreamError,
    PlayStreamError,
    StreamClosedError,
}

pub type Result<T> = std::result::Result<T, AudioOutputError>;

// audio engineering things ... worthwhile to study in more depth
// relying on implementation references from symphonia for this
mod pulseaudio {
    use super::{AudioOutput, AudioOutputError, Result};

    use libpulse_binding as libpulse;
    use libpulse_simple_binding as libpulse_simple;
    use symphonia::core::{
        audio::{self, AudioBufferRef, Channels},
        units,
    };

    pub struct PulseAudioOutput {
        pulse: libpulse_simple::Simple,
        buffer: audio::RawSampleBuffer<f32>,
    }

    impl PulseAudioOutput {
        pub fn open(
            spec: audio::SignalSpec,
            duration: units::Duration,
        ) -> Result<Box<dyn AudioOutput>> {
            let buffer = audio::RawSampleBuffer::<f32>::new(duration, spec);
            let pulse_spec = libpulse::sample::Spec {
                format: libpulse::sample::Format::FLOAT32NE,
                channels: spec.channels.count() as u8,
                rate: spec.rate,
            };
            assert!(pulse_spec.is_valid());

            let pulse_channel_map = map_channels(spec.channels);

            let pulse_connection = libpulse_simple::Simple::new(
                None,                                  // use default server
                "fy",                                  // client name
                libpulse::stream::Direction::Playback, // stream is playback
                None,                                  // use default device
                "fy music player",                     // descriptive name for stream
                &pulse_spec,                           // sample type / signal spec
                pulse_channel_map.as_ref(),            // channel map
                None,                                  // default buffering attributes
            );

            match pulse_connection {
                Ok(pulse) => return Ok(Box::new(PulseAudioOutput { pulse, buffer })),
                Err(e) => {
                    eprintln!("audio output stream open error: {}", e);
                    return Err(AudioOutputError::OpenStreamError);
                }
            }
        }
    }

    impl AudioOutput for PulseAudioOutput {
        fn write(&mut self, decoded: AudioBufferRef<'_>) -> Result<()> {
            if decoded.frames() == 0 {
                return Ok(());
            }

            self.buffer.copy_interleaved_ref(decoded);

            match self.pulse.write(self.buffer.as_bytes()) {
                Err(e) => {
                    eprintln!("audio output stream write error: {}", e);
                    return Err(AudioOutputError::StreamClosedError);
                }
                _ => return Ok(()),
            }
        }

        fn flush(&mut self) {
            // performance is best-effort
            // take the result and ignore it
            self.pulse.drain().unwrap();
        }
    }

    // map symphonia channels to a pulseaudio channel map
    fn map_channels(channels: Channels) -> Option<libpulse::channelmap::Map> {
        let mut map: libpulse::channelmap::Map = Default::default();
        map.init();
        map.set_len(channels.count() as u8);
        let is_mono = channels.count() == 1;

        for (index, channel) in channels.iter().enumerate() {
            map.get_mut()[index] = match channel {
                Channels::FRONT_LEFT if is_mono => libpulse::channelmap::Position::Mono,
                Channels::FRONT_LEFT => libpulse::channelmap::Position::FrontLeft,
                Channels::FRONT_RIGHT => libpulse::channelmap::Position::FrontRight,
                Channels::FRONT_CENTRE => libpulse::channelmap::Position::FrontCenter,
                Channels::REAR_LEFT => libpulse::channelmap::Position::RearLeft,
                Channels::REAR_CENTRE => libpulse::channelmap::Position::RearCenter,
                Channels::REAR_RIGHT => libpulse::channelmap::Position::RearRight,
                Channels::LFE1 => libpulse::channelmap::Position::Lfe,
                Channels::FRONT_LEFT_CENTRE => libpulse::channelmap::Position::FrontLeftOfCenter,
                Channels::FRONT_RIGHT_CENTRE => libpulse::channelmap::Position::FrontRightOfCenter,
                Channels::SIDE_LEFT => libpulse::channelmap::Position::SideLeft,
                Channels::SIDE_RIGHT => libpulse::channelmap::Position::SideRight,
                Channels::TOP_CENTRE => libpulse::channelmap::Position::TopCenter,
                Channels::TOP_FRONT_LEFT => libpulse::channelmap::Position::TopFrontLeft,
                Channels::TOP_FRONT_CENTRE => libpulse::channelmap::Position::TopFrontCenter,
                Channels::TOP_FRONT_RIGHT => libpulse::channelmap::Position::TopFrontRight,
                Channels::TOP_REAR_LEFT => libpulse::channelmap::Position::TopRearLeft,
                Channels::TOP_REAR_CENTRE => libpulse::channelmap::Position::TopRearCenter,
                Channels::TOP_REAR_RIGHT => libpulse::channelmap::Position::TopRearRight,
                _ => {
                    // if the symphonia channel cannot map to a pulseaudio position
                    eprintln!("libpulse: failed to map channel {:?} to output", channel);
                    return None;
                }
            }
        }
        Some(map)
    }
}

pub fn open(spec: SignalSpec, duration: units::Duration) -> Result<Box<dyn AudioOutput>> {
    pulseaudio::PulseAudioOutput::open(spec, duration)
}
