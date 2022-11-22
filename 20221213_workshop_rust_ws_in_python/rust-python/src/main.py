import asyncio
import logging
import time
import sys

from ruws import Stream


async def main_loop(duration):

    url = "wss://ftx.com/ws/"
    stream = Stream(url)
    stream.run(duration)

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
    duration = int(sys.argv[1])
    asyncio.run(main_loop(duration))
