use std::convert::TryInto;
use std::env;
use std::io::{BufReader, BufWriter, Read};
use std::{fs::File, io::Write};

use lightbox::processing::{process_samples, FFT_LENGTH};

const BATCH_SIZE: usize = 4;

fn main() {
    let mut input = BufReader::with_capacity(FFT_LENGTH, File::open("in.tmp").unwrap());
    let mut output = BufWriter::with_capacity(FFT_LENGTH, File::create("out.tmp").unwrap());

    let mut in_byte_buf = Vec::with_capacity(FFT_LENGTH * BATCH_SIZE);
    let mut sample_buf = [0f32; FFT_LENGTH];

    println!("Reading bytes");
    input.read_to_end(&mut in_byte_buf).unwrap();
    println!("Read {} bytes", in_byte_buf.len());

    let mut nan_count = 0;
    let mut inf_count = 0;

    for batch in in_byte_buf.chunks(FFT_LENGTH * BATCH_SIZE) {
        for (i, bytes) in batch.chunks(BATCH_SIZE).enumerate() {
            // print!("{}: ", i * 4);
            // print!("{:?} => ", &bytes);
            sample_buf[i] = f32::from_be_bytes(bytes.try_into().unwrap());
            // println!("{}", sample_buf[i]);
            if sample_buf[i].is_nan() {
                println!("NaN sample");
                sample_buf[i] = 0.0;
                nan_count += 1;
            } else if sample_buf[i].is_infinite() {
                println!("Inf sample");
                sample_buf[i] = 0.0;
                inf_count += 1;
            }
        }

        for sample in process_samples(&mut sample_buf).iter() {
            // println!(
            //     "re: {}, im: {}, cmp: {}, ampl: {}",
            //     sample.re, sample.im, sample, ampl
            // );
            output.write_all(&sample.to_be_bytes()).unwrap();
        }
    }
    dbg!(nan_count, inf_count);
}
