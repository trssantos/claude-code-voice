use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream, StreamConfig};
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

pub struct AudioCapturer {
    device: Device,
    config: StreamConfig,
}

impl AudioCapturer {
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .context("No input device available")?;

        info!("Using audio input device: {}", device.name()?);

        let config = device
            .default_input_config()
            .context("Failed to get default input config")?;

        info!(
            "Audio config: {} Hz, {} channels, {:?}",
            config.sample_rate().0,
            config.channels(),
            config.sample_format()
        );

        Ok(Self {
            device,
            config: config.into(),
        })
    }

    pub async fn capture_while_held(
        &self,
        hotkey_manager: &crate::hotkey::HotkeyManager,
    ) -> Result<Vec<f32>> {
        let samples = Arc::new(Mutex::new(Vec::new()));
        let samples_clone = samples.clone();

        let err_fn = |err| {
            warn!("Audio stream error: {}", err);
        };

        // Build the audio stream
        let stream = match self.config.sample_format() {
            SampleFormat::F32 => self.build_stream::<f32>(samples_clone, err_fn)?,
            SampleFormat::I16 => self.build_stream::<i16>(samples_clone, err_fn)?,
            SampleFormat::U16 => self.build_stream::<u16>(samples_clone, err_fn)?,
            format => anyhow::bail!("Unsupported sample format: {:?}", format),
        };

        stream.play()?;

        // Wait for hotkey release
        hotkey_manager.wait_for_release().await?;

        // Stop the stream
        drop(stream);

        // Get the captured samples
        let captured_samples = samples.lock().unwrap().clone();

        info!("Captured {} audio samples", captured_samples.len());

        Ok(captured_samples)
    }

    fn build_stream<T>(
        &self,
        samples: Arc<Mutex<Vec<f32>>>,
        err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
    ) -> Result<Stream>
    where
        T: cpal::Sample + cpal::SizedSample,
    {
        let config = self.config.clone();

        let stream = self.device.build_input_stream(
            &config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                let mut samples = samples.lock().unwrap();
                for &sample in data {
                    samples.push(sample.to_float_sample());
                }
            },
            err_fn,
            None,
        )?;

        Ok(stream)
    }
}

// Helper trait to convert samples to f32
trait ToFloatSample {
    fn to_float_sample(&self) -> f32;
}

impl ToFloatSample for f32 {
    fn to_float_sample(&self) -> f32 {
        *self
    }
}

impl ToFloatSample for i16 {
    fn to_float_sample(&self) -> f32 {
        *self as f32 / i16::MAX as f32
    }
}

impl ToFloatSample for u16 {
    fn to_float_sample(&self) -> f32 {
        (*self as f32 - 32768.0) / 32768.0
    }
}
