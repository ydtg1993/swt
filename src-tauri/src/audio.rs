/// Pure-Rust audio decoding and resampling (symphonia + rubato).
/// Extracts 16kHz mono PCM from media files without ffmpeg.

use std::fs::File;
use std::path::Path;

use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// Check if this file is a video format that needs audio extraction.
pub fn needs_extraction(path: &str) -> bool {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    matches!(
        ext.as_str(),
        "mp4" | "mkv" | "avi" | "mov" | "webm" | "flv" | "wmv" | "m4a"
    )
}

/// Extract audio to 16kHz mono WAV.
/// Returns the path to the extracted WAV file.
pub fn extract_audio(input_path: &str, output_path: &str) -> Result<(), String> {
    // Try symphonia first (pure Rust)
    if let Err(e) = extract_audio_symphonia(input_path, output_path) {
        // Fall back to ffmpeg for unsupported formats
        eprintln!(
            "[audio] symphonia failed: {} — falling back to ffmpeg",
            e
        );
        extract_audio_ffmpeg(input_path, output_path)?;
    }
    Ok(())
}

/// Count samples for converting a decoded buffer to total sample count.
fn buffer_sample_count(buf: &AudioBufferRef) -> usize {
    match buf {
        AudioBufferRef::F32(b) => b.frames(),
        AudioBufferRef::U8(b) => b.frames(),
        AudioBufferRef::S8(b) => b.frames(),
        AudioBufferRef::U16(b) => b.frames(),
        AudioBufferRef::S16(b) => b.frames(),
        AudioBufferRef::U24(b) => b.frames(),
        AudioBufferRef::S24(b) => b.frames(),
        AudioBufferRef::U32(b) => b.frames(),
        AudioBufferRef::S32(b) => b.frames(),
        AudioBufferRef::F64(b) => b.frames(),
    }
}

/// Copy decoded samples into a f32 Vec (mono or downmixed from stereo).
fn copy_to_f32(buf: &AudioBufferRef, dest: &mut Vec<f32>) {
    let frames = buffer_sample_count(buf);
    dest.reserve(frames);
    match buf {
        AudioBufferRef::F32(b) => {
            let channels = b.spec().channels.count();
            for f in 0..frames {
                let mut sum = 0.0f32;
                for c in 0..channels {
                    sum += b.chan(c)[f];
                }
                dest.push(sum / channels as f32);
            }
        }
        AudioBufferRef::S16(b) => {
            let channels = b.spec().channels.count();
            for f in 0..frames {
                let mut sum = 0.0f32;
                for c in 0..channels {
                    sum += b.chan(c)[f] as f32 / 32768.0;
                }
                dest.push(sum / channels as f32);
            }
        }
        AudioBufferRef::U8(b) => {
            let channels = b.spec().channels.count();
            for f in 0..frames {
                let mut sum = 0.0f32;
                for c in 0..channels {
                    sum += (b.chan(c)[f] as f32 - 128.0) / 128.0;
                }
                dest.push(sum / channels as f32);
            }
        }
        AudioBufferRef::S32(b) => {
            let channels = b.spec().channels.count();
            for f in 0..frames {
                let mut sum = 0.0f32;
                for c in 0..channels {
                    sum += b.chan(c)[f] as f32 / 2147483648.0;
                }
                dest.push(sum / channels as f32);
            }
        }
        _ => {
            // Fallback: treat all samples as zeros
            for _ in 0..frames {
                dest.push(0.0);
            }
        }
    }
}

fn extract_audio_symphonia(input_path: &str, output_path: &str) -> Result<(), String> {
    let src = File::open(input_path).map_err(|e| format!("open failed: {}", e))?;
    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = Path::new(input_path).extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| format!("probe failed: {}", e))?;

    let mut format = probed.format;

    let track = format
        .tracks()
        .iter()
        .find(|t| {
            use symphonia::core::codecs::CODEC_TYPE_AAC;
            use symphonia::core::codecs::CODEC_TYPE_ALAC;
            use symphonia::core::codecs::CODEC_TYPE_FLAC;
            use symphonia::core::codecs::CODEC_TYPE_MP3;
            use symphonia::core::codecs::CODEC_TYPE_NULL;
            use symphonia::core::codecs::CODEC_TYPE_VORBIS;
            let ct = t.codec_params.codec;
            ct == CODEC_TYPE_AAC
                || ct == CODEC_TYPE_ALAC
                || ct == CODEC_TYPE_FLAC
                || ct == CODEC_TYPE_MP3
                || ct == CODEC_TYPE_NULL
                || ct == CODEC_TYPE_VORBIS
        })
        .ok_or("no supported audio track found")?;

    let track_id = track.id;
    let sample_rate = track
        .codec_params
        .sample_rate
        .ok_or("unknown sample rate")?;

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| format!("decoder error: {}", e))?;

    // Collect all decoded samples
    let mut samples: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(symphonia::core::errors::Error::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(e) => {
                eprintln!("[audio] packet read error: {}", e);
                break;
            }
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = decoder
            .decode(&packet)
            .map_err(|e| format!("decode error: {}", e))?;

        copy_to_f32(&decoded, &mut samples);
    }

    if samples.is_empty() {
        return Err("no audio samples decoded".into());
    }

    // Resample to 16kHz mono using rubato
    let resampled = if sample_rate != 16000 {
        resample_sinc(&samples, sample_rate, 16000)?
    } else {
        samples
    };

    // Write 16-bit PCM WAV
    write_wav_i16(output_path, &resampled, 16000)
}

/// High-quality sinc resampling using rubato.
fn resample_sinc(input: &[f32], from_rate: u32, to_rate: u32) -> Result<Vec<f32>, String> {
    use rubato::{Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction};

    let params = SincInterpolationParameters {
        sinc_len: 256,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 256,
        window: WindowFunction::BlackmanHarris2,
    };

    let mut resampler = SincFixedIn::<f32>::new(
        to_rate as f64 / from_rate as f64,
        2.0,
        params,
        input.len(),
        1, // 1 channel (mono)
    )
    .map_err(|e| format!("resampler error: {}", e))?;

    let input_frames: Vec<Vec<f32>> = vec![input.to_vec()];
    let output = resampler
        .process(&input_frames, None)
        .map_err(|e| format!("resampling error: {}", e))?;

    Ok(output.into_iter().next().unwrap_or_default())
}

/// Write 16-bit mono PCM as WAV file.
fn write_wav_i16(path: &str, samples: &[f32], sample_rate: u32) -> Result<(), String> {
    use std::io::Write;

    let num_samples = samples.len();
    let data_size = (num_samples * 2) as u32; // 16-bit = 2 bytes per sample

    let mut file = File::create(path).map_err(|e| format!("create wav: {}", e))?;

    // WAV header
    file.write_all(b"RIFF").map_err(|e| e.to_string())?;
    file.write_all(&(36 + data_size).to_le_bytes()).map_err(|e| e.to_string())?;
    file.write_all(b"WAVE").map_err(|e| e.to_string())?;
    file.write_all(b"fmt ").map_err(|e| e.to_string())?;
    file.write_all(&16u32.to_le_bytes()).map_err(|e| e.to_string())?; // chunk size
    file.write_all(&1u16.to_le_bytes()).map_err(|e| e.to_string())?; // PCM
    file.write_all(&1u16.to_le_bytes()).map_err(|e| e.to_string())?; // mono
    file.write_all(&sample_rate.to_le_bytes()).map_err(|e| e.to_string())?;
    file.write_all(&(sample_rate * 2).to_le_bytes()).map_err(|e| e.to_string())?; // byte rate
    file.write_all(&2u16.to_le_bytes()).map_err(|e| e.to_string())?; // block align
    file.write_all(&16u16.to_le_bytes()).map_err(|e| e.to_string())?; // bits per sample
    file.write_all(b"data").map_err(|e| e.to_string())?;
    file.write_all(&data_size.to_le_bytes()).map_err(|e| e.to_string())?;

    // PCM data
    for &s in samples {
        let clamped = s.clamp(-1.0, 1.0);
        let sample = (clamped * 32767.0) as i16;
        file.write_all(&sample.to_le_bytes()).map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Get audio duration using symphonia (pure Rust).
pub fn get_duration_symphonia(path: &str) -> Result<f64, String> {
    let src = File::open(path).map_err(|e| format!("open: {}", e))?;
    let mss = MediaSourceStream::new(Box::new(src), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = Path::new(path).extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| format!("probe: {}", e))?;
    let format = probed.format;

    if let Some(track) = format.tracks().first() {
        if let Some(params) = track.codec_params.n_frames {
            if let Some(sr) = track.codec_params.sample_rate {
                return Ok(params as f64 / sr as f64);
            }
        }
    }
    Err("no duration available".into())
}

/// ffmpeg fallback for unsupported formats.
fn extract_audio_ffmpeg(input_path: &str, output_path: &str) -> Result<(), String> {
    let ffmpeg = crate::commands::transcribe::find_tool("ffmpeg")
        .ok_or("ffmpeg not found (needed for this format)")?;
    let status = std::process::Command::new(&ffmpeg)
        .args([
            "-y", "-i", input_path, "-ar", "16000", "-ac", "1",
            "-sample_fmt", "s16", "-f", "wav", output_path,
        ])
        .status()
        .map_err(|e| format!("ffmpeg failed: {}", e))?;
    if !status.success() {
        return Err("ffmpeg exited with error".into());
    }
    Ok(())
}
