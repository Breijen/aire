use std::path::Path;
use hound::SampleFormat;
use rubato::{Resampler, Fft, FixedSync, Indexing};
use rubato::audioadapter_buffers::direct::InterleavedSlice;
use crate::source::Source;

/// Loads and plays audio from a WAV file. Supports 16-bit and 32-bit integer
/// PCM (Pulse-Code Modulation, the standard uncompressed audio format) and
/// 32-bit float. Mono files are duplicated to both channels. If the file sample
/// rate differs from the device rate, the audio is resampled on load.
pub struct FileSource {
    samples: Vec<f32>,
    current_pos: usize,
    channels: usize,
    looping: bool,
}

impl FileSource {
    /// Loads a WAV file and prepares it for playback at the given device sample rate.
    pub fn new(path: impl AsRef<Path>, device_rate: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let mut reader = hound::WavReader::open(path)?;
        let spec = reader.spec();
        let file_rate = spec.sample_rate;
        let channels = spec.channels as usize;

        let raw: Vec<f32> = match spec.sample_format {
            SampleFormat::Int => match spec.bits_per_sample {
                16 => reader.samples::<i16>()
                    .map(|s| Ok(s? as f32 / i16::MAX as f32))
                    .collect::<Result<_, hound::Error>>()?,
                32 => reader.samples::<i32>()
                    .map(|s| Ok(s? as f32 / i32::MAX as f32))
                    .collect::<Result<_, hound::Error>>()?,
                _ => return Err("unsupported bit depth".into()),
            },
            SampleFormat::Float => reader.samples::<f32>()
                .map(|s| Ok(s?))
                .collect::<Result<_, hound::Error>>()?,
        };

        let samples = if file_rate != device_rate {
            Self::resample(raw, device_rate, file_rate, channels)
        } else {
            raw
        };

        Ok(Self { samples, current_pos: 0, channels, looping: false })
    }

    /// Enables looping. The source restarts from the beginning when it reaches
    /// the end and will never report as finished.
    pub fn looping(mut self) -> Self {
        self.looping = true;
        self
    }

    #[cfg(test)]
    fn from_samples(samples: Vec<f32>, channels: usize) -> Self {
        Self { samples, current_pos: 0, channels, looping: false }
    }

    fn resample(raw: Vec<f32>, device_rate: u32, file_rate: u32, channels: usize) -> Vec<f32> {
        let frames = raw.len() / channels;
        let input_f64: Vec<f64> = raw.iter().map(|s| *s as f64).collect();

        let ratio = device_rate as f64 / file_rate as f64;
        let output_frames = (frames as f64 * ratio).ceil() as usize + 2048;
        let mut output_f64 = vec![0.0f64; output_frames * channels];

        let mut resampler = Fft::<f64>::new(
            file_rate as usize,
            device_rate as usize,
            1024,
            channels,
            2,
            FixedSync::Input,
        ).unwrap();

        let input_adapter = InterleavedSlice::new(&input_f64, channels, frames).unwrap();
        let mut output_adapter = InterleavedSlice::new_mut(&mut output_f64, channels, output_frames).unwrap();

        let mut indexing = Indexing {
            input_offset: 0,
            output_offset: 0,
            active_channels_mask: None,
            partial_len: None,
        };

        let mut input_frames_left = frames;
        let mut input_frames_next = resampler.input_frames_next();

        while input_frames_left >= input_frames_next {
            let (frames_read, frames_written) = resampler.process_into_buffer(
                &input_adapter,
                &mut output_adapter,
                Some(&indexing),
            ).unwrap();
            indexing.input_offset += frames_read;
            indexing.output_offset += frames_written;
            input_frames_left -= frames_read;
            input_frames_next = resampler.input_frames_next();
        }

        let total = indexing.output_offset * channels;
        output_f64[..total].iter().map(|s| *s as f32).collect()
    }
}

impl Source for FileSource {
    fn next_sample(&mut self) -> (f32, f32) {
        if self.current_pos >= self.samples.len() {
            return (0.0, 0.0);
        }

        let sample = if self.channels == 2 {
            let left = self.samples[self.current_pos];
            let right = self.samples[self.current_pos + 1];
            self.current_pos += 2;
            (left, right)
        } else {
            let s = self.samples[self.current_pos];
            self.current_pos += 1;
            (s, s)
        };

        if self.looping && self.current_pos >= self.samples.len() {
            self.current_pos = 0;
        }

        sample
    }

    fn is_finished(&self) -> bool {
        !self.looping && self.current_pos >= self.samples.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mono_duplicates_to_both_channels() {
        let mut src = FileSource::from_samples(vec![0.5, 0.8], 1);
        assert_eq!(src.next_sample(), (0.5, 0.5));
        assert_eq!(src.next_sample(), (0.8, 0.8));
    }

    #[test]
    fn stereo_reads_interleaved_pairs() {
        let mut src = FileSource::from_samples(vec![0.1, 0.2, 0.3, 0.4], 2);
        assert_eq!(src.next_sample(), (0.1, 0.2));
        assert_eq!(src.next_sample(), (0.3, 0.4));
    }

    #[test]
    fn finishes_when_exhausted() {
        let mut src = FileSource::from_samples(vec![0.5], 1);
        src.next_sample();
        assert!(src.is_finished());
    }

    #[test]
    fn looping_wraps_and_never_finishes() {
        let mut src = FileSource::from_samples(vec![0.1, 0.2], 1).looping();
        src.next_sample();
        src.next_sample();
        let (l, _) = src.next_sample();
        assert!((l - 0.1).abs() < 0.001);
        assert!(!src.is_finished());
    }
}