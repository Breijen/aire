//! A real-time audio engine for games and interactive applications.
//!
//! # Quick start
//!
//! ```no_run
//! use aire::{Engine, Sound, Oscillator, Waveform};
//!
//! let engine = Engine::new()?;
//! let rate = engine.sample_rate();
//!
//! let osc = Oscillator::new(Waveform::Sine, 440.0, rate).duration(2000);
//! let _handle = engine.add_sound(Sound::new(osc, rate))?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Key types
//!
//! - [`Engine`] — opens the output device and drives the audio thread
//! - [`EngineConfig`] — optional configuration (command buffer size, etc.)
//! - [`Sound`] — wraps a source with volume, pan, and an effect chain
//! - [`SoundHandle`] — controls a playing sound from any thread
//! - [`GroupHandle`] — controls volume and pan for a named category of sounds
//! - [`Source`] — trait for anything that produces audio
//! - [`Effect`] — trait for DSP effects
//! - [`FileSource`] — loads or streams WAV, OGG, FLAC, and MP3 files
//! - [`Oscillator`] — band-limited synthesized waveform source with six shapes
//! - [`Adsr`] — ADSR amplitude envelope with linear or exponential curves
//! - [`DecodePool`] — background thread pool for streaming decode.
//!   Only needed when using [`FileSource::stream`].

mod engine;
mod error;
mod handle;
mod mixer;
mod source;
mod utils;
mod dsp;
mod sound;
pub(crate) mod streaming;

pub use streaming::pool::DecodePool;

pub use engine::{Engine, EngineConfig};
pub use error::AireError;
pub use handle::{GroupHandle, SoundHandle};
pub use source::Source;
pub use source::file::FileSource;
pub use source::oscillator::{Oscillator, Waveform};
pub use dsp::adsr::{Adsr, Curve};
pub use dsp::effect::Effect;
pub use sound::Sound;
