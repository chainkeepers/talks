import asyncio
from aiohttp import web
import sys
from orjson import dumps
import weakref
from time import time
from random import shuffle


def create_handler(stream):

    async def handler(request):

        ws = web.WebSocketResponse()
        await ws.prepare(request)

        with stream.subscribe() as sub:
            async for msg in sub:
                await ws.send_str(msg)

    return handler


class Subscription:
    def __init__(self, parent):
        self.queue = asyncio.Queue()
        self.parent = weakref.ref(parent)

    def on_next(self, value):
        self.queue.put_nowait(value)

    async def on_send(self, value):
        await self.queue.put(value)
        
    async def __aiter__(self):
        while True:
            yield await self.queue.get()

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.close()

    def close(self):
        parent = self.parent()
        if parent is not None:
            parent.unsubscribe(self)


class Stream:
    def __init__(self):
        self.listeners = []

    def subscribe(self):
        queue = Subscription(self)
        self.listeners.append(queue)
        return queue

    def unsubscribe(self, sub):
        self.listeners.remove(sub)

    def on_next(self, value):
        shuffle(self.listeners)
        for listener in self.listeners:
            listener.on_next(value)

    async def on_send(self, value):
        shuffle(self.listeners)
        for listener in self.listeners:
            await listener.on_send(value)


async def stream_loop(stream, interval, pad): 
    while True:
        ts = time()
        stream.on_next(dumps({'data': {'time': ts, 'pad': pad * ['A']}}).decode('utf-8'))
        await asyncio.sleep(interval / 1000)
            

async def main_loop(duration, datalen):

    stream = Stream()
    stream_task = asyncio.get_running_loop().create_task(stream_loop(stream, 10, datalen))

    async def shutdown(app):
        for ws in app['websockets'].values():
            await ws.close(code=WSCloseCode.GOING_AWAY)
        app['websockets'].clear()

    app = web.Application()
    app['websockets'] = {}
    app.on_shutdown.append(shutdown)
    app.router.add_get('/', create_handler(stream))

    runner = web.AppRunner(app)
    await runner.setup()
    site = web.TCPSite(runner, 'localhost', 49158)
    await site.start()

    print("Server is up")

    await asyncio.sleep(duration)
    stream_task.cancel()

    print("Server stopped")



if __name__ == "__main__":
    duration = int(sys.argv[1])
    datalen = int(sys.argv[2])
    asyncio.run(main_loop(duration, datalen))
