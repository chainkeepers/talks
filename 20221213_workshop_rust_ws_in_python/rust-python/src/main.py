import asyncio
import logging
from time import time
import sys

from ruws import Stream

async def loop(stream):
    while True:
        # print('looping')
        # print(stream.queue)
        await asyncio.sleep(0.01)


async def main_loop(duration):

    # url = "wss://ftx.com/ws/"
    url = "ws://localhost:49158/"

    stream = Stream(url)
    stream.run(duration)

    asyncio.get_event_loop().create_task(loop(stream))

    async for data in stream:
        ex_time = data["ex_time"]
        ru_time = data["ru_time"]
        loc_time = time()
        print(
            f"{loc_time:>10.9f}, "
            f"{ex_time:>10.9f}, "
            f"{1000. * (loc_time - ex_time):>4.6f}, "
            f"{1000. * (ru_time - ex_time):>4.6f}"
        )
        stream.queue.task_done()


if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    duration = int(sys.argv[1])
    asyncio.run(main_loop(duration))
