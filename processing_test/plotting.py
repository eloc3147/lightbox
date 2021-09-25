from __future__ import annotations

from pathlib import Path
from typing import Any, Literal, Optional, Union
import argparse
import wave
from multiprocessing.pool import ThreadPool
from queue import Queue
from threading import Thread

from pathlib import Path

from matplotlib import pyplot as plt
import matplotlib.gridspec as gridspec
import numpy as np
from tqdm import trange

from processing_test import ProcessorInterface, FFT_LENGTH, FFT_OUT_LENGTH

# Input constants
SAMPLE_RATE = 44_100  # Audio sample rate, in Hz

DPI = 150
ASPECT_RATIO = (16, 9)

# Calculated constants
SAMPLE_TIME = 1 / SAMPLE_RATE
CHUNK_TIME = SAMPLE_TIME * FFT_LENGTH
FREQS = SAMPLE_RATE * np.arange(FFT_OUT_LENGTH, dtype=np.float32) / FFT_LENGTH

PLOT_WDITH = DPI * ASPECT_RATIO[0]
PLOT_HEIGHT = DPI * ASPECT_RATIO[1]


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

    np_ampls = np.zeros((len(raw_samples) // FFT_LENGTH, FFT_OUT_LENGTH), dtype=np.float64)

    for idx in range(len(raw_samples) // FFT_LENGTH):
        np_ampls[idx] = process_np(raw_samples[idx * FFT_LENGTH: (idx + 1) * FFT_LENGTH])[1]

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
    include_np: bool = True,
) -> None:
    fig = plt.figure(constrained_layout=True, figsize=(16, 9))
    
    if include_np:
        num_rows = 3
    else:
        num_rows = 2

    gs = gridspec.GridSpec(num_rows, 1, figure=fig)

    ax1 = fig.add_subplot(gs[0, 0])
    draw_waveform(sample_idex, raw_samples, ax1, ylim=wavform_ylim)

    ax2 = fig.add_subplot(gs[1, 0])
    ax2.plot(FREQS, fft_samples, color="C2")
    ax2.set_ylabel("Power (dB)")
    ax2.set_title("Implementation FFT")

    if include_np:
        np_freqs, np_ampls = process_np(raw_samples)
        db_power = 10 * np.log10(np_ampls)

        ax3 = fig.add_subplot(gs[2, 0])
        ax3.plot(np_freqs, db_power, color="C3")
        ax3.set_ylabel("Power (dB)")
        ax3.set_title("Numpy FFT")
        if fft_ylim:
            ax2.set_ylim(fft_ylim)

        if plot_path:
            plt.savefig(str(plot_path), dpi=120)
            plt.close(fig)
        else:
            plt.show()


def load_wav(sample_file: Union[Path, str], channel: Union[Literal["left"], Literal["right"]]) -> np.ndarray:
    """Load audio samples from a wav file and return one audio channel.

    Args:
        sample_file (Union[Path, str]): path to the wav file
        channel (Union[Literal["left"], Literal["right"]]): the channel to use

    Returns:
        np.ndarray: the samples
    """
    if isinstance(sample_file, Path):
        sample_file = str(sample_file)
    
    wav = wave.open(sample_file, mode="rb")

    (nchannels, sampwidth, framerate, nframes, comptype, compname) = wav.getparams()
    print(
        "Wav params: {{\n\tnchannels: {},\n\tsampwidth: {},\n\tframerate: {},\n\tnframes: {},\n\tcomptype: {},\n\tcompname: {},\n}}".format(
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
    if channel == "left":
        channel_idx = 0
    else:
        channel_idx = 1

    return np.frombuffer(frames, dtype=dt).reshape((nframes, 2))[:, channel_idx].astype(np.float32) / ((2 ** 16) / 2)


def process_np(raw_samples: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    fourier = np.fft.fft(raw_samples)
    freqs = np.fft.fftfreq(len(raw_samples)) * len(raw_samples) * SAMPLE_RATE
    return (np.fft.fftshift(freqs)[FFT_OUT_LENGTH:], np.abs(np.fft.fftshift(fourier))[FFT_OUT_LENGTH:] + 10e-10)


def render_chunk(interface: ProcessorInterface, render_dir: Path, samples: np.ndarray, chunk_index: int, status_queue: Queue):
    sample_index = chunk_index * FFT_LENGTH
    processed_chunk = interface.process_chunks(samples)

    np_freqs, np_samples = process_np(samples)
    np_power = 10 * np.log10(np_samples)

    interface.render_chunk(
        samples,
        SAMPLE_TIME,
        processed_chunk,
        FREQS,
        np_power,
        np_freqs,
        str(render_dir / "chunk_{0:04d}.png".format(chunk_index)),
        PLOT_WDITH,
        PLOT_HEIGHT,
    )

    status_queue.put(True)


def render_thread(interface: ProcessorInterface, render_dir: Path, samples: np.ndarray, status_queue: Queue):
    pool = ThreadPool(16)

    pool.map(
        lambda idx: render_chunk(interface, render_dir, samples[(idx * FFT_LENGTH):(idx * FFT_LENGTH) + FFT_LENGTH], idx, status_queue),
        range(len(samples) // FFT_LENGTH),
        chunksize=128,
    )


def main() -> None:
    parser = argparse.ArgumentParser(description="Porcess some audio")
    parser.add_argument("input", type=str, help="An audio file to process")

    args = parser.parse_args()

    audio_file = Path(args.input).resolve()
    samples = load_wav(audio_file, "left")
    print("Number of chunks: {0}".format(len(samples) / FFT_LENGTH))

    render_dir = Path.cwd() / "render"
    render_dir.mkdir(parents=True, exist_ok=True)

    interface = ProcessorInterface(window=False)

    status_queue = Queue()
    thread = Thread(target=render_thread, args=(interface, render_dir, samples, status_queue), daemon=True)
    thread.start()
    
    for _ in trange(len(samples) // FFT_LENGTH):
        status_queue.get()


if __name__ == "__main__":
    main()
