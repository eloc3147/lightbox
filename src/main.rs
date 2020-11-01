mod cmap;

use cmap::COLORMAP;
use std::fs::File;
use std::io::BufReader;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};

use blinkt::Blinkt;

use rodio::source::Buffered;
use rodio::{Decoder, Device, Source};

use cpal::traits::{DeviceTrait, HostTrait};
use thread_priority::unix::{
    set_thread_priority_and_policy, thread_native_id, RealtimeThreadSchedulePolicy,
    ThreadSchedulePolicy,
};
use thread_priority::ThreadPriority;

use average::Mean;

const LED_COUNT: usize = 144;
const LIGHT_BUF_LEN: usize = 256;
const LIGHT_AVG_DURATION: f32 = 0.25;
const LIGHT_SAMPLE_RATE: usize = 30;
const VOLUME_MUL: f32 = 1.0 / 10000.0;

const LIGHT_AVG_COUNT: usize = (LIGHT_AVG_DURATION * LIGHT_SAMPLE_RATE as f32) as usize;

fn light_loop(
    source: Buffered<Decoder<BufReader<File>>>,
    exit: Arc<AtomicBool>,
    time_rx: mpsc::Receiver<Instant>,
) {
    let sample_rate = dbg!(source.sample_rate());
    let led_sample_rate = dbg!(sample_rate / LIGHT_SAMPLE_RATE as u32);
    let mut sample_buf = [0f32; LIGHT_BUF_LEN];
    let mut averaging_buf = [0f64; LIGHT_BUF_LEN / 2 * LIGHT_AVG_COUNT];
    let mut avg_index = 0usize;

    set_thread_priority_and_policy(
        thread_native_id(),
        ThreadPriority::Max,
        ThreadSchedulePolicy::Realtime(RealtimeThreadSchedulePolicy::RoundRobin),
    )
    .unwrap();

    let mut blinkt = Blinkt::with_spi(16_000_000, LED_COUNT).unwrap();
    blinkt.set_all_pixels_brightness(1.0);

    let start_time = time_rx.recv().unwrap();

    let mut sample_count = 0u128;
    for sample in source {
        if exit.load(Ordering::Relaxed) {
            break;
        }

        let sample_index = (sample_count % led_sample_rate as u128) as usize;

        if sample_index < LIGHT_BUF_LEN {
            sample_buf[sample_index] = sample as f32;
        }

        if sample_index == LIGHT_BUF_LEN - 1 {
            let spectrum = microfft::real::rfft_256(&mut sample_buf);

            for (i, sample) in spectrum.iter().enumerate() {
                let bin_index = i * LIGHT_AVG_COUNT;
                averaging_buf[bin_index + (avg_index % LIGHT_AVG_COUNT)] =
                    (sample.log(10.0).norm() / 10.0) as f64;
                let mut value = averaging_buf[(bin_index)..(bin_index + LIGHT_AVG_COUNT)]
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
                    (0.5 + value / 2.0) as f32,
                );

                avg_index = avg_index.wrapping_add(1);
            }
            blinkt.show().unwrap();
        }

        if let Some(duration) =
            Duration::from_micros((sample_count * 1000000 / sample_rate as u128) as u64)
                .checked_sub(start_time.elapsed())
        {
            thread::sleep(duration);
        }

        sample_count += 1;
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
    let (time_tx, time_rx) = mpsc::sync_channel(0);
    let buf = source.clone();
    let e = exit.clone();
    println!("Starting light loop");
    let led_thread = thread::spawn(move || {
        light_loop(buf, e, time_rx);
    });

    // Play audio
    sink.append(source);
    sink.play();
    time_tx.send(Instant::now()).unwrap();

    led_thread.join().unwrap();
    println!("Exiting");
}
