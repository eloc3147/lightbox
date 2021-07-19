from __future__ import annotations

import struct
import subprocess

from matplotlib import pyplot as plt
import matplotlib.gridspec as gridspec
import numpy as np

FFT_EXEC = r"E:\cargo_target\release\processing_test.exe"

# Input constants
NUM_BYTES = 4  # Number of bytes per sample (currently f32)
FFT_SIZE = 512 # Number of samples in FFT
SAMPLE_RATE = 44_100 # Audio sample rate, in Hz

DTYPE = ">f4"

# Calculated constants
FFT_OUT_SIZE = FFT_SIZE // 2
SAMPLE_TIME = 1 / SAMPLE_RATE
CHUNK_TIME = SAMPLE_TIME * FFT_SIZE


# def do_plot_grid(sample_idex: int, raw_samples: np.ndarray[np.float32], fft_samples: np.ndarray[np.float32]) -> None:
#     fig = plt.figure(constrained_layout=True)
#     # gs = gridspec.GridSpec(1 + np.ceil(NUM_FFTS / 4).astype(np.uint), 4, figure=fig)
# 
#     ax1 = fig.add_subplot(gs[0, :])
#     ax1.plot(np.arange(sample_idex, sample_idex + len(raw_samples), dtype=np.float32) * SAMPLE_TIME, raw_samples)
# 
#     ax2 = fig.add_subplot(gs[1, 0])
#     ax2.plot(fft_samples[0:FFT_OUT_SIZE])
#     print(fft_samples[0:4])
# 
#     # for fft_num in range(1, NUM_FFTS):
#     #     print(fft_samples[fft_num * FFT_OUT_SIZE:(fft_num * FFT_OUT_SIZE) + 3])
#     #     print("PC: ", 1 + (fft_num // 4), fft_num % 4)
#     #     ax = fig.add_subplot(gs[1 + (fft_num // 4), fft_num % 4], sharey=ax2)
#     #     ax.plot(fft_samples[fft_num * FFT_OUT_SIZE:(fft_num + 1) * FFT_OUT_SIZE])
# 
#     plt.show()
 
def do_plot(sample_idex: int, freqs: np.ndarray[np.float32], raw_samples: np.ndarray[np.float32], fft_samples: np.ndarray[np.float32]) -> None:
    fig = plt.figure(constrained_layout=True)
    gs = gridspec.GridSpec(2, 1, figure=fig)

    ax1 = fig.add_subplot(gs[0, 0])
    ax1.plot(np.arange(sample_idex, sample_idex + len(raw_samples), dtype=np.float32) * SAMPLE_TIME, raw_samples)

    ax2 = fig.add_subplot(gs[1, 0])
    ax2.plot(freqs, fft_samples[0:FFT_OUT_SIZE])

    ax2.set_xscale('log')
    ax2.set_yscale('log')
    plt.show()


def process_chunk(raw_samples: np.ndarray[np.float32], sample_index: int) -> None:
    print("Processing {0} samples: [{1}, ...]".format(len(raw_samples), ", ".join(map(str, raw_samples[:5]))))
    raw_samples.tofile("in.tmp")
    print("Subproc output:", subprocess.check_output([FFT_EXEC]).decode())

    fft_samples = np.fromfile("out.tmp", dtype=DTYPE)
    print("FFT len: ", len(fft_samples))

    freqs = SAMPLE_RATE * np.arange((FFT_SIZE / 2)) / FFT_SIZE

    do_plot(sample_index, freqs, raw_samples, fft_samples)


def process_file() -> None:
    with open("raw.bin", "rb") as rs:
        sample_index = 0
        while True:
            raw_bytes = rs.read(FFT_SIZE * NUM_BYTES)
            print("Raw len: ", len(raw_bytes))
            raw_samples = np.zeros(FFT_SIZE, dtype=DTYPE)
            for i in range(FFT_SIZE):
                raw_samples[i] = struct.unpack(">f", raw_bytes[i * NUM_BYTES:(i + 1) * NUM_BYTES])[0]

            process_chunk(raw_samples, sample_index)
            sample_index += 4


def process_generated() -> None:
    sin_freq = 1200
    samples = np.sin(2 * np.pi * np.arange(SAMPLE_RATE * CHUNK_TIME) * sin_freq / SAMPLE_RATE).astype(DTYPE)
    # samples = np.linspace(0, 100, int(SAMPLE_RATE * CHUNK_TIME), dtype=DTYPE)
    print("dtype: {}".format(samples.dtype))
    print(len(samples), samples)
    process_chunk(samples, 0)



if __name__ == "__main__":
    # process_file()
    process_generated()
