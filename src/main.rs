use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};

use blinkt::Blinkt;

fn hsv_to_rgb(h: u16, s: u16, v: u8) -> (u8, u8, u8) {
    let f = (h % 60) * 255 / 60;

    let p = ((255 - s) * v as u16 / 255) as u8;
    let q = ((255 - f * s / 255) * v as u16 / 255) as u8;
    let t = ((255 - (255 - f) * s / 255) * v as u16 / 255) as u8;

    match (h / 60) % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        5 => (v, p, q),
        _ => (0, 0, 0),
    }
}

fn main() {
    let led_count = 144;
    let rainbow_width = 2;
    let rainbow_ratio = 360.0 / (led_count * rainbow_width) as f32;
    let exit = Arc::new(AtomicBool::new(false));
    let e = exit.clone();

    ctrlc::set_handler(move || {
        e.store(true, Ordering::Relaxed);
    })
    .expect("Error setting Ctrl-C handler");

    let mut blinkt = Blinkt::with_spi(16_000_000, led_count).unwrap();
    blinkt.set_all_pixels_brightness(0.5);

    while !exit.load(Ordering::Relaxed) {
        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Unable to get system time")
            .as_millis() as usize
            / 6;

        for pixel in 0..led_count {
            let offset = (pixel as f32 * rainbow_ratio).round() as usize;
            let h = ((time + offset) % 360) as u16;
            print!("{}, ", h);
            let (r, g, b) = hsv_to_rgb(h, 255, 255);

            blinkt.set_pixel(pixel, r, g, b)
        }
        println!("\n");

        blinkt.show().unwrap();

        thread::sleep(Duration::from_millis(10));
    }

    println!("Exiting");
    blinkt.set_all_pixels_brightness(0.0);
}
