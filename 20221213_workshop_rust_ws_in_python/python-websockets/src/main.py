import asyncio
from orjson import loads as orjson_loads
from time import time
import sys
import websockets


async def client_loop():

    url = "ws://localhost:49158"

    async with websockets.connect(url, compression=None) as ws:
        async for resp in ws:

            data = orjson_loads(resp)

            if "data" not in data:
                continue

            ex_time = data["data"]["time"]
            loc_time = time()

            print(
                f"{loc_time:>10.9f}, "
                f"{ex_time:>10.9f}, "
                f"{1000. * (loc_time - ex_time):>4.6f}"
            )


async def term_loop(app, duration):
    await asyncio.sleep(duration)
    app.cancel()


async def main_loop(duration):
    app = asyncio.create_task(client_loop())
    await term_loop(app, duration)


if __name__ == "__main__":
    duration = int(sys.argv[1])
    asyncio.run(main_loop(duration))
