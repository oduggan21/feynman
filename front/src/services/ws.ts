
export function openRelay(): WebSocket{
    console.log("Connecting to backend WebSocket...");
    const ws = new WebSocket("ws://localhost:3000/ws");
    ws.binaryType = "arraybuffer";
    
    ws.onopen = () => {
        console.log("Connected to backend WebSocket");
    };
    
    ws.onerror = (error) => {
        console.error("WebSocket error:", error);
    };
    
    ws.onclose = (event) => {
        console.log("WebSocket closed:", event.code, event.reason);
    };
    
    return ws;
}
//create a websocket connection to the backend

