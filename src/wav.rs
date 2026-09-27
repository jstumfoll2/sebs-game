//! Tiny WAV file helpers: read 16-bit PCM WAV files, resample them, and write them.
//! (Used for the synthesized sound effects and for the voice clips.)

/// Decoded audio: samples from -1.0 to 1.0, interleaved if stereo.
pub struct Audio {
    pub samples: Vec<f32>,
    pub channels: u16,
    pub rate: u32,
}

impl Audio {
    pub fn seconds(&self) -> f32 {
        self.samples.len() as f32 / self.channels as f32 / self.rate as f32
    }

    /// Change the sample rate using straight-line interpolation between samples.
    pub fn resample(&self, rate: u32) -> Audio {
        if rate == self.rate {
            return Audio { samples: self.samples.clone(), channels: self.channels, rate };
        }
        let ch = self.channels as usize;
        let frames = self.samples.len() / ch;
        let new_frames = (frames as u64 * rate as u64 / self.rate as u64) as usize;
        let step = self.rate as f64 / rate as f64;
        let mut samples = Vec::with_capacity(new_frames * ch);
        for i in 0..new_frames {
            let pos = i as f64 * step;
            let a = (pos as usize).min(frames.saturating_sub(1));
            let b = (a + 1).min(frames.saturating_sub(1));
            let t = (pos - a as f64) as f32;
            for c in 0..ch {
                let (sa, sb) = (self.samples[a * ch + c], self.samples[b * ch + c]);
                samples.push(sa + (sb - sa) * t);
            }
        }
        Audio { samples, channels: self.channels, rate }
    }
}

/// Read a 16-bit PCM WAV file. Returns None for anything else.
pub fn decode(bytes: &[u8]) -> Option<Audio> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return None;
    }
    let u16_at = |i: usize| u16::from_le_bytes([bytes[i], bytes[i + 1]]);
    let u32_at = |i: usize| u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);

    // Walk through the file's "chunks" looking for the format and the data.
    let (mut channels, mut rate, mut bits) = (0, 0, 0);
    let mut pos = 12;
    while pos + 8 <= bytes.len() {
        let id = &bytes[pos..pos + 4];
        let size = u32_at(pos + 4) as usize;
        let body = pos + 8;
        if id == b"fmt " && body + 16 <= bytes.len() {
            if u16_at(body) != 1 {
                return None; // not plain PCM
            }
            channels = u16_at(body + 2);
            rate = u32_at(body + 4);
            bits = u16_at(body + 14);
        } else if id == b"data" {
            if bits != 16 || channels == 0 || channels > 2 || rate == 0 {
                return None;
            }
            let end = (body + size).min(bytes.len());
            let samples = bytes[body..end]
                .chunks_exact(2)
                .map(|b| i16::from_le_bytes([b[0], b[1]]) as f32 / 32768.0)
                .collect();
            return Some(Audio { samples, channels, rate });
        }
        pos = body + size + (size % 2); // chunks are padded to even sizes
    }
    None
}

/// Write audio as a 16-bit PCM WAV file.
pub fn encode(audio: &Audio) -> Vec<u8> {
    let data_len = (audio.samples.len() * 2) as u32;
    let block = audio.channels as u32 * 2;
    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes()); // size of this chunk
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&audio.channels.to_le_bytes());
    out.extend_from_slice(&audio.rate.to_le_bytes());
    out.extend_from_slice(&(audio.rate * block).to_le_bytes()); // bytes per second
    out.extend_from_slice(&(block as u16).to_le_bytes()); // bytes per sample frame
    out.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for s in &audio.samples {
        let v = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}
