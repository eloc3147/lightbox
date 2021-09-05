use std::convert::TryInto;
use std::fs::File;
use std::io::{BufWriter, Write};

use average::Mean;
use blinkt::Blinkt;
use const_format::formatcp;

use lightbox::processing::{Processor, FFT_LENGTH};

pub struct LEDConfig {
    /// Number of LEDs in the chain
    pub led_count: usize,
    // Duration over which to average FFT buckets
    // average_duration: f32,
    /// Size of averaging window
    pub average_count: usize,
    /// Color mapping
    pub color_map: &'static [[u8; 3]; 256],

    /// Whether to apply a Hamming window to the incoming samples
    pub hamming_window: bool,
}

pub struct LEDControler {
    leds: Blinkt,
    processor: Processor,
    sample_buf: Box<[f32]>,
    sample_index: usize,
    averaging_buf: Vec<f64>,
    color_map: &'static [[u8; 3]; 256],
    average_count: usize,
    average_index: usize,
    raw_sample_f: BufWriter<File>,
    fft_sample_f: BufWriter<File>,
}

impl LEDControler {
    pub fn new(config: LEDConfig) -> LEDControler {
        // Setup LEDs
        let mut leds = Blinkt::with_spi(16_000_000, config.led_count).unwrap();
        leds.set_all_pixels_brightness(1.0);

        let raw_sample_f =
            BufWriter::new(File::create("raw_samples.bin").expect("Unable to create sample file"));
        let fft_sample_f =
            BufWriter::new(File::create("fft_samples.bin").expect("Unable to create sample file"));

        let processor = match config.hamming_window {
            true => Processor::new_with_hamming(),
            false => Processor::new_without_window(),
        };

        LEDControler {
            leds,
            processor,
            sample_buf: Box::new([0f32; FFT_LENGTH]),
            sample_index: 0,
            averaging_buf: vec![0f64; FFT_LENGTH / 2 * config.average_count],
            color_map: config.color_map,
            average_count: config.average_count,
            average_index: 0,
            raw_sample_f,
            fft_sample_f,
        }
    }

    pub fn feed_samples(&mut self, samples: &[f32]) {
        // Currently don't skip any samples
        // Probably want to change that at some point
        //
        // Also assuming sampels are fed at the correct speed
        // No speed limiting in LED loop
        for sample in samples.iter() {
            self.sample_buf[self.sample_index] = *sample;
            if self.sample_index == FFT_LENGTH - 1 {
                self.render_spectrum();
                self.sample_index = 0;
            } else {
                self.sample_index += 1;
            }
        }
        // println!(
        //     "Fed {} samples, sample index is now {}",
        //     samples.len(),
        //     self.sample_index
        // );
    }

    fn render_spectrum(&mut self) {
        let samples: &mut [f32; FFT_LENGTH] = &mut self.sample_buf.as_ref().try_into().expect(
            formatcp!("Expected sample_buf to be {} in length", FFT_LENGTH),
        );
        for sample in samples.iter() {
            self.raw_sample_f
                .write(&sample.to_be_bytes())
                .expect("Unable to write sample");
        }
        let spectrum = self.processor.process(samples);

        // Samples are grouped by average, spaced out by bin index
        // eg: BIN1_AVG1, BIN1_AVG2, BIN1_AVG3, BIN2_AVG1 etc
        for (bin_num, mut sample) in IntoIterator::into_iter(spectrum).enumerate() {
            self.fft_sample_f
                .write(&sample.to_be_bytes())
                .expect("Unable to write sample");
            // Index to start of bin
            // let bin_index = bin_num * self.average_count;

            // self.averaging_buf[bin_index + self.average_index] = *sample;

            // Slice of averaging values of bin
            // let mut value = self.averaging_buf[bin_index..(bin_index + self.average_count)]
            //     .iter()
            //     .collect::<Mean>()
            //     .mean();

            // Clamp value to <= 1
            if sample > 1.0 {
                sample = 1.0;
            }
            // let color = self.map_to_color(value);

            self.leds.set_pixel_rgbb(
                bin_num + 8,
                255, // color[0],
                0,   // color[1],
                0,   // color[2],
                sample,
            );
        }
        self.leds.show().unwrap();
        self.average_index = self.average_index.wrapping_add(1) % self.average_count;
    }

    #[inline]
    fn map_to_color(&self, value: f64) -> [u8; 3] {
        self.color_map[(value * (self.color_map.len() - 1) as f64) as usize]
    }
}

impl Drop for LEDControler {
    fn drop(&mut self) {
        self.leds.set_all_pixels_brightness(0.0);
        println!("FFT_LEN = {}", FFT_LENGTH);
    }
}
