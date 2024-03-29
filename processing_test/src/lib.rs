use std::{convert::TryInto, path::PathBuf};

use image::imageops::FilterType;
use image::{self, DynamicImage, ImageBuffer};
use plotters::coord::Shift;
use plotters::prelude::*;
use pyo3::{exceptions::PyException, prelude::*};

use lightbox::processing::{Processor, Window, FFT_LENGTH, FFT_OUT_LENGTH};
use lightbox::visualizer::BasicFFT;

fn plot<X, Y, B>(
    area: &DrawingArea<B, Shift>,
    x_values: X,
    y_values: Y,
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
    title: &str,
) -> Result<(), DrawingAreaErrorKind<B::ErrorType>>
where
    B: DrawingBackend,
    X: IntoIterator<Item = f32>,
    Y: IntoIterator<Item = f32>,
{
    let mut cc = ChartBuilder::on(area)
        .margin(5)
        .set_all_label_area_size(50)
        .caption(title, ("sans-serif", 40))
        .build_cartesian_2d(min_x..max_x, min_y..max_y)?;

    cc.configure_mesh()
        .x_labels(20)
        .y_labels(10)
        .disable_mesh()
        .x_label_formatter(&|v| format!("{:.1}", v))
        .y_label_formatter(&|v| format!("{:.1}", v))
        .draw()?;

    cc.draw_series(LineSeries::new(x_values.into_iter().zip(y_values), &RED))?
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

    cc.configure_series_labels().border_style(&BLACK).draw()?;

    Ok(())
}

fn plot_image<B>(
    area: &DrawingArea<B, Shift>,
    rbg_data: Vec<u8>,
    title: &str,
) -> Result<(), DrawingAreaErrorKind<B::ErrorType>>
where
    B: DrawingBackend,
{
    let image_width = rbg_data.len() / 3;
    let mut cc = ChartBuilder::on(area)
        .margin(5)
        .set_all_label_area_size(50)
        .caption(title, ("sans-serif", 40))
        .build_cartesian_2d(0.0..image_width as f32, 0.0..1.0)?;

    let image_buffer =
        ImageBuffer::from_raw(image_width as u32, 1, rbg_data).expect("Image buffer too small");

    let (plot_width, plot_height) = cc.plotting_area().get_pixel_range();
    let image = DynamicImage::ImageRgb8(image_buffer).resize_exact(
        plot_width.len() as u32,
        plot_height.len() as u32,
        FilterType::Nearest,
    );

    cc.configure_mesh().disable_mesh().draw()?;

    let elem: BitMapElement<_> = ((0.0, 1.0), image).into();
    cc.draw_series(std::iter::once(elem))?;
    Ok(())
}

#[pyclass]
struct ProcessorInterface {
    processor: Processor,
    led_count: usize,
}

#[pymethods]
impl ProcessorInterface {
    #[new]
    fn init(py: Python, window: bool, led_count: usize) -> PyResult<Self> {
        py.allow_threads(|| {
            let window = match window {
                true => Window::Hamming,
                false => Window::None,
            };

            let processor = Processor::new(window, Box::new(BasicFFT::new()), led_count);

            Ok(ProcessorInterface {
                processor,
                led_count,
            })
        })
    }

    /// Convert one or more chunks with the lightbox processor
    /// Incomplete chunks will be cut short to the nearest complete chunk.
    fn convert_samples(&mut self, py: Python, mut samples: Vec<f32>) -> Vec<f32> {
        py.allow_threads(|| {
            let num_chunks = samples.len() / FFT_LENGTH;
            let mut out_samples = Vec::with_capacity(num_chunks * FFT_OUT_LENGTH);

            for input in samples.chunks_exact_mut(FFT_LENGTH) {
                out_samples.extend(self.processor.convert_samples(input.try_into().unwrap()));
            }

            out_samples
        })
    }

    /// Render one or more chunks with the lightbox processor
    /// Incomplete chunks will be cut short to the nearest complete chunk.
    fn render_led_view(&mut self, py: Python, mut samples: Vec<f32>) -> Vec<u8> {
        py.allow_threads(|| {
            let num_chunks = samples.len() / FFT_OUT_LENGTH;
            let mut out_samples = Vec::with_capacity(num_chunks * self.led_count * 3);

            for input in samples.chunks_exact_mut(FFT_OUT_LENGTH) {
                out_samples.extend(self.processor.render_samples(input.try_into().unwrap()));
            }

            out_samples
        })
    }

    /// Get the window applied to chunks before applying FFT, if any.
    fn window(&self, py: Python) -> Option<[f32; FFT_LENGTH]> {
        py.allow_threads(|| {
            self.processor.window().map(|s| {
                s.clone()
                    .try_into()
                    .expect("processor window is not FFT_LENGTH")
            })
        })
    }

    fn render_chunk(
        &self,
        py: Python,
        input_samples: Vec<f32>,
        sample_duration: f32,
        impl_samples: Vec<f32>,
        impl_freqs: Vec<f32>,
        led_view: Vec<u8>,
        output_path: String,
        width: u32,
        height: u32,
        min_y: f32,
        max_y: f32,
    ) -> PyResult<()> {
        py.allow_threads(|| {
            let out_path = PathBuf::from(output_path);
            let root_area = BitMapBackend::new(&out_path, (width, height)).into_drawing_area();

            root_area
                .fill(&WHITE)
                .map_err(|e| PyException::new_err(format!("Error doing thing A: {:?}", e)))?;

            let areas = root_area.split_evenly((3, 1));

            let max_x = input_samples.len() as f32 * sample_duration;
            plot(
                &areas[0],
                (0..input_samples.len()).map(|s| s as f32 * sample_duration),
                input_samples,
                0.0,
                max_x,
                -1.2,
                1.2,
                "Time domain signal",
            )
            .map_err(|e| {
                PyException::new_err(format!("Error rendering time domain plot: {:?}", e))
            })?;

            let min_x = impl_freqs[0];
            let max_x = impl_freqs[impl_freqs.len() - 1];
            let y_margin = (min_y - max_y).abs() * 0.05;

            plot(
                &areas[1],
                impl_freqs,
                impl_samples,
                min_x,
                max_x,
                min_y - y_margin,
                max_y + y_margin,
                "Impl Spectrum",
            )
            .map_err(|e| PyException::new_err(format!("Error rendering impl plot: {:?}", e)))?;

            plot_image(&areas[2], led_view, "LED View")
                .map_err(|e| PyException::new_err(format!("Error rendering LED plot: {:?}", e)))?;

            root_area.present().expect("Unable to write result to file");
            Ok(())
        })
    }
}

#[pymodule]
fn processing_test(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add("FFT_LENGTH", FFT_LENGTH)?;
    m.add("FFT_OUT_LENGTH", FFT_OUT_LENGTH)?;
    m.add_class::<ProcessorInterface>()?;

    Ok(())
}
