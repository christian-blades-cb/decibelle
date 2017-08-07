extern crate signalbool;
extern crate portaudio;
#[macro_use]
extern crate log;
extern crate env_logger;

use std::{thread,time};

const SAMPLE_RATE: f64 = 44_100.0;
const CHANNELS: i32 = 2;
const FRAMES: u32 = 256;
const INTERLEAVED: bool = true;

// 0.0790569415 - https://github.com/yorban/SmartSoundMeasurement/blob/master/app/src/main/java/com/csau/smartsound/splmeasurement/SmartSoundMeasurement.java
// 0.45255; // from https://github.com/johnliu55tw/ALSASoundMeter/blob/master/sound_meter.c
const K: f64 = 0.0790569415;
const BASE: f64 = 110.0; // very scientific, not calibrated at all

fn main() {
    env_logger::init().unwrap();
    let sb = signalbool::SignalBool::new(&[signalbool::Signal::SIGINT],
                                         signalbool::Flag::Interrupt)
        .unwrap();
    debug!("signal handling");

    let pa = portaudio::PortAudio::new().unwrap();
    info!("PortAudio version {}", pa.version());

    let input = pa.default_input_device().unwrap();
    let input_info = pa.device_info(input).unwrap();
    info!("Default input device {:#?}", input_info);

    // Construct the input stream parameters.
    let latency = input_info.default_low_input_latency;
    let input_params =
        portaudio::StreamParameters::<f32>::new(input, CHANNELS, INTERLEAVED, latency);
    let stream_settings = portaudio::stream::InputSettings::new(input_params, SAMPLE_RATE, FRAMES);

    let mut stream = pa.open_non_blocking_stream(stream_settings, audio_callback).unwrap();

    debug!("starting stream");
    stream.start().unwrap();

    loop {
        if sb.caught() {
            info!("signal caught!");
            break;
        }
        thread::sleep(time::Duration::from_millis(100));
    }
    
    stream.close().unwrap();
}

fn audio_callback(args: portaudio::stream::InputCallbackArgs<f32>) -> portaudio::stream::CallbackResult {
    if args.frames > 0 {
        debug!("frames: {}", args.frames);
        let sum_squares: f64 = args.buffer.into_iter().map(|&x| x as f64 * x as f64).sum();
        let rms = (sum_squares / args.buffer.len() as f64).sqrt();
        println!("rms: {}", rms);
    }
    return portaudio::Continue;
}
