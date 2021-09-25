use plotters::coord::Shift;
use plotters::prelude::*;
use pyo3::{exceptions::PyException, prelude::*};
use std::{convert::TryInto, path::PathBuf};

use lightbox::processing::{Processor, FFT_LENGTH, FFT_OUT_LENGTH};

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

#[pyclass]
struct ProcessorInterface {
    processor: Processor,
}

#[pymethods]
impl ProcessorInterface {
    #[new]
    fn init(py: Python, window: bool) -> PyResult<Self> {
        py.allow_threads(|| {
            let processor = if window {
                Processor::new_with_hamming()
            } else {
                Processor::new_without_window()
            };

            Ok(ProcessorInterface { processor })
        })
    }

    /// Process one or more chunks with the lightbox processor
    /// Incomplete chunks will be cut short to the nearest complete chunk.
    fn process_chunks(&self, py: Python, mut samples: Vec<f32>) -> Vec<f32> {
        py.allow_threads(|| {
            let num_chunks = samples.len() / FFT_LENGTH;
            let mut out_samples = vec![0f32; num_chunks * FFT_OUT_LENGTH];

            for (input, output) in samples
                .chunks_exact_mut(FFT_LENGTH)
                .zip(out_samples.chunks_exact_mut(FFT_OUT_LENGTH))
            {
                self.processor
                    .process(input.try_into().unwrap(), output.try_into().unwrap());
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

            let areas = root_area.split_evenly((2, 1));

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
