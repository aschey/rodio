use std::{
    io::{BufReader, Cursor},
    thread,
    time::Duration,
};

use rodio::Source;

use reqwest::header::{HeaderValue, CONTENT_LENGTH, RANGE};
use reqwest::StatusCode;
use std::fs::File;
use std::str::FromStr;

struct PartialRangeIter {
    start: u64,
    end: u64,
    buffer_size: u32,
}

impl PartialRangeIter {
    pub fn new(start: u64, end: u64, buffer_size: u32) -> Result<Self, &'static str> {
        if buffer_size == 0 {
            Err("invalid buffer_size, give a value greater than zero.")?;
        }
        Ok(PartialRangeIter {
            start,
            end,
            buffer_size,
        })
    }
}

impl Iterator for PartialRangeIter {
    type Item = HeaderValue;
    fn next(&mut self) -> Option<Self::Item> {
        if self.start > self.end {
            None
        } else {
            let prev_start = self.start;
            self.start += std::cmp::min(self.buffer_size as u64, self.end - self.start + 1);
            Some(
                HeaderValue::from_str(&format!("bytes={}-{}", prev_start, self.start - 1))
                    .expect("string provided by format!"),
            )
        }
    }
}

fn main() {
    pretty_env_logger::init();
    let url = "https://www.kozco.com/tech/LRMonoPhase4.wav";
    const CHUNK_SIZE: u32 = 10240;

    let client = reqwest::blocking::Client::new();
    let response = client.head(url).send().unwrap();
    let length = response
        .headers()
        .get(CONTENT_LENGTH)
        .ok_or("response doesn't include the content length")
        .unwrap();
    let length = u64::from_str(length.to_str().unwrap())
        .map_err(|_| "invalid Content-Length header")
        .unwrap();

    let mut output_file = File::create("download.bin").unwrap();

    thread::spawn(move || {
        println!("starting download...");
        for range in PartialRangeIter::new(0, length - 1, CHUNK_SIZE).unwrap() {
            println!("range {:?}", range);
            let mut response = client.get(url).header(RANGE, range).send().unwrap();

            let status = response.status();
            if !(status == StatusCode::OK || status == StatusCode::PARTIAL_CONTENT) {
                panic!("Unexpected server response: {}", status)
            }
            std::io::copy(&mut response, &mut output_file).unwrap();
        }
        let content = response.text().unwrap();
        std::io::copy(&mut content.as_bytes(), &mut output_file).unwrap();

        println!("Finished with success!");
    });
    thread::sleep(Duration::from_nanos(1));
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();

    let file = std::fs::File::open("C:\\shared_files\\Music\\EDM Mixes\\April - 2013.mp3").unwrap();
    let file2 = std::fs::File::open("download.bin").unwrap();

    // let decoder1 = rodio::Decoder::new(BufReader::new(resp)).unwrap();
    let mut decoder2 = rodio::Decoder::new(BufReader::new(file2)).unwrap();
    //S.buffered();
    //sink.append(decoder1);
    sink.append(decoder2);
    thread::sleep(Duration::from_secs(1));
    //sink.seek(Duration::from_secs(3));

    sink.sleep_until_end();
}
