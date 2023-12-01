use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{
    default_host, Device, FromSample, Host, OutputCallbackInfo, SampleFormat, SizedSample,
    StreamConfig, SupportedStreamConfig,
};
use rand::random;
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct Audio {
    host: Host,
    pub device: Device,
    pub config: SupportedStreamConfig,
}

pub enum AudioPacket {
    Sample(f32),
    Exiting,
}

impl Default for Audio {
    fn default() -> Self {
        let host = default_host();
        let device = host.default_output_device().unwrap();
        let config = device.default_output_config().unwrap();

        Self {
            host,
            device,
            config,
        }
    }
}

pub fn run_audio(device: Device, config: SupportedStreamConfig) -> Sender<AudioPacket> {
    let sample_format = config.sample_format();
    match sample_format {
        SampleFormat::I8 => run_audio_stream::<i8>(device, config.config()),
        SampleFormat::I16 => run_audio_stream::<i16>(device, config.config()),
        SampleFormat::I32 => run_audio_stream::<i32>(device, config.config()),
        SampleFormat::I64 => run_audio_stream::<i64>(device, config.config()),
        SampleFormat::U8 => run_audio_stream::<u8>(device, config.config()),
        SampleFormat::U16 => run_audio_stream::<u16>(device, config.config()),
        SampleFormat::U32 => run_audio_stream::<u32>(device, config.config()),
        SampleFormat::U64 => run_audio_stream::<u64>(device, config.config()),
        SampleFormat::F32 => run_audio_stream::<f32>(device, config.config()),
        SampleFormat::F64 => run_audio_stream::<f64>(device, config.config()),
        _ => panic!("unsupported audio streaming format {sample_format}"),
    }
}

pub fn run_audio_stream<T>(device: Device, config: StreamConfig) -> Sender<AudioPacket>
where
    T: SizedSample + FromSample<f32>,
{
    let mut sample_buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let mut other_buffer = sample_buffer.clone();

    let (tx, rx) = channel::<AudioPacket>();
    let channels = config.channels;

    std::thread::spawn(move || {
        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [T], _: &OutputCallbackInfo| {
                    for frame in data.chunks_mut(channels as usize) {
                        let mut sample = None;
                        loop {
                            let s = other_buffer.lock().unwrap().pop();
                            if s.is_some() {
                                sample = s;
                                break;
                            } else {
                                std::thread::yield_now();
                            }
                        }
                        let value = T::from_sample(sample.unwrap());
                        for sample in frame.iter_mut() {
                            *sample = value;
                        }
                    }
                },
                |_| {},
                None,
            )
            .unwrap();

        stream.play().unwrap();
        loop {
            match rx.try_recv() {
                Ok(packet) => match packet {
                    AudioPacket::Sample(sample) => {
                        let mut buffer = sample_buffer.lock().unwrap();
                        if buffer.len() < 32 {
                            for _ in 0..channels {
                                buffer.push(sample);
                            }
                        }
                    }
                    AudioPacket::Exiting => break,
                },
                Err(_) => {}
            }
            // std::thread::sleep(Duration::from_millis(100));
        }
    });

    tx
}
