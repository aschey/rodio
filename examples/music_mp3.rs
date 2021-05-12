use std::io::BufReader;

fn main() {
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();

    let file = std::fs::File::open("examples/music.mp3").unwrap();
    let file2 = std::fs::File::open(
        "C:\\Users\\asche\\Downloads\\Stanley.Clarke.AAC\\01 01 Vulcan Princess_short.m4a",
    )
    .unwrap();
    let decoder1 = rodio::Decoder::new(BufReader::new(file)).unwrap();
    let decoder2 = rodio::Decoder::new(BufReader::new(file2)).unwrap();
    sink.append(decoder1);
    sink.append(decoder2);

    sink.sleep_until_end();
}
