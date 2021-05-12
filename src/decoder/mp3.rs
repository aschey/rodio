use std::time::Duration;
use std::{
    io::{Read, Seek},
    marker::PhantomData,
};

use symphonia::core::{
    audio::{AsAudioBufferRef, AudioBuffer, AudioBufferRef, RawSampleBuffer, SampleBuffer, Signal},
    codecs::DecoderOptions,
    conv::IntoSample,
    formats::{FormatOptions, FormatReader, Packet},
    io::{FiniteStream, MediaSource, MediaSourceStream},
    meta::MetadataOptions,
    probe::{Hint, ProbeResult},
    sample::Sample,
};

use crate::{source::SourceExt, Source};

pub struct Mp3Decoder {
    decoder: Box<dyn symphonia::core::codecs::Decoder>,
    current_frame: Packet,
    current_frame_offset: usize,
    probed: ProbeResult,
    buffer: SampleBuffer<i16>,
    channels: usize,
}

unsafe impl Send for Mp3Decoder {}

impl Mp3Decoder {
    pub fn new(mut probed: ProbeResult) -> Result<Self, ()> {
        //let mut hint = Hint::new();
        //let mss = MediaSourceStream::new(data as Box<dyn MediaSource>, Default::default());

        //let format_opts: FormatOptions = Default::default();
        //let metadata_opts: MetadataOptions = Default::default();
        // let mut probed = symphonia::default::get_probe()
        //     .format(&hint, mss, &format_opts, &metadata_opts)
        //     .unwrap();
        let stream = probed.format.default_stream().unwrap();

        let mut decoder = symphonia::default::get_codecs()
            .make(
                &stream.codec_params,
                &DecoderOptions {
                    verify: true,
                    ..Default::default()
                },
            )
            .unwrap();
        //let d = Box::new(*decoder as (dyn symphonia::core::codecs::Decoder + Send));
        let current_frame = probed.format.next_packet().unwrap();

        let decoded = decoder.decode(&current_frame).unwrap();
        let spec = decoded.spec().clone();
        let duration = symphonia::core::units::Duration::from(decoded.capacity() as u64);
        let mut buf = SampleBuffer::<i16>::new(duration, spec.to_owned());
        buf.copy_interleaved_ref(decoded);

        Ok(Mp3Decoder {
            decoder,
            current_frame,
            current_frame_offset: 0,
            probed,
            buffer: buf,
            channels: spec.channels.count(),
        })
    }
    pub fn into_inner(self) -> ProbeResult {
        self.probed
    }
}

impl Source for Mp3Decoder {
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.buffer.samples().len())
    }

    #[inline]
    fn channels(&self) -> u16 {
        self.channels as u16
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        self.probed
            .format
            .default_stream()
            .unwrap()
            .codec_params
            .sample_rate
            .unwrap()
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl Iterator for Mp3Decoder {
    type Item = i16;

    #[inline]
    fn next(&mut self) -> Option<i16> {
        if self.current_frame_offset == self.buffer.samples().len() {
            match self.probed.format.next_packet() {
                Ok(p) => {
                    self.current_frame = p;
                    let decoded = self.decoder.decode(&self.current_frame).unwrap();
                    let spec = decoded.spec();
                    let duration =
                        symphonia::core::units::Duration::from(decoded.capacity() as u64);
                    let mut buf = SampleBuffer::<i16>::new(duration, spec.to_owned());
                    buf.copy_interleaved_ref(decoded);
                    self.buffer = buf;
                }
                Err(_) => return None,
            };
            self.current_frame_offset = 0;
        }

        let s = self.buffer.samples()[self.current_frame_offset];

        self.current_frame_offset += 1;

        Some(s)
    }
}

impl SourceExt for Mp3Decoder {
    fn request_pos(&mut self, pos: f32) -> bool {
        // let pos = (pos * self.sample_rate() as f32) as u64;
        // self.decoder.seek_samples(pos).is_ok()
        false
    }
}
