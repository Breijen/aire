use std::error::Error;
use std::path::{Path, PathBuf};
use lewton::inside_ogg::OggStreamReader;

use crate::AireError;
use super::StreamDecoder;

// WAV

pub(crate) struct WavDecoder {
    reader: hound::WavReader<std::io::BufReader<std::fs::File>>,
    spec: hound::WavSpec,
    finished: bool,
}

impl WavDecoder {
    pub(crate) fn new(path: &Path) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let reader = hound::WavReader::open(path)?;
        let spec = reader.spec();
        Ok(WavDecoder { reader, spec, finished: false })
    }
}

impl StreamDecoder for WavDecoder {
    fn channels(&self) -> usize { self.spec.channels as usize }
    fn sample_rate(&self) -> u32 { self.spec.sample_rate }

    fn decode_chunk(&mut self, out: &mut Vec<f32>, target: usize) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let mut written = 0;

        match self.spec.sample_format {
            hound::SampleFormat::Float => {
                for s in self.reader.samples::<f32>().take(target) {
                    out.push(s?);
                    written += 1;
                }
            }
            hound::SampleFormat::Int => match self.spec.bits_per_sample {
                16 => {
                    for s in self.reader.samples::<i16>().take(target) {
                        out.push(s? as f32 / i16::MAX as f32);
                        written += 1;
                    }
                }
                32 => {
                    for s in self.reader.samples::<i32>().take(target) {
                        out.push(s? as f32 / i32::MAX as f32);
                        written += 1;
                    }
                }
                _ => return Err("unsupported WAV bit depth".into()),
            },
        }

        if written < target { self.finished = true; }
        Ok(written)
    }

    fn reset(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.reader.seek(0)?;
        self.finished = false;
        Ok(())
    }

    fn finished(&self) -> bool { self.finished }
}

// OGG

pub(crate) struct OggDecoder {
    path: PathBuf,
    reader: OggStreamReader<std::fs::File>,
    channels: usize,
    sample_rate: u32,
    overflow: Vec<f32>,
    finished: bool,
}

impl OggDecoder {
    pub(crate) fn new(path: &Path) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let reader = OggStreamReader::new(std::fs::File::open(path)?)?;
        let channels = reader.ident_hdr.audio_channels as usize;
        let sample_rate = reader.ident_hdr.audio_sample_rate;
        Ok(OggDecoder {
            path: path.to_path_buf(),
            reader,
            channels,
            sample_rate,
            overflow: Vec::new(),
            finished: false,
        })
    }
}

impl StreamDecoder for OggDecoder {
    fn channels(&self) -> usize { self.channels }
    fn sample_rate(&self) -> u32 { self.sample_rate }

    fn decode_chunk(&mut self, out: &mut Vec<f32>, target: usize) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let ch = self.channels;

        while self.overflow.len() < target && !self.finished {
            match self.reader.read_dec_packet_generic::<Vec<Vec<f32>>>()? {
                None => { self.finished = true; }
                Some(packet) => {
                    let frames = packet[0].len();
                    for i in 0..frames {
                        for c in 0..ch { self.overflow.push(packet[c][i]); }
                    }
                }
            }
        }

        let take = self.overflow.len().min(target);
        out.extend_from_slice(&self.overflow[..take]);
        self.overflow.drain(..take);
        Ok(take)
    }

    fn reset(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        // lewton does not expose a seek API, so we reopen the file to loop.
        self.reader = OggStreamReader::new(std::fs::File::open(&self.path)?)?;
        self.overflow.clear();
        self.finished = false;
        Ok(())
    }

    fn finished(&self) -> bool { self.finished }
}

pub(crate) fn open_stream_decoder(path: &Path) -> Result<Box<dyn StreamDecoder>, Box<dyn Error + Send + Sync>> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

    Ok(match ext.as_str() {
        "wav"  => Box::new(WavDecoder::new(path)?),
        "ogg"  => Box::new(OggDecoder::new(path)?),
        "flac" | "mp3" => return Err(AireError::StreamingNotSupported(ext).into()),
        other  => return Err(AireError::FileExtNotSupported(other.to_string()).into()),
    })
}
