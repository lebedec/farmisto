import asyncio
import socket
import struct

from websockets.legacy.server import serve


def serve_websocket_proxy(ws_port: int, tcp_in: int):

    context = {
        'client': None,
        'websocket': None
    }

    async def handle_tcp(client):
        print('handle', client)
        loop = asyncio.get_event_loop()
        context['client'] = client
        while True:
            header = await loop.sock_recv(client, 4)
            if not header:
                break
            length = struct.unpack('i', header)
            data = await loop.sock_recv(client, length)
            message = data.decode('utf-8')
            websocket = context.get('websocket')
            if websocket is not None:
                websocket.send(message)
            else:
                print('skip', message)
        client.close()

    async def serve_tcp():
        loop = asyncio.get_event_loop()
        server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        print('serve TCP:', tcp_in)
        server.bind(('localhost', tcp_in))
        server.listen(8)
        server.setblocking(False)
        while True:
            client, _ = await loop.sock_accept(server)
            loop.create_task(handle_tcp(client))

    async def handle_ws(websocket):
        print('handle', websocket)
        loop = asyncio.get_event_loop()
        context['websocket'] = websocket
        async for message in websocket:
            client = context.get('client')
            if client is not None:
                data = message.encode('utf-8')
                await loop.sock_sendall(client, struct.pack('i', len(data)))
                await loop.sock_sendall(client, data)
            else:
                print('skipt', message)

    async def serve_ws():
        print('serve WS:', ws_port)
        async with serve(handle_ws, "localhost", ws_port):
            await asyncio.Future()

    asyncio.run(asyncio.wait([serve_ws(), serve_tcp()]))


if __name__ == '__main__':
    serve_websocket_proxy(9098, 9099)