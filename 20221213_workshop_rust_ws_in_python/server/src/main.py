import asyncio
import websockets
import sys
import orjson
import weakref
import time


def create_handler(stream):

    async def handler(ws):
        with stream.subscribe() as sub:
            async for msg in sub:
                await ws.send(orjson.dumps(msg))

    return handler


class Subscription:
    def __init__(self, parent):
        self.queue = asyncio.Queue()
        self.parent = weakref.ref(parent)

    async def send(self, value):
        await self.queue.put(value)
        
    async def __aiter__(self):
        while True:
            yield await self.queue.get()

    def __enter__(self):
        return self

    def __exit__(self, *_):
        parent = self.parent()
        if parent is not None:
            parent.listeners.remove(self)


class Stream:

    def __init__(self):
        self.listeners = []

    def subscribe(self):
        queue = Subscription(self)
        self.listeners.append(queue)
        return queue

    async def send(self, value):
        for listener in self.listeners:
            await listener.send(value)


async def stream_loop(stream, interval): 
    while True:
        ts = time.time()
        await stream.send({'time': ts})
        await asyncio.sleep(interval / 1000)
            

async def main_loop(duration):

    loop = asyncio.get_running_loop()
    stream = Stream()
    stream_task = loop.create_task(stream_loop(stream, 10))

    async with websockets.serve(create_handler(stream), "localhost", 49158):
        print("Server is up")
        await asyncio.sleep(duration)
        stream_task.cancel()

    print("Server stopped")


if __name__ == "__main__":
    duration = int(sys.argv[1])
    asyncio.run(main_loop(duration))
