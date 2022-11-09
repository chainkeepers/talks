import asyncio
import orjson
import time
import sys

from aiohttp import ClientSession
import websockets


def subscribe(market):
    return f"{{\"op\": \"subscribe\", \"channel\": \"ticker\", \"market\": \"{market}\"}}"


async def client_loop():
    url = "wss://ftx.com/ws/";

    async with ClientSession() as session:
        async with session.ws_connect(url) as ws:

            await ws.send_str(subscribe("BTC-PERP"))

            async for resp in ws:
                data = resp.json(loads=orjson.loads)

                if "data" not in data:
                    continue
                
                ex_time = data["data"]["time"]
                loc_time = time.time()
                
                print(
                    f"""{loc_time:>10.9f}, {ex_time:>10.9f}, {1000. * (loc_time - ex_time):>4.6f}: {data["data"]["bidSize"]:>10.6}  {data["data"]["bid"]:.9} {data["data"]["ask"]:.9}  {data["data"]["askSize"]: <10.6}"""
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
