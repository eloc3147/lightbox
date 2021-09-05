from __future__ import annotations

from pathlib import Path
from typing import Any, Optional
import argparse
import wave

FFT_EXEC = Path.cwd() / "build" / "release" / "processing_test.exe"

import subprocess
from pathlib import Path

from matplotlib import pyplot as plt
import matplotlib.gridspec as gridspec
import numpy as np
from tqdm import trange

# Input constants
NUM_BYTES = 4         # Number of bytes per sample (currently f32)
FFT_SIZE = 2048        # Number of samples in FFT
SAMPLE_RATE = 44_100  # Audio sample rate, in Hz

DTYPE = ">f4"

# Calculated constants
FFT_OUT_SIZE = FFT_SIZE // 2
SAMPLE_TIME = 1 / SAMPLE_RATE
CHUNK_TIME = SAMPLE_TIME * FFT_SIZE
FREQS = SAMPLE_RATE * np.arange((FFT_SIZE / 2), dtype=DTYPE) / FFT_SIZE


def draw_waveform(sample_idex: int, samples: np.ndarray, axis: Any, ylim: Optional[tuple[float, float]] = None) -> None:
    axis.plot(np.arange(sample_idex, sample_idex + len(samples), dtype=np.float32) * SAMPLE_TIME, samples, color="C1")
    axis.set_xticks(np.arange(sample_idex, sample_idex + len(samples), len(samples) / 10, dtype=np.float32) * SAMPLE_TIME)
    if ylim:
        axis.set_ylim(ylim)
    axis.set_title("Time domain signal")


def plot_series(
    sample_idex: int,
    raw_samples: np.ndarray,
    fft_samples: np.ndarray,
) -> None:
    fig = plt.figure(constrained_layout=True, figsize=(16, 9))
    gs = gridspec.GridSpec(3, 2, figure=fig, width_ratios=(30, 1))

    ax1 = fig.add_subplot(gs[0, 0])
    draw_waveform(sample_idex, raw_samples, ax1)

    ax2 = fig.add_subplot(gs[1, 0])
    impl_fig = ax2.imshow(np.rot90(fft_samples), cmap=plt.get_cmap("viridis"), aspect="auto")
    ax2.grid(False)
    ax2.set_ylabel("Frequency")
    ax2.set_title("Implementation FFT")

    ax3 = fig.add_subplot(gs[1, 1])
    plt.colorbar(impl_fig, cax=ax3)

    np_ampls = np.zeros((len(raw_samples) // FFT_SIZE, FFT_OUT_SIZE), dtype=np.float64)

    for idx in range(len(raw_samples) // FFT_SIZE):
        np_ampls[idx] = process_np(raw_samples[idx * FFT_SIZE: (idx + 1) * FFT_SIZE])[1]

    db_power = 10 * np.log10(np_ampls)

    ax4 = fig.add_subplot(gs[2, 0])
    np_fig = ax4.imshow(np.rot90(db_power), cmap=plt.get_cmap("viridis"), aspect="auto")
    ax4.grid(False)
    ax4.set_ylabel("Frequency")
    ax4.set_title("Numpy FFT")

    ax5 = fig.add_subplot(gs[2, 1])
    plt.colorbar(np_fig, cax=ax5)

    plt.show()


def plot_chunk(
    sample_idex: int,
    raw_samples: np.ndarray,
    fft_samples: np.ndarray,
    plot_path: Optional[Path] = None,
    wavform_ylim: Optional[tuple[float, float]] = None,
    fft_ylim: Optional[tuple[float, float]] = None,
) -> None:
    fig = plt.figure(constrained_layout=True, figsize=(16, 9))
    gs = gridspec.GridSpec(3, 1, figure=fig)

    ax1 = fig.add_subplot(gs[0, 0])
    draw_waveform(sample_idex, raw_samples, ax1, ylim=wavform_ylim)

    db_power = 10 * np.log10(fft_samples)
    ax2 = fig.add_subplot(gs[1, 0])
    ax2.plot(FREQS, db_power, color="C2")
    ax2.set_ylabel("Power (dB)")
    ax2.set_title("Implementation FFT")

    np_freqs, np_ampls = process_np(raw_samples)
    db_power = 10 * np.log10(np_ampls)

    ax2 = fig.add_subplot(gs[2, 0])
    ax2.plot(np_freqs, db_power, color="C3")
    ax2.set_ylabel("Power (dB)")
    ax2.set_title("Numpy FFT")
    if fft_ylim:
        ax2.set_ylim(fft_ylim)

    if plot_path:
        plt.savefig(str(plot_path), dpi=120)
        plt.close(fig)
    else:
        plt.show()


def process_samples(raw_samples: np.ndarray) -> np.ndarray:
    print("Processing {0} samples.".format(len(raw_samples)))
    raw_samples.tofile("in.tmp")

    result = subprocess.run([str(FFT_EXEC)], capture_output=True)
    if result.returncode != 0:
        print("Sample processing crashed")
        print("Process STDOUT: {0}".format(result.stdout.decode()))
        print("Process STDERR: {0}".format(result.stderr.decode()))
        raise Exception()

    return np.fromfile("out.tmp", dtype=DTYPE)


def process_np(raw_samples: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    fourier = np.fft.fft(raw_samples)
    freqs = np.fft.fftfreq(len(raw_samples)) * len(raw_samples) * SAMPLE_RATE
    return (np.fft.fftshift(freqs)[FFT_OUT_SIZE:], np.abs(np.fft.fftshift(fourier))[FFT_OUT_SIZE:])


def process_chunk(
    raw_samples: np.ndarray,
    sample_index: int,
    plot_path: Optional[Path] = None,
    wavform_ylim: Optional[tuple[float, float]] = None,
    fft_ylim: Optional[tuple[float, float]] = None,
) -> None:
    fft_samples = process_samples(raw_samples)

    plot_chunk(sample_index, raw_samples, fft_samples, plot_path=plot_path, wavform_ylim=wavform_ylim, fft_ylim=fft_ylim)


def main() -> None:
    parser = argparse.ArgumentParser(description="Porcess some audio")
    parser.add_argument("input", type=str, help="An audio file to process")

    args = parser.parse_args()

    audio_file = Path(args.input).resolve()
    
    wav = wave.open(str(audio_file), mode="rb")

    (nchannels, sampwidth, framerate, nframes, comptype, compname) = wav.getparams()

    print(
        "Params {{\n\tnchannels: {},\n\tsampwidth: {},\n\tframerate: {},\n\tnframes: {},\n\tcomptype: {},\n\tcompname: {},\n}}".format(
            nchannels,
            sampwidth,
            framerate,
            nframes,
            comptype,
            compname,
        ),
    )

    frames = wav.readframes(nframes)
    print("Read {0} bytes. {1} bytes per frame".format(len(frames), len(frames) // nframes))

    dt = np.dtype(np.int16)
    dt = dt.newbyteorder("L")
    
    # Left channel only
    samples = np.frombuffer(frames, dtype=dt).reshape((nframes, 2))[:, 0].astype(DTYPE) / ((2 ** 16) / 2)

    print("Number of chunks: {0}".format(len(samples) / FFT_SIZE))

    render_dir = Path.cwd() / "render"
    render_dir.mkdir(parents=True, exist_ok=True)

    wav_min = np.min(samples) * 1.1
    wav_max = np.max(samples) * 1.1

    for chunk in trange(len(samples) // FFT_SIZE):
        sample_index = chunk * FFT_SIZE

        process_chunk(
            samples[sample_index:sample_index + FFT_SIZE],
            sample_index,
            plot_path=render_dir / "chunk_{0:04d}.png".format(chunk),
            wavform_ylim=(wav_min, wav_max),
            fft_ylim=(-80, 30),
        )



if __name__ == "__main__":
    main()
