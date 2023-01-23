import pytest
from asyncio import sleep, gather, get_running_loop
import logging

from orjson import dumps as orjson_dumps

from ruws import Stream


async def test_stream():

    endpoint = 'wss://fstream.binance.com/ws/'

    stream = Stream(endpoint)

    msg = {
        "method": "SUBSCRIBE",
        "params": ["btcusdt@bookTicker"],
        "id": 0,
    }

    for _ in range(10):
        stream.send(orjson_dumps(msg).decode('utf-8'))

    async def interrupt(delay):
        await sleep(delay)
        stream.close()

    task = get_running_loop().create_task(interrupt(3)) 

    async for data in stream:
        pass

    gather(task)

    with pytest.raises(RuntimeError):
        async for data in stream:
            pass
