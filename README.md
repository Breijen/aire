[![Crates.io](https://img.shields.io/crates/v/aire)](https://crates.io/crates/aire)
[![Docs.rs](https://docs.rs/aire/badge.svg)](https://docs.rs/aire)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)

# AIRE

AIRE is an audio engine for Rust built for games and interactive applications. It lets you load sounds, play them, and control them at runtime.

## Features

- Play WAV, OGG, and FLAC files
- Synthesize audio with a six-waveform oscillator (sine, saw, triangle, square, pulse)
- Loop sounds
- Control volume and pan at runtime
- Pause, resume, and stop sounds
- Apply amplitude envelopes (ADSR)
- Write custom audio sources and effects

## Usage

Add AIRE to your `Cargo.toml`:

```toml
[dependencies]
aire = "0.2"
```

### Play a sound

```rust
use aire::{Engine, FileSource, Sound};
use std::{thread, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::new()?;
    let source = FileSource::new("sound.wav", engine.sample_rate())?;
    let _handle = engine.add_sound(Sound::new(source, 0.0, 0.5, engine.sample_rate()))?;

    thread::sleep(Duration::from_secs(5));
    Ok(())
}
```

### Synthesize a tone

```rust
use aire::{Engine, Oscillator, Sound, Waveform};
use std::{thread, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::new()?;
    let rate = engine.sample_rate();

    let osc = Oscillator::new(Waveform::Saw, 220.0, rate)
        .amplitude(-6.0)
        .duration(2000);
    let _handle = engine.add_sound(Sound::new(osc, 0.0, 0.5, rate))?;

    thread::sleep(Duration::from_secs(3));
    Ok(())
}
```

### Control a sound at runtime

```rust
use aire::{Engine, FileSource, Sound};
use std::{thread, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::new()?;

    let source = FileSource::new("music.wav", engine.sample_rate())?.looping();
    let handle = engine.add_sound(Sound::new(source, 0.0, 0.5, engine.sample_rate()))?;

    thread::sleep(Duration::from_secs(3));
    handle.set_volume(-6.0)?;
    handle.set_pan(0.25)?;

    thread::sleep(Duration::from_secs(3));
    handle.pause()?;

    thread::sleep(Duration::from_secs(2));
    handle.resume()?;

    thread::sleep(Duration::from_secs(3));
    handle.stop()?;

    Ok(())
}
```

## Roadmap

Planned features and future direction are tracked on the [project board](https://github.com/users/Breijen/projects/3).

## Contributing

Issues and pull requests are welcome. If you're planning something big, open an issue first so we can discuss it before you put in the work.

For bug reports, include a minimal reproducible example if you can. It makes things a lot faster to track down.

Before opening a PR, make sure `cargo test` passes and there are no new warnings.

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE) at your option.
