use microfft::real::rfft_1024;

use crate::processing::{FFT_LENGTH, FFT_OUT_LENGTH};

pub trait Visualizer {
    fn process(
        &mut self,
        input: &mut [f32; FFT_LENGTH],
        output: &mut [f32; FFT_OUT_LENGTH],
        window: &Option<Vec<f32>>,
    );
}

pub struct BasicFFT;

impl BasicFFT {
    pub fn new() -> Self {
        BasicFFT
    }
}

impl Visualizer for BasicFFT {
    fn process(
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
}
