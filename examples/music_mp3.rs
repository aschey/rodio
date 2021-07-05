use std::{io::BufReader, thread, time::Duration};

use rodio::Source;

fn main() {
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();

    //let file = std::fs::File::open("C:\\shared_files\\Music\\EDM Mixes\\April - 2013.mp3").unwrap();
    let file2 = std::fs::File::open("examples/music.mp3").unwrap();
    //let decoder1 = rodio::Decoder::new(BufReader::new(file)).unwrap();
    let mut decoder2 = rodio::Decoder::new(BufReader::new(file2)).unwrap();
    //S.buffered();
    //sink.append(decoder1);
    sink.append(decoder2);
    sink.sleep_until_end();
    // thread::sleep(Duration::from_millis(1));
}
