
export function openRelay(): WebSocket{
    const ws = new WebSocket("ws://localhost:3000/ws");
    ws.binaryType = "arraybuffer";
    return ws;
}
//create a websocket connection to the backend

