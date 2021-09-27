use microfft::real::rfft_1024;

use crate::cmap::COLORMAP;
use crate::processing::{FFT_LENGTH, FFT_OUT_LENGTH};

pub trait Visualizer {
    fn convert_samples(
        &mut self,
        input: &mut [f32; FFT_LENGTH],
        output: &mut [f32; FFT_OUT_LENGTH],
        window: &Option<Vec<f32>>,
    );

    fn render_samples(&mut self, input: &mut [f32; FFT_OUT_LENGTH], output: &mut [u8]);
}

pub struct BasicFFT;

impl BasicFFT {
    pub fn new() -> Self {
        BasicFFT
    }
}

impl Visualizer for BasicFFT {
    fn convert_samples(
        &mut self,
        input: &mut [f32; FFT_LENGTH],
        output: &mut [f32; FFT_OUT_LENGTH],
        window: &Option<Vec<f32>>,
    ) {
        if let Some(window) = &window {
            for (i, sample) in input.iter_mut().enumerate() {
                *sample *= window[i];
            }
        }

        let spectrum = rfft_1024(input);

        // since the real-valued coefficient at the Nyquist frequency is packed into the
        // imaginary part of the DC bin, it must be cleared before computing the amplitudes
        spectrum[0].im = 0.0;

        for (i, sample) in spectrum.iter().enumerate() {
            // Add tiny value to prevent passing 0 to log10
            let real = sample.norm_sqr().sqrt() + 1e-10;

            // Convert to decibels
            output[i] = 10.0 * real.log10();
        }

        // TODO: Add nyquist filter?
    }

    fn render_samples(&mut self, input: &mut [f32; FFT_OUT_LENGTH], output: &mut [u8]) {
        let len = std::cmp::min(FFT_OUT_LENGTH, output.len() / 3);

        for (index, sample) in input[..len].iter().enumerate() {
            // Clip outside -20..30 and rescale to 0..256
            let sample = ((*sample + 20.0) / 50.0) * 256.0;
            let quantized = COLORMAP[sample as usize];
            output[(index * 3)] = quantized[0];
            output[(index * 3) + 1] = quantized[1];
            output[(index * 3) + 2] = quantized[2];
        }
    }
}
