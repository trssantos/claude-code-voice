use anyhow::{Context, Result};
use tracing::info;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct Transcriber {
    ctx: WhisperContext,
}

impl Transcriber {
    pub fn new(model_size: &str) -> Result<Self> {
        let model_path = crate::model::get_model_path(model_size)?;

        if !model_path.exists() {
            anyhow::bail!(
                "Model not found at {}. Run 'claude-code-voice download {}' first.",
                model_path.display(),
                model_size
            );
        }

        info!("Loading Whisper model from: {}", model_path.display());

        let ctx = WhisperContext::new_with_params(
            model_path.to_str().unwrap(),
            WhisperContextParameters::default(),
        )
        .context("Failed to load Whisper model")?;

        info!("Whisper model loaded successfully");

        Ok(Self { ctx })
    }

    pub async fn transcribe(&self, audio_samples: &[f32]) -> Result<String> {
        if audio_samples.is_empty() {
            return Ok(String::new());
        }

        // Resample to 16kHz if needed (Whisper requirement)
        let resampled = self.resample_to_16khz(audio_samples);

        // Create transcription parameters
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        // Optimize for speed
        params.set_n_threads(num_cpus::get() as i32);
        params.set_translate(false);
        params.set_language(Some("en"));
        params.set_print_progress(false);
        params.set_print_special(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        // Run transcription
        let mut state = self.ctx.create_state()?;
        state
            .full(params, &resampled)
            .context("Transcription failed")?;

        // Get the transcribed text
        let num_segments = state.full_n_segments()?;
        let mut result = String::new();

        for i in 0..num_segments {
            let segment = state.full_get_segment_text(i)?;
            result.push_str(&segment);
            result.push(' ');
        }

        let result = result.trim().to_string();

        Ok(result)
    }

    fn resample_to_16khz(&self, samples: &[f32]) -> Vec<f32> {
        // Simple decimation resampling
        // In production, you'd use a proper resampler like rubato
        // For now, assume input is already close to 16kHz or use simple decimation

        // Most modern audio devices use 44100 or 48000 Hz
        // We'll assume 48000 and decimate by 3 to get 16000
        let input_rate = 48000;
        let output_rate = 16000;
        let ratio = input_rate / output_rate;

        if ratio == 1 {
            return samples.to_vec();
        }

        let mut resampled = Vec::with_capacity(samples.len() / ratio);
        for i in (0..samples.len()).step_by(ratio) {
            resampled.push(samples[i]);
        }

        resampled
    }
}
