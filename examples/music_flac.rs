use std::{io::BufReader, thread, time::Duration};

fn main() {
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();

    let file = std::fs::File::open("examples/music.flac").unwrap();
    sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());
    let receiver = sink.get_current_receiver().unwrap();
    thread::sleep(Duration::from_secs(1));
    sink.seek(Duration::from_secs(3));

    receiver.recv().unwrap();
}
