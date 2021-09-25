"""Render animation."""

import subprocess  # noqa: S404

from plotting import SAMPLE_RATE, FFT_LENGTH


def main():
    """Render animation."""
    fps = 1 / ((1 / SAMPLE_RATE) * FFT_LENGTH)
    print("Rendering at {0} fps".format(fps))

    subprocess.run([
        "ffmpeg",
        "-vsync",
        "0",
        "-hwaccel",
        "cuda",
        "-hwaccel_output_format",
        "cuda",
        "-framerate",
        str(fps),
        "-i",
        r"render\chunk_%04d.png",
        "-i",
        "song.wav",
        "-c:v",
        "h264_nvenc",
        "-b:v",
        "5M",
        "render.mp4",
    ])  # noqa: S603, S607


if __name__ == "__main__":
    main()