from libruws import PyStream


class Stream:
    def __init__(self, *args, **kwargs):
        self.stream = PyStream(*args, **kwargs)

    def __getattr__(self, key):
        return getattr(self.stream, key)

    async def __aiter__(self):
        while True:
            data = await self.stream.queue.get()
            if isinstance(data, Exception):
                raise data
            yield data
