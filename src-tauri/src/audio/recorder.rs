use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, SizedSample, Stream, StreamConfig};
use hound::{SampleFormat as WavSampleFormat, WavSpec, WavWriter};

type SharedWriter = Arc<Mutex<Option<WavWriter<BufWriter<File>>>>>;

pub struct Recorder {
    stream: Option<Stream>,
    writer: Option<SharedWriter>,
}

impl Recorder {
    pub fn new() -> Result<Self> {
        Ok(Self {
            stream: None,
            writer: None,
        })
    }

    pub fn is_recording(&self) -> bool {
        self.stream.is_some()
    }

    pub fn start(&mut self, output_path: PathBuf) -> Result<()> {
        if self.stream.is_some() {
            return Ok(());
        }

        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .context("no default microphone found")?;
        let config = device
            .default_input_config()
            .context("failed to read microphone config")?;

        let wav_spec = WavSpec {
            channels: config.channels(),
            sample_rate: config.sample_rate(),
            bits_per_sample: 16,
            sample_format: WavSampleFormat::Int,
        };
        let writer = Arc::new(Mutex::new(Some(
            WavWriter::create(output_path, wav_spec).context("failed to create wav file")?,
        )));

        let stream = match config.sample_format() {
            SampleFormat::I8 => {
                self.build_input_stream::<i8>(&device, &config.into(), writer.clone())?
            }
            SampleFormat::I16 => {
                self.build_input_stream::<i16>(&device, &config.into(), writer.clone())?
            }
            SampleFormat::I32 => {
                self.build_input_stream::<i32>(&device, &config.into(), writer.clone())?
            }
            SampleFormat::U8 => {
                self.build_input_stream::<u8>(&device, &config.into(), writer.clone())?
            }
            SampleFormat::U16 => {
                self.build_input_stream::<u16>(&device, &config.into(), writer.clone())?
            }
            SampleFormat::U32 => {
                self.build_input_stream::<u32>(&device, &config.into(), writer.clone())?
            }
            SampleFormat::F32 => {
                self.build_input_stream::<f32>(&device, &config.into(), writer.clone())?
            }
            SampleFormat::F64 => {
                self.build_input_stream::<f64>(&device, &config.into(), writer.clone())?
            }
            _ => anyhow::bail!("unsupported microphone format"),
        };

        stream.play().context("failed to start microphone stream")?;
        self.stream = Some(stream);
        self.writer = Some(writer);
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        self.stream.take();
        if let Some(writer) = self.writer.take() {
            let mut lock = writer
                .lock()
                .map_err(|_| anyhow::anyhow!("poisoned writer"))?;
            if let Some(writer) = lock.take() {
                writer.finalize().context("failed to finalise wav file")?;
            }
        }
        Ok(())
    }

    fn build_input_stream<T>(
        &self,
        device: &cpal::Device,
        config: &StreamConfig,
        writer: SharedWriter,
    ) -> Result<Stream>
    where
        T: Sample + SizedSample + Send + 'static,
        i16: FromSample<T>,
    {
        let stream = device.build_input_stream(
            config,
            move |data: &[T], _| {
                if let Ok(mut lock) = writer.lock() {
                    if let Some(writer) = lock.as_mut() {
                        for sample in data {
                            let sample_i16 = i16::from_sample(*sample);
                            let _ = writer.write_sample(sample_i16);
                        }
                    }
                }
            },
            move |err| {
                log::error!("microphone stream error: {err}");
            },
            None,
        )?;
        Ok(stream)
    }
}
