use microfft::real::rfft_512;
use microfft::Complex32;

pub const FFT_LENGTH: usize = 512;

pub fn process_samples(samples: &mut [f32; FFT_LENGTH]) -> [f32; FFT_LENGTH / 2] {
    let spectrum = rfft_512(samples);
    // since the real-valued coefficient at the Nyquist frequency is packed into the
    // imaginary part of the DC bin, it must be cleared before computing the amplitudes
    spectrum[0].im = 0.0;

    let mut normalized = [0f32; FFT_LENGTH / 2];
    for (i, sample) in spectrum.iter().enumerate() {
        normalized[i] = sample.norm_sqr().log10();
    }

    normalized
}
