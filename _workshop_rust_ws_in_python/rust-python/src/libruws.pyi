class _Stream:
    async def __aiter__(self):
        while True:
            yield await self.queue.get()
