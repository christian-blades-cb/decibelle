extern crate alsa_sys;
extern crate dsp;
extern crate time;
extern crate libc;
extern crate signalbool;
#[macro_use]
extern crate log;
extern crate env_logger;

use std::{mem, ffi};

fn main() {
    env_logger::init().unwrap();
    let sb = signalbool::SignalBool::new(&[signalbool::Signal::SIGINT],
                                         signalbool::Flag::Interrupt)
        .unwrap();
    debug!("signal handling");

    unsafe {
        let mut handle: *mut alsa_sys::snd_pcm_t = mem::uninitialized();
        let device = ffi::CString::new("default").expect("unable to get device name");
        let err = alsa_sys::snd_pcm_open(&mut handle,
                                         device.as_ptr(),
                                         alsa_sys::SND_PCM_STREAM_CAPTURE,
                                         alsa_sys::SND_PCM_NONBLOCK);
        if err < 0 {
            panic!("device not available");
        }

        let err = alsa_sys::snd_pcm_set_params(handle,
                                               alsa_sys::SND_PCM_FORMAT_S16_LE,
                                               alsa_sys::SND_PCM_ACCESS_RW_INTERLEAVED,
                                               1,
                                               48000,
                                               1,
                                               500000);
        if err < 0 {
            alsa_sys::snd_pcm_close(handle);
            panic!("unable to setup pcm capture");
        }

        debug!("setup complete");

        let buffer: &[i16] = &[0; 8 * 1024];
        let bufptr = buffer.as_ptr() as *mut libc::c_void;
        let bufsize = buffer.len() as alsa_sys::snd_pcm_uframes_t;

        loop {
            if sb.caught() {
                println!("signal caught!");
                break;
            }

            let frames = alsa_sys::snd_pcm_readi(handle, bufptr, bufsize);
            debug!("frames {}", frames);
            let sums: i16 = (0..frames).map(|x| buffer[x as usize] * buffer[x as usize]).sum();
            let rms = (sums as f64).sqrt();
            format!("{}\n", rms);
            info!("loop'd");
        }

        alsa_sys::snd_pcm_close(handle);
    }
}
