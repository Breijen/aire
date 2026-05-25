use std::thread;
use std::time::Duration;

use aire::{DecodePool, Engine, FileSource, Sound};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::new()?;
    let rate   = engine.sample_rate();

    // The pool owns the background decode threads. Create it once and keep it
    // alive for as long as streaming sources are playing.
    let pool = DecodePool::new(1);

    // Stream the file instead of loading it all into memory.
    let source = FileSource::stream("./examples/example.ogg", rate, &pool)?.looping();
    let handle = engine.add_sound(Sound::new(source, 0.0, 0.5, rate))?;

    println!("streaming (looping)...");
    thread::sleep(Duration::from_secs(5));

    println!("volume down");
    handle.set_volume(-12.0)?;
    thread::sleep(Duration::from_secs(3));

    println!("stop");
    handle.stop()?;
    thread::sleep(Duration::from_millis(500));

    Ok(())
}
