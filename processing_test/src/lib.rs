use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use std::convert::TryInto;
use std::fs::File;
use std::io::{BufReader, BufWriter, ErrorKind, Read, Write};
use std::path::Path;

use lightbox::processing::{Processor, FFT_LENGTH, FFT_OUT_LENGTH};

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

    fn process_chunks(&self, mut samples: Vec<f32>) -> PyResult<Vec<f32>> {
        let num_chunks = samples.len() / FFT_LENGTH;
        let mut out_samples = Vec::with_capacity(num_chunks * FFT_OUT_LENGTH);

        for chunk in samples.chunks_exact_mut(FFT_LENGTH) {
            let processed = self.processor.process(chunk.try_into().unwrap());
            out_samples.extend_from_slice(&processed);
        }

        Ok(out_samples)
    }

    fn window(&self) -> Option<Vec<f32>> {
        self.processor.window().map(|s| s.to_vec())
    }
}

#[pymodule]
fn processing_test(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<ProcessorInterface>()?;

    Ok(())
}
