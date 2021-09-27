from typing import Optional

FFT_LENGTH: int
FFT_OUT_LENGTH: int

class ProcessorInterface(object):
    def __init__(self, window: bool, led_count: int) -> None: ...

    def convert_samples(self, samples: list[float]) -> list[float]: ...
    def render_led_view(self, samples: list[float]) -> list[int]: ...
    def window(self) -> Optional[list[float]]: ...
    def render_chunk(self,
        input_samples: list[float],
        sample_duration: float,
        impl_samples: list[float],
        impl_freqs: list[float],
        led_view: list[int],
        output_path: str,
        width: int,
        height: int,
    ) -> None: ...
