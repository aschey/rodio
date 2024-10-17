//! Test signal generator example.

fn main() {
    use rodio::source::{chirp, Function, SignalGenerator, Source};
    use std::thread;
    use std::time::Duration;

    let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();

    let test_signal_duration = Duration::from_millis(1000);
    let interval_duration = Duration::from_millis(1500);

    println!("Playing 1000 Hz tone");
    stream_handle
        .play_raw(
            SignalGenerator::new(cpal::SampleRate(48000), 1000.0, Function::Sine)
                .amplify(0.1)
                .take_duration(test_signal_duration),
        )
        .unwrap();

    thread::sleep(interval_duration);

    println!("Playing 10,000 Hz tone");
    stream_handle
        .play_raw(
            SignalGenerator::new(cpal::SampleRate(48000), 10000.0, Function::Sine)
                .amplify(0.1)
                .take_duration(test_signal_duration),
        )
        .unwrap();

    thread::sleep(interval_duration);

    println!("Playing 440 Hz Triangle Wave");
    stream_handle
        .play_raw(
            SignalGenerator::new(cpal::SampleRate(48000), 440.0, Function::Triangle)
                .amplify(0.1)
                .take_duration(test_signal_duration),
        )
        .unwrap();

    thread::sleep(interval_duration);

    println!("Playing 440 Hz Sawtooth Wave");
    stream_handle
        .play_raw(
            SignalGenerator::new(cpal::SampleRate(48000), 440.0, Function::Sawtooth)
                .amplify(0.1)
                .take_duration(test_signal_duration),
        )
        .unwrap();

    thread::sleep(interval_duration);

    println!("Playing 440 Hz Square Wave");
    stream_handle
        .play_raw(
            SignalGenerator::new(cpal::SampleRate(48000), 440.0, Function::Square)
                .amplify(0.1)
                .take_duration(test_signal_duration),
        )
        .unwrap();

    thread::sleep(interval_duration);

    println!("Playing 20-10000 Hz Sweep");
    stream_handle
        .play_raw(
            chirp(
                cpal::SampleRate(48000),
                20.0,
                10000.0,
                Duration::from_secs(1),
            )
            .amplify(0.1)
            .take_duration(test_signal_duration),
        )
        .unwrap();

    thread::sleep(interval_duration);
}