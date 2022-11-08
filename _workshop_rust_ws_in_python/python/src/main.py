import asyncio
import orjson
from datetime import datetime

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
                loc_time = datetime.now().timestamp()
                
                print(
                    f"""{loc_time:>10.9f}, {ex_time:>10.9f}, {1000. * (loc_time - ex_time):>4.6f}: {data["data"]["bidSize"]:>10.6}  {data["data"]["bid"]} {data["data"]["ask"]}  {data["data"]["askSize"]: <10.6}"""
                )


async def term_loop(app):
    await asyncio.sleep(5)
    app.cancel()


async def main_loop():
    app = asyncio.create_task(client_loop())
    await term_loop(app)


if __name__ == "__main__":
    asyncio.run(main_loop())
