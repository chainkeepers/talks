from asyncio import Queue


class PyStream:

    url: str
    queue: Queue

    def __init__(self, url: str) -> None:
        ...

    def run(self, duration: int) -> None:
        ...
