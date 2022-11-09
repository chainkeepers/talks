import asyncio
import logging
import time

import ruws


async def main_loop():

    url = "wss://ftx.com/ws/"

    stream = ruws.Stream(url)

    print(f"The stream URL is {stream.url}")

    stream.run()

    async for data in stream:
        py_time = time.time()
        bid = data["bid"]
        ask = data["ask"]
        ex_time = data["ex_time"]
        loc_time = time.time()
        if bid is not None and ask is not None:
            print(
                f"""{loc_time:>10.9f}, {ex_time:>10.9f}, {1000. * (loc_time - ex_time):>4.6f}: {data["bid_size"]:>10.6}  {bid:.9} {ask:.9}  {data["ask_size"]: <10.6}"""
            )



if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO)
    asyncio.run(main_loop())
