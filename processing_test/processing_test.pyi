from typing import Optional

FFT_LENGTH: int
FFT_OUT_LENGTH: int

class ProcessorInterface(object):
    def __init__(self, window: bool) -> None: ...

    def process_chunks(self, samples: list[float]) -> list[float]: ...
    def window(self) -> Optional[list[float]]: ...
    def render_chunk(self,
        input_samples: list[float],
        sample_duration: float,
        impl_samples: list[float],
        impl_freqs: list[float],
        np_samples: list[float],
        np_freqs: list[float],
        output_path: str,
        width: int,
        height: int,
    ) -> None: ...
