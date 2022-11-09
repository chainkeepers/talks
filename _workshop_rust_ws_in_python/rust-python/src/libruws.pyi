class _Stream:

    url: str
    queue: str

    def __init__(self, url: str) -> None:
        ...

    def run(self, duration: int) -> None:
        ...
