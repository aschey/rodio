//! Decodes samples from an audio file.

use std::fmt;
#[allow(unused_imports)]
use std::io::{Read, Seek, SeekFrom};
use std::mem;
use std::time::Duration;
use std::{error::Error, io};

use symphonia::core::{
    formats::{FormatOptions, SeekTo},
    io::{MediaSource, MediaSourceStream, ReadOnlySource},
    meta::MetadataOptions,
    probe::Hint,
    units::Time,
};

use crate::Source;

#[cfg(feature = "flac")]
mod flac;
#[cfg(feature = "mp3")]
mod mp3;
#[cfg(feature = "vorbis")]
mod vorbis;
#[cfg(feature = "wav")]
mod wav;

/// Source of audio samples from decoding a file.
///
/// Supports MP3, WAV, Vorbis and Flac.
pub struct Decoder<R>(DecoderImpl<R>)
where
    R: Read + Seek;

pub struct LoopedDecoder<R>(DecoderImpl<R>)
where
    R: Read + Seek;

enum DecoderImpl<R>
where
    R: Read + Seek,
{
    #[cfg(feature = "wav")]
    Wav(wav::WavDecoder<R>),
    #[cfg(feature = "vorbis")]
    Vorbis(vorbis::VorbisDecoder<R>),
    #[cfg(feature = "flac")]
    Flac(flac::FlacDecoder<R>),
    #[cfg(feature = "mp3")]
    Mp3(mp3::Mp3Decoder),
    None(::std::marker::PhantomData<R>),
}

/// `ReadSeekSource` implements a seekable `MediaSource` for any reader that implements the
/// `std::io::Read` and `std::io::Seek` traits.
pub struct ReadSeekSource<T: io::Read + io::Seek> {
    inner: T,
}

impl<T: io::Read + io::Seek> ReadSeekSource<T> {
    /// Instantiates a new `ReadSeekSource<T>` by taking ownership and wrapping the provided
    /// `Read + Seek`er.
    pub fn new(inner: T) -> Self {
        ReadSeekSource { inner }
    }

    /// Gets a reference to the underlying reader.
    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Gets a mutable reference to the underlying reader.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Unwraps this `ReadSeekSource<T>`, returning the underlying reader.
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: io::Read + io::Seek> MediaSource for ReadSeekSource<T> {
    fn is_seekable(&self) -> bool {
        true
    }

    fn len(&self) -> Option<u64> {
        None
    }
}

impl<T: io::Read + io::Seek> io::Read for ReadSeekSource<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<T: io::Read + io::Seek> io::Seek for ReadSeekSource<T> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}

impl<R> Decoder<R>
where
    R: Read + Seek + Send + 'static,
{
    /// Builds a new decoder.
    ///
    /// Attempts to automatically detect the format of the source of data.
    #[allow(unused_variables)]
    pub fn new(data: R) -> Result<Decoder<R>, DecoderError> {
        #[cfg(feature = "wav")]
        let data = match wav::WavDecoder::new(data) {
            Err(data) => data,
            Ok(decoder) => {
                return Ok(Decoder(DecoderImpl::Wav(decoder)));
            }
        };

        #[cfg(feature = "flac")]
        let data = match flac::FlacDecoder::new(data) {
            Err(data) => data,
            Ok(decoder) => {
                return Ok(Decoder(DecoderImpl::Flac(decoder)));
            }
        };

        #[cfg(feature = "vorbis")]
        let data = match vorbis::VorbisDecoder::new(data) {
            Err(data) => data,
            Ok(decoder) => {
                return Ok(Decoder(DecoderImpl::Vorbis(decoder)));
            }
        };

        #[cfg(feature = "mp3")]
        let data = {
            let mss = MediaSourceStream::new(
                Box::new(ReadSeekSource::new(data)) as Box<dyn MediaSource>,
                Default::default(),
            );
            let mut hint = Hint::new();

            let format_opts: FormatOptions = Default::default();
            let metadata_opts: MetadataOptions = Default::default();
            let probed = symphonia::default::get_probe()
                .format(&hint, mss, &format_opts, &metadata_opts)
                .unwrap();
            match mp3::Mp3Decoder::new(probed) {
                Err(data) => data,
                Ok(decoder) => {
                    return Ok(Decoder(DecoderImpl::Mp3(decoder)));
                }
            }
        };

        Err(DecoderError::UnrecognizedFormat)
    }
    pub fn new_looped(data: R) -> Result<LoopedDecoder<R>, DecoderError> {
        Self::new(data).map(LoopedDecoder::new)
    }

    /// Builds a new decoder from wav data.
    #[cfg(feature = "wav")]
    pub fn new_wav(data: R) -> Result<Decoder<R>, DecoderError> {
        match wav::WavDecoder::new(data) {
            Err(_) => Err(DecoderError::UnrecognizedFormat),
            Ok(decoder) => Ok(Decoder(DecoderImpl::Wav(decoder))),
        }
    }

    /// Builds a new decoder from flac data.
    #[cfg(feature = "flac")]
    pub fn new_flac(data: R) -> Result<Decoder<R>, DecoderError> {
        match flac::FlacDecoder::new(data) {
            Err(_) => Err(DecoderError::UnrecognizedFormat),
            Ok(decoder) => Ok(Decoder(DecoderImpl::Flac(decoder))),
        }
    }

    /// Builds a new decoder from vorbis data.
    #[cfg(feature = "vorbis")]
    pub fn new_vorbis(data: R) -> Result<Decoder<R>, DecoderError> {
        match vorbis::VorbisDecoder::new(data) {
            Err(_) => Err(DecoderError::UnrecognizedFormat),
            Ok(decoder) => Ok(Decoder(DecoderImpl::Vorbis(decoder))),
        }
    }

    /// Builds a new decoder from mp3 data.
    #[cfg(feature = "mp3")]
    pub fn new_mp3(data: R) -> Result<Decoder<R>, DecoderError> {
        let mss = MediaSourceStream::new(
            Box::new(ReadOnlySource::new(data)) as Box<dyn MediaSource>,
            Default::default(),
        );
        let mut hint = Hint::new();
        let format_opts: FormatOptions = Default::default();
        let metadata_opts: MetadataOptions = Default::default();
        let mut probed = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)
            .unwrap();
        match mp3::Mp3Decoder::new(probed) {
            Err(_) => Err(DecoderError::UnrecognizedFormat),
            Ok(decoder) => {
                return Ok(Decoder(DecoderImpl::Mp3(decoder)));
            }
        }
    }
}

impl<R> LoopedDecoder<R>
where
    R: Read + Seek + Send,
{
    fn new(decoder: Decoder<R>) -> LoopedDecoder<R> {
        Self(decoder.0)
    }
}

impl<R> Iterator for Decoder<R>
where
    R: Read + Seek,
{
    type Item = i16;

    #[inline]
    fn next(&mut self) -> Option<i16> {
        match &mut self.0 {
            #[cfg(feature = "wav")]
            DecoderImpl::Wav(source) => source.next(),
            #[cfg(feature = "vorbis")]
            DecoderImpl::Vorbis(source) => source.next(),
            #[cfg(feature = "flac")]
            DecoderImpl::Flac(source) => source.next(),
            #[cfg(feature = "mp3")]
            DecoderImpl::Mp3(source) => source.next(),
            DecoderImpl::None(_) => None,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.0 {
            #[cfg(feature = "wav")]
            DecoderImpl::Wav(source) => source.size_hint(),
            #[cfg(feature = "vorbis")]
            DecoderImpl::Vorbis(source) => source.size_hint(),
            #[cfg(feature = "flac")]
            DecoderImpl::Flac(source) => source.size_hint(),
            #[cfg(feature = "mp3")]
            DecoderImpl::Mp3(source) => source.size_hint(),
            DecoderImpl::None(_) => (0, None),
        }
    }
}

impl<R> Source for Decoder<R>
where
    R: Read + Seek,
{
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        match &self.0 {
            #[cfg(feature = "wav")]
            DecoderImpl::Wav(source) => source.current_frame_len(),
            #[cfg(feature = "vorbis")]
            DecoderImpl::Vorbis(source) => source.current_frame_len(),
            #[cfg(feature = "flac")]
            DecoderImpl::Flac(source) => source.current_frame_len(),
            #[cfg(feature = "mp3")]
            DecoderImpl::Mp3(source) => source.current_frame_len(),
            DecoderImpl::None(_) => Some(0),
        }
    }

    #[inline]
    fn channels(&self) -> u16 {
        match &self.0 {
            #[cfg(feature = "wav")]
            DecoderImpl::Wav(source) => source.channels(),
            #[cfg(feature = "vorbis")]
            DecoderImpl::Vorbis(source) => source.channels(),
            #[cfg(feature = "flac")]
            DecoderImpl::Flac(source) => source.channels(),
            #[cfg(feature = "mp3")]
            DecoderImpl::Mp3(source) => source.channels(),
            DecoderImpl::None(_) => 0,
        }
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        match &self.0 {
            #[cfg(feature = "wav")]
            DecoderImpl::Wav(source) => source.sample_rate(),
            #[cfg(feature = "vorbis")]
            DecoderImpl::Vorbis(source) => source.sample_rate(),
            #[cfg(feature = "flac")]
            DecoderImpl::Flac(source) => source.sample_rate(),
            #[cfg(feature = "mp3")]
            DecoderImpl::Mp3(source) => source.sample_rate(),
            DecoderImpl::None(_) => 1,
        }
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        match &self.0 {
            #[cfg(feature = "wav")]
            DecoderImpl::Wav(source) => source.total_duration(),
            #[cfg(feature = "vorbis")]
            DecoderImpl::Vorbis(source) => source.total_duration(),
            #[cfg(feature = "flac")]
            DecoderImpl::Flac(source) => source.total_duration(),
            #[cfg(feature = "mp3")]
            DecoderImpl::Mp3(source) => source.total_duration(),
            DecoderImpl::None(_) => Some(Duration::default()),
        }
    }
}

impl<R> Iterator for LoopedDecoder<R>
where
    R: Read + Seek,
{
    type Item = i16;

    #[inline]
    fn next(&mut self) -> Option<i16> {
        if let Some(sample) = match &mut self.0 {
            #[cfg(feature = "wav")]
            DecoderImpl::Wav(source) => source.next(),
            #[cfg(feature = "vorbis")]
            DecoderImpl::Vorbis(source) => source.next(),
            #[cfg(feature = "flac")]
            DecoderImpl::Flac(source) => source.next(),
            #[cfg(feature = "mp3")]
            DecoderImpl::Mp3(source) => source.next(),
            DecoderImpl::None(_) => None,
        } {
            Some(sample)
        } else {
            let decoder = mem::replace(&mut self.0, DecoderImpl::None(Default::default()));
            let (decoder, sample) = match decoder {
                #[cfg(feature = "wav")]
                DecoderImpl::Wav(source) => {
                    let mut reader = source.into_inner();
                    reader.seek(SeekFrom::Start(0)).ok()?;
                    let mut source = wav::WavDecoder::new(reader).ok()?;
                    let sample = source.next();
                    (DecoderImpl::Wav(source), sample)
                }
                #[cfg(feature = "vorbis")]
                DecoderImpl::Vorbis(source) => {
                    use lewton::inside_ogg::OggStreamReader;
                    let mut reader = source.into_inner().into_inner();
                    reader.seek_bytes(SeekFrom::Start(0)).ok()?;
                    let mut source = vorbis::VorbisDecoder::from_stream_reader(
                        OggStreamReader::from_ogg_reader(reader).ok()?,
                    );
                    let sample = source.next();
                    (DecoderImpl::Vorbis(source), sample)
                }
                #[cfg(feature = "flac")]
                DecoderImpl::Flac(source) => {
                    let mut reader = source.into_inner();
                    reader.seek(SeekFrom::Start(0)).ok()?;
                    let mut source = flac::FlacDecoder::new(reader).ok()?;
                    let sample = source.next();
                    (DecoderImpl::Flac(source), sample)
                }
                #[cfg(feature = "mp3")]
                DecoderImpl::Mp3(source) => {
                    let mut reader = source.into_inner();
                    reader
                        .format
                        .seek(SeekTo::Time {
                            time: Time::from_ss(0, 0).unwrap(),
                        })
                        .ok()?;
                    let mut source = mp3::Mp3Decoder::new(reader).ok()?;
                    let sample = source.next();
                    (DecoderImpl::Mp3(source), sample)
                }
                none @ DecoderImpl::None(_) => (none, None),
            };
            self.0 = decoder;
            sample
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.0 {
            #[cfg(feature = "wav")]
            DecoderImpl::Wav(source) => (source.size_hint().0, None),
            #[cfg(feature = "vorbis")]
            DecoderImpl::Vorbis(source) => (source.size_hint().0, None),
            #[cfg(feature = "flac")]
            DecoderImpl::Flac(source) => (source.size_hint().0, None),
            #[cfg(feature = "mp3")]
            DecoderImpl::Mp3(source) => (source.size_hint().0, None),
            DecoderImpl::None(_) => (0, None),
        }
    }
}

impl<R> Source for LoopedDecoder<R>
where
    R: Read + Seek,
{
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        match &self.0 {
            #[cfg(feature = "wav")]
            DecoderImpl::Wav(source) => source.current_frame_len(),
            #[cfg(feature = "vorbis")]
            DecoderImpl::Vorbis(source) => source.current_frame_len(),
            #[cfg(feature = "flac")]
            DecoderImpl::Flac(source) => source.current_frame_len(),
            #[cfg(feature = "mp3")]
            DecoderImpl::Mp3(source) => source.current_frame_len(),
            DecoderImpl::None(_) => Some(0),
        }
    }

    #[inline]
    fn channels(&self) -> u16 {
        match &self.0 {
            #[cfg(feature = "wav")]
            DecoderImpl::Wav(source) => source.channels(),
            #[cfg(feature = "vorbis")]
            DecoderImpl::Vorbis(source) => source.channels(),
            #[cfg(feature = "flac")]
            DecoderImpl::Flac(source) => source.channels(),
            #[cfg(feature = "mp3")]
            DecoderImpl::Mp3(source) => source.channels(),
            DecoderImpl::None(_) => 0,
        }
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        match &self.0 {
            #[cfg(feature = "wav")]
            DecoderImpl::Wav(source) => source.sample_rate(),
            #[cfg(feature = "vorbis")]
            DecoderImpl::Vorbis(source) => source.sample_rate(),
            #[cfg(feature = "flac")]
            DecoderImpl::Flac(source) => source.sample_rate(),
            #[cfg(feature = "mp3")]
            DecoderImpl::Mp3(source) => source.sample_rate(),
            DecoderImpl::None(_) => 1,
        }
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

/// Error that can happen when creating a decoder.
#[derive(Debug, Clone)]
pub enum DecoderError {
    /// The format of the data has not been recognized.
    UnrecognizedFormat,
}

impl fmt::Display for DecoderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecoderError::UnrecognizedFormat => write!(f, "Unrecognized format"),
        }
    }
}

impl Error for DecoderError {
    fn description(&self) -> &str {
        match self {
            DecoderError::UnrecognizedFormat => "Unrecognized format",
        }
    }
}
