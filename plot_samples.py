import struct
import subprocess
from pathlib import Path

from matplotlib import pyplot as plt
import matplotlib.gridspec as gridspec
import numpy as np

FFT_EXEC = r"E:\cargo_target\release\processing_test.exe"

# Input constants
FFT_SIZE = 512
FFT_OUT_SIZE = FFT_SIZE // 2
SAMPLE_RATE = 44_100

# Calculated constants
BATCH_SIZE = FFT_SIZE * 4
SAMPLE_TIME = 1 / SAMPLE_RATE
SAMPLE_TIMES = np.arange(FFT_SIZE, dtype=np.float32) * SAMPLE_TIME
SAMPLE_BATCH_LEN = BATCH_SIZE * SAMPLE_TIME


def do_plot(sample_idex, raw_samples, fft_samples) -> None:
    fig = plt.figure(constrained_layout=True)
    gs = gridspec.GridSpec(2, 4, figure=fig)

    ax1 = fig.add_subplot(gs[0, 0])
    ax1.plot(SAMPLE_TIMES + (sample_idex * SAMPLE_BATCH_LEN), raw_samples[0:FFT_SIZE])

    ax2 = fig.add_subplot(gs[1, 0])
    ax2.plot(fft_samples[0:FFT_OUT_SIZE])
    print(fft_samples[0: 3])

    for col in range(1, 4):
        ax = fig.add_subplot(gs[0, col], sharey=ax1)
        ax.plot(SAMPLE_TIMES + ((sample_idex + col) * SAMPLE_BATCH_LEN), raw_samples[col * FFT_SIZE:(col + 1) * FFT_SIZE])

        ax = fig.add_subplot(gs[1, col], sharey=ax2)
        ax.plot(fft_samples[col * FFT_OUT_SIZE:(col + 1) * FFT_OUT_SIZE])
        print(fft_samples[col * FFT_OUT_SIZE:(col * FFT_OUT_SIZE) + 3])

    plt.show()


def main() -> None:
    with open("raw.bin", "rb") as rs:
        sample_index = 0
        while True:
            raw_bytes = rs.read(BATCH_SIZE * 4)
            print(len(raw_bytes))
            raw_samples = np.zeros(BATCH_SIZE, dtype=np.float32)
            for i in range(BATCH_SIZE):
                raw_samples[i] = struct.unpack(">f", raw_bytes[i * 4:(i + 1) * 4])[0]

            print("Processing {} bytes".format(len(raw_bytes)))
            Path("in.tmp").write_bytes(raw_bytes)
            print(subprocess.check_output([FFT_EXEC]))
            fft_bytes = Path("out.tmp").read_bytes()
            print(len(fft_bytes))

            fft_samples = np.zeros(BATCH_SIZE // 2, dtype=np.float32)
            for i in range(BATCH_SIZE // 2):
                index = i * 4
                fft_samples[i] = struct.unpack(">f", fft_bytes[index:index + 4])[0]

            do_plot(sample_index, raw_samples, fft_samples)
            sample_index += 4


if __name__ == "__main__":
    main()
