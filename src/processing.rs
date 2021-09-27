use crate::visualizer::Visualizer;
use apodize;

pub const FFT_LENGTH: usize = 1024;
pub const FFT_OUT_LENGTH: usize = FFT_LENGTH / 2;

pub enum Window {
    None,
    Blackman,
    Hamming,
    Hanning,
    Nuttall,
}

pub struct Processor {
    window: Option<Vec<f32>>,
    visualizer: Box<dyn Visualizer + Send + Sync>,
    led_count: usize,
    sample_buffer: Box<[f32; FFT_OUT_LENGTH]>,
    render_buffer: Vec<u8>,
}

impl Processor {
    pub fn new(
        window_mode: Window,
        visualizer: Box<dyn Visualizer + Send + Sync>,
        led_count: usize,
    ) -> Processor {
        let window = Some(window_mode)
            .and_then(|w| match w {
                Window::None => None,
                Window::Blackman => Some(apodize::blackman_iter(FFT_LENGTH)),
                Window::Hamming => Some(apodize::hamming_iter(FFT_LENGTH)),
                Window::Hanning => Some(apodize::hanning_iter(FFT_LENGTH)),
                Window::Nuttall => Some(apodize::nuttall_iter(FFT_LENGTH)),
            })
            .map(|w| w.map(|f| f as f32).collect::<Vec<f32>>());

        Processor {
            window,
            visualizer,
            led_count,
            sample_buffer: Box::new([0.0; FFT_OUT_LENGTH]),
            render_buffer: vec![0; led_count * 3],
        }
    }

    #[inline]
    pub fn convert_samples(&mut self, input: &mut [f32; FFT_LENGTH]) -> &[f32; FFT_OUT_LENGTH] {
        self.visualizer
            .convert_samples(input, &mut self.sample_buffer, &self.window);

        &self.sample_buffer
    }

    #[inline]
    pub fn render_samples(&mut self, input: &mut [f32; FFT_OUT_LENGTH]) -> &[u8] {
        self.visualizer
            .render_samples(input, &mut self.render_buffer);

        &self.render_buffer
    }

    #[inline]
    pub fn process_samples(&mut self, input: &mut [f32; FFT_LENGTH]) -> &[u8] {
        self.visualizer
            .convert_samples(input, &mut self.sample_buffer, &self.window);

        self.visualizer
            .render_samples(&mut self.sample_buffer, &mut self.render_buffer);

        &self.render_buffer
    }

    /// Get a reference to the window if one is in use
    pub fn window(&self) -> Option<&[f32]> {
        self.window.as_ref().map(|v| v.as_ref())
    }
}
