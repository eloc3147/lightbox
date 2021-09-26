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
}

impl Processor {
    pub fn new(window_mode: Window, visualizer: Box<dyn Visualizer + Send + Sync>) -> Processor {
        let window = Some(window_mode)
            .and_then(|w| match w {
                Window::None => None,
                Window::Blackman => Some(apodize::blackman_iter(FFT_LENGTH)),
                Window::Hamming => Some(apodize::hamming_iter(FFT_LENGTH)),
                Window::Hanning => Some(apodize::hanning_iter(FFT_LENGTH)),
                Window::Nuttall => Some(apodize::nuttall_iter(FFT_LENGTH)),
            })
            .map(|w| w.map(|f| f as f32).collect::<Vec<f32>>());

        Processor { window, visualizer }
    }

    #[inline]
    pub fn process(&mut self, input: &mut [f32; FFT_LENGTH], output: &mut [f32; FFT_OUT_LENGTH]) {
        self.visualizer.process(input, output, &self.window)
    }

    /// Get a reference to the window if one is in use
    pub fn window(&self) -> Option<&[f32]> {
        self.window.as_ref().map(|v| v.as_ref())
    }
}
