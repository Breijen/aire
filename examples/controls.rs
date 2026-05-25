use std::thread;
use std::time::Duration;
use aire::{Engine, FileSource, Sound};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::new()?;

    // drop any WAV file into examples/ and update the path below
    let source = FileSource::load("./examples/example.ogg", engine.sample_rate())?.looping();
    let handle = engine.add_sound(Sound::new(source, 0.0, 0.5, engine.sample_rate()))?;

    println!("playing...");
    thread::sleep(Duration::from_secs(3));

    println!("volume down (-12db)");
    handle.set_volume(-12.0)?;
    thread::sleep(Duration::from_secs(3));

    println!("pan right");
    handle.set_pan(1.0)?;
    thread::sleep(Duration::from_secs(3));

    println!("pan center, volume back to 0db");
    handle.set_pan(0.5)?;
    handle.set_volume(0.0)?;
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
