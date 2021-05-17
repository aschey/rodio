use std::io::{Read, Seek, SeekFrom};
use std::time::Duration;
use std::vec;

use crate::Source;

use lewton::inside_ogg::SeekableOggStreamReader;

/// Decoder for an OGG file that contains Vorbis sound format.
pub struct VorbisDecoder<R>
where
    R: Read + Seek,
{
    stream_reader: SeekableOggStreamReader<R>,
    current_data: vec::IntoIter<i16>,
}

impl<R> VorbisDecoder<R>
where
    R: Read + Seek,
{
    /// Attempts to decode the data as ogg/vorbis.
    pub fn new(mut data: R) -> Result<VorbisDecoder<R>, R> {
        if !is_vorbis(data.by_ref()) {
            return Err(data);
        }

        let stream_reader = SeekableOggStreamReader::new(data).unwrap();
        Ok(Self::from_stream_reader(stream_reader))
    }
    pub fn from_stream_reader(mut stream_reader: SeekableOggStreamReader<R>) -> Self {
        let mut data = match stream_reader.inner_mut().read_dec_packet_itl() {
            Ok(Some(d)) => d,
            _ => Vec::new(),
        };

        // The first packet is always empty, therefore
        // we need to read the second frame to get some data
        if let Ok(Some(mut d)) = stream_reader.inner_mut().read_dec_packet_itl() {
            data.append(&mut d);
        }

        VorbisDecoder {
            stream_reader,
            current_data: data.into_iter(),
        }
    }
    pub fn into_inner(self) -> SeekableOggStreamReader<R> {
        self.stream_reader
    }

    fn get_next(&mut self) {
        loop {
            match self
                .stream_reader
                .inner_mut()
                .read_dec_packet_itl()
                .unwrap()
            {
                Some(data) => {
                    if data.len() > 0 {
                        self.current_data = data.into_iter();
                        break;
                    }
                }
                None => {
                    break;
                }
            }
        }
    }
}

impl<R> Source for VorbisDecoder<R>
where
    R: Read + Seek,
{
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.current_data.len())
    }

    #[inline]
    fn channels(&self) -> u16 {
        self.stream_reader.inner().ident_hdr().audio_channels as u16
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        self.stream_reader.inner().ident_hdr().audio_sample_rate
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        None
    }

    fn seek(&mut self, time: Duration) -> Result<Duration, ()> {
        match self
            .stream_reader
            .seek_absgp(time.as_secs() * self.sample_rate() as u64)
        {
            Ok(()) => Ok(time),
            Err(_) => Err(()),
        }
    }
}

impl<R> Iterator for VorbisDecoder<R>
where
    R: Read + Seek,
{
    type Item = i16;

    #[inline]
    fn next(&mut self) -> Option<i16> {
        if let Some(sample) = self.current_data.next() {
            if self.current_data.len() == 0 {
                self.get_next();
            }
            Some(sample)
        } else {
            self.get_next();
            self.current_data.next()
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.current_data.size_hint().0, None)
    }
}

/// Returns true if the stream contains Vorbis data, then resets it to where it was.
fn is_vorbis<R>(mut data: R) -> bool
where
    R: Read + Seek,
{
    let stream_pos = data.seek(SeekFrom::Current(0)).unwrap();

    if ogg_metadata::read_format(&mut data).is_err() {
        data.seek(SeekFrom::Start(stream_pos)).unwrap();
        return false;
    }

    data.seek(SeekFrom::Start(stream_pos)).unwrap();
    true
}
