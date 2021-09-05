use apodize;
use microfft::real::rfft_2048;

pub const FFT_LENGTH: usize = 2048;
pub const FFT_OUT_LENGTH: usize = FFT_LENGTH / 2;

pub struct Processor {
    window: Option<Vec<f32>>,
}

impl Processor {
    pub fn new_without_window() -> Processor {
        Processor { window: None }
    }

    pub fn new_with_hamming() -> Processor {
        Processor {
            window: Some(
                apodize::hamming_iter(FFT_LENGTH)
                    .map(|f| f as f32)
                    .collect::<Vec<f32>>(),
            ),
        }
    }

    pub fn process(&self, samples: &mut [f32; FFT_LENGTH]) -> [f32; FFT_OUT_LENGTH] {
        if let Some(window) = &self.window {
            for (i, sample) in samples.iter_mut().enumerate() {
                *sample *= window[i];
            }
        }

        let spectrum = rfft_2048(samples);
        // since the real-valued coefficient at the Nyquist frequency is packed into the
        // imaginary part of the DC bin, it must be cleared before computing the amplitudes
        spectrum[0].im = 0.0;

        let mut processed = [0f32; FFT_OUT_LENGTH];
        for (i, sample) in spectrum.iter().enumerate() {
            processed[i] = sample.norm_sqr().sqrt();
        }

        // TODO: Add nyquist filter

        processed
    }

    /// Get a reference to the window if one is in use
    pub fn window(&self) -> Option<&[f32]> {
        self.window.as_ref().map(|v| v.as_ref())
    }
}
