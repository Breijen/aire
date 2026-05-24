use std::path::Path;
use hound::SampleFormat;
use rubato::{Resampler, Fft, FixedSync, Indexing};
use rubato::audioadapter_buffers::direct::InterleavedSlice;
use crate::AireError;
use crate::source::Source;

/// Loads and plays audio from a file. Supports WAV, OGG, and FLAC.
/// Mono files are duplicated to both channels. If the file sample
/// rate differs from the device rate, the audio is resampled on load.
pub struct FileSource {
    samples: Vec<f32>,
    current_pos: usize,
    channels: usize,
    looping: bool,
}

impl FileSource {
    /// Loads an audio file and prepares it for playback at the given device sample rate.
    /// Supports `.wav`, `.ogg`, and `.flac`.
    pub fn new(path: impl AsRef<Path>, device_rate: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let ext = path.as_ref()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let (raw, file_rate, channels) = match ext.as_str() {
            "wav"  => Self::load_wav(path.as_ref())?,
            "ogg"  => Self::load_ogg(path.as_ref())?,
            "flac" => Self::load_flac(path.as_ref())?,
            other  => return Err(AireError::FileExtNotSupported(other.to_string()).into()),
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

    fn load_wav(path: &Path) -> Result<(Vec<f32>, u32, usize), Box<dyn std::error::Error>> {
        let mut reader = hound::WavReader::open(path)?;
        let spec = reader.spec();
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

        Ok((raw, spec.sample_rate, channels))
    }

    fn load_ogg(path: &Path) -> Result<(Vec<f32>, u32, usize), Box<dyn std::error::Error>> {
        use lewton::inside_ogg::OggStreamReader;

        let mut reader = OggStreamReader::new(std::fs::File::open(path)?)?;
        let channels = reader.ident_hdr.audio_channels as usize;
        let sample_rate = reader.ident_hdr.audio_sample_rate;

        let mut raw: Vec<f32> = Vec::new();
        while let Some(packet) = reader.read_dec_packet_generic::<Vec<Vec<f32>>>()? {
            let frames = packet[0].len();
            for i in 0..frames {
                for ch in &packet {
                    raw.push(ch[i]);
                }
            }
        }

        Ok((raw, sample_rate, channels))
    }

    fn load_flac(path: &Path) -> Result<(Vec<f32>, u32, usize), Box<dyn std::error::Error>> {
        let mut reader = claxon::FlacReader::open(path)?;
        let info = reader.streaminfo();
        let channels = info.channels as usize;
        let sample_rate = info.sample_rate;
        let max_val = (1_i64 << (info.bits_per_sample - 1)) as f32;

        let mut raw: Vec<f32> = Vec::new();
        for sample in reader.samples() {
            raw.push(sample? as f32 / max_val);
        }

        Ok((raw, sample_rate, channels))
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
    fn fill_buffer(&mut self, buffer: &mut [(f32, f32)]) {
        for frame in buffer.iter_mut() {
            if self.current_pos >= self.samples.len() {
                *frame = (0.0, 0.0);
                continue;
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

            *frame = sample;
        }
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
        let mut buf = [(0.0f32, 0.0f32); 2];
        src.fill_buffer(&mut buf);
        assert_eq!(buf[0], (0.5, 0.5));
        assert_eq!(buf[1], (0.8, 0.8));
    }

    #[test]
    fn stereo_reads_interleaved_pairs() {
        let mut src = FileSource::from_samples(vec![0.1, 0.2, 0.3, 0.4], 2);
        let mut buf = [(0.0f32, 0.0f32); 2];
        src.fill_buffer(&mut buf);
        assert_eq!(buf[0], (0.1, 0.2));
        assert_eq!(buf[1], (0.3, 0.4));
    }

    #[test]
    fn finishes_when_exhausted() {
        let mut src = FileSource::from_samples(vec![0.5], 1);
        let mut buf = [(0.0f32, 0.0f32); 1];
        src.fill_buffer(&mut buf);
        assert!(src.is_finished());
    }

    #[test]
    fn looping_wraps_and_never_finishes() {
        let mut src = FileSource::from_samples(vec![0.1, 0.2], 1).looping();
        let mut buf = [(0.0f32, 0.0f32); 1];
        src.fill_buffer(&mut buf);
        src.fill_buffer(&mut buf);
        src.fill_buffer(&mut buf);
        let (l, _) = buf[0];
        assert!((l - 0.1).abs() < 0.001);
        assert!(!src.is_finished());
    }
}
