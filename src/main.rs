mod cmap;

use cmap::COLORMAP;
use std::fs::File;
use std::io::BufReader;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use blinkt::Blinkt;

use rodio::source::Buffered;
use rodio::{Decoder, Device, Source};

use cpal::traits::{DeviceTrait, HostTrait};

use average::Mean;

const LIGHT_BUF_LEN: usize = 256;
const LIGHT_AVG_DURATION: f32 = 0.1;
const LIGHT_SAMPLE_RATE: usize = 30;
const VOLUME_MUL: f32 = 1.0 / 10000.0;

const LIGHT_AVG_COUNT: usize = (LIGHT_AVG_DURATION * LIGHT_SAMPLE_RATE as f32) as usize;
const LIGHT_SAMPLE_TIME: Duration = Duration::from_millis(1000 / LIGHT_SAMPLE_RATE as u64);

fn light_loop(mut source: Buffered<Decoder<BufReader<File>>>, exit: Arc<AtomicBool>) {
    let mut sample_buf = [0f32; LIGHT_BUF_LEN];
    let mut averaging_buf = [0f64; LIGHT_BUF_LEN / 2 * LIGHT_AVG_COUNT];
    let mut avg_index = 0usize;
    let led_count = 144;

    let mut blinkt = Blinkt::with_spi(16_000_000, led_count).unwrap();
    blinkt.set_all_pixels_brightness(1.0);

    // let ratio = led_count as f32 / TURBO_COLORMAP.len() as f32;
    // for i in 0..led_count {
    //     let color = TURBO_COLORMAP[(i as f32 * ratio) as usize];
    //     blinkt.set_pixel(i, color[0], color[1], color[2]);
    // }

    while !exit.load(Ordering::Relaxed) {
        let sample_start = Instant::now();

        for i in 0..LIGHT_BUF_LEN {
            match source.next() {
                Some(s) => sample_buf[i] = s as f32,
                None => return,
            }
        }

        let spectrum = microfft::real::rfft_256(&mut sample_buf);
        for (i, sample) in spectrum.iter().enumerate() {
            averaging_buf[i * LIGHT_AVG_COUNT + (avg_index % LIGHT_AVG_COUNT)] =
                (sample.l1_norm() * VOLUME_MUL) as f64;
            let mut value = averaging_buf[(i * LIGHT_AVG_COUNT)..((i + 1) * LIGHT_AVG_COUNT)]
                .iter()
                .collect::<Mean>()
                .mean();

            if value > 1.0 {
                value = 1.0;
            }
            let color = COLORMAP[(value * (COLORMAP.len() - 1) as f64) as usize];

            blinkt.set_pixel_rgbb(
                i + 8,
                color[0],
                color[1],
                color[2],
                (0.75 + value / 4.0) as f32,
            );
        }
        blinkt.show().unwrap();

        if let Some(duration) = LIGHT_SAMPLE_TIME.checked_sub(sample_start.elapsed()) {
            thread::sleep(duration);
        }

        avg_index = avg_index.wrapping_add(1);
    }

    blinkt.set_all_pixels_brightness(0.0);
}

fn main() {
    // Setup Ctrl-C handling
    let exit = Arc::new(AtomicBool::new(false));
    let e = exit.clone();
    ctrlc::set_handler(move || {
        e.store(true, Ordering::Relaxed);
    })
    .expect("Error setting Ctrl-C handler");

    // Find output device
    let host = cpal::default_host();
    let devices: Vec<_> = host.devices().unwrap().map(Box::new).collect();
    println!("Available hosts:");
    for device in devices.iter() {
        println!("\t{}", device.name().unwrap());
    }

    let device = match devices
        .into_iter()
        .filter(|d| d.name().unwrap().starts_with("plughw"))
        .next()
    {
        Some(d) => {
            println!("Selected device: {}", d.name().unwrap());
            Device::from(*d)
        }
        None => {
            println!("Device not found");
            return;
        }
    };

    // Stream wav
    let (_stream, stream_handle) = rodio::OutputStream::try_from_device(&device).unwrap();
    let sink = rodio::Sink::try_new(&stream_handle).unwrap();

    let file = File::open("sample.wav").unwrap();
    let source = rodio::Decoder::new(BufReader::new(file))
        .unwrap()
        .buffered();

    // Start LED control
    let buf = source.clone();
    let e = exit.clone();
    println!("Starting light loop");
    let led_thread = thread::spawn(move || {
        light_loop(buf, e);
    });

    // Play audio
    sink.append(source);
    sink.play();

    while !exit.load(Ordering::Relaxed) {}

    println!("Exiting");
    led_thread.join().unwrap()
}
