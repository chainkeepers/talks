from asyncio import Queue


class StreamEnded(StopAsyncIteration):
    pass


class StreamError(Exception):
    pass


class PyStream:

    queue: Queue

    def __init__(self) -> None:
        ...

    def run(self, url: str, duration: int) -> None:
        ...

    def close(self) -> None:
        ...
