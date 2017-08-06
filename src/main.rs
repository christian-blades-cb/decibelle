extern crate signalbool;
extern crate portaudio;
#[macro_use]
extern crate log;
extern crate env_logger;

// use std::collections::VecDeque;

const SAMPLE_RATE: f64 = 44_100.0;
const CHANNELS: i32 = 2;
const FRAMES: u32 = 256;
const INTERLEAVED: bool = true;

const K: f64 = 0.45255; // from https://github.com/johnliu55tw/ALSASoundMeter/blob/master/sound_meter.c

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
    let mut stream = pa.open_blocking_stream(stream_settings).unwrap();

    // let mut buffer: VecDeque<f32> = VecDeque::with_capacity(FRAMES as usize * CHANNELS as usize);

    debug!("starting stream");
    stream.start().unwrap();

    // We'll use this function to wait for read/write availability.
    fn wait_for_stream<F>(f: F, name: &str) -> u32
        where F: Fn() -> Result<portaudio::StreamAvailable, portaudio::error::Error>
    {
        'waiting_for_stream: loop {
            match f() {
                Ok(available) => {
                    match available {
                        portaudio::StreamAvailable::Frames(frames) => return frames as u32,
                        portaudio::StreamAvailable::InputOverflowed => {
                            println!("Input stream has overflowed")
                        }
                        portaudio::StreamAvailable::OutputUnderflowed => {
                            println!("Output stream has underflowed")
                        }
                    }
                }
                Err(err) => {
                    panic!("An error occurred while waiting for the {} stream: {}",
                           name,
                           err)
                }
            }
        }
    };

    'stream: loop {
        if sb.caught() {
            println!("caught signal!");
            break;
        }

        let frames = wait_for_stream(|| stream.read_available(), "Read");
        if frames > 0 {
            let samples: &[f32] = stream.read(frames).unwrap();
            let sum_squares: f64 = samples.into_iter().map(|&x| x as f64 * x as f64).sum();
            let rms = (sum_squares / samples.len() as f64).sqrt();
            println!("rms: {} db: {}", rms, 20.0 * (K * rms).log(10.0));
        }
    }

    stream.close().unwrap();
}
