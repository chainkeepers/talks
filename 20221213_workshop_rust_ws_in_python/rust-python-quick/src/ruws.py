from contextlib import contextmanager
from libruws import PyStream


class Stream:
    def __init__(self, *args, **kwargs):
        self.stream = PyStream(*args, **kwargs)

    def __getattr__(self, key):
        return getattr(self.stream, key)

    async def __aiter__(self):
        get = self.stream.queue.get

        while True:
            data = await get()
            if isinstance(data, StopAsyncIteration):
                return
            if isinstance(data, Exception):
                raise data
            yield data


@contextmanager
def connect(url: str) -> Stream:
    stream = Stream(url)
    try:
        yield stream
    finally:
        stream.close()
