use std::convert::TryInto;
use std::fs::File;
use std::io::{BufReader, BufWriter, ErrorKind, Read, Write};
use std::path::Path;

use lightbox::processing::{Processor, FFT_LENGTH, FFT_OUT_LENGTH};

const ENCODED_NUM_BYTES: usize = 4;

fn read_samples<P>(file: P) -> Vec<f32>
where
    P: AsRef<Path>,
{
    let mut reader =
        BufReader::with_capacity(FFT_LENGTH, File::open(file).expect("File does not exist"));
    let mut samples = Vec::with_capacity(FFT_LENGTH);

    let mut byte_buf = [0u8; ENCODED_NUM_BYTES];
    loop {
        match reader.read_exact(&mut byte_buf) {
            Ok(_) => {}
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => return samples,
            Err(e) => panic!("{:?}", e),
        }
        samples.push(f32::from_be_bytes(byte_buf));
    }
}

fn write_samples<P>(file: P, samples: &[f32])
where
    P: AsRef<Path>,
{
    let mut writer = BufWriter::with_capacity(
        FFT_LENGTH,
        File::create(file).expect("Error creating sample file"),
    );

    for sample in samples {
        writer.write_all(&sample.to_be_bytes()).unwrap();
    }
}

fn main() {
    let mut samples = read_samples("in.tmp");
    let num_chunks = samples.len() / FFT_LENGTH;

    let processor = Processor::new_with_hamming();

    write_samples("window.tmp", processor.window().unwrap());

    let mut out_samples = Vec::with_capacity(num_chunks * FFT_OUT_LENGTH);

    for chunk in samples.chunks_mut(FFT_LENGTH) {
        let processed = processor.process(chunk.try_into().unwrap());
        out_samples.extend_from_slice(&processed);
    }

    write_samples("out.tmp", &out_samples);
}
