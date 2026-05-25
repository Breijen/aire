use std::thread;
use std::time::Duration;
use aire::{Engine, FileSource, Sound};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::new()?;

    // drop any audio file (WAV, OGG, FLAC, MP3) into examples/ and update the path below
    let source = FileSource::load("./examples/example.ogg", engine.sample_rate())?.looping();

    // init first
    let handle = engine.add_sound(Sound::new(source, engine.sample_rate()))?;

    // then change params
    handle.volume(1.0)?;

    println!("playing...");
    thread::sleep(Duration::from_secs(3));

    println!("volume down (-12db)");
    handle.volume(-12.0)?;
    thread::sleep(Duration::from_secs(3));

    println!("pan right");
    handle.pan(1.0)?;
    thread::sleep(Duration::from_secs(3));

    println!("pan center, volume back to 0db");
    handle.pan(0.5)?;
    handle.volume(0.0)?;
    thread::sleep(Duration::from_secs(3));

    println!("pause");
    handle.pause()?;
    thread::sleep(Duration::from_secs(2));

    println!("resume");
    handle.resume()?;
    thread::sleep(Duration::from_secs(3));

    println!("stop");
    handle.stop()?;
    thread::sleep(Duration::from_secs(1));

    Ok(())
}
