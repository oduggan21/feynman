import { useState, useEffect } from "react";
import { openRelay } from "./services/ws";
import { useMic } from "./hooks/useMic";
import { playAudio } from "./services/audio";

export default function App() {
  const [ws, setWs] = useState<WebSocket | null>(null);
  const [running, setRunning] = useState(false);
  const [connected, setConnected] = useState(false);
  const [error, setError] = useState<string>("");

  useMic(ws, running);

  useEffect(() => {
    const socket = openRelay();
    
    socket.onopen = () => {
      setConnected(true);
      setError("");
    };
    
    socket.onclose = () => {
      setConnected(false);
      setRunning(false);
    };
    
    socket.onerror = () => {
      setError("Failed to connect to backend");
      setConnected(false);
      setRunning(false);
    };

    socket.onmessage = e => {
      if (typeof e.data === "string") {
        console.log("Received text message:", e.data);
        // Handle error messages
        if (e.data.includes("Failed to connect to OpenAI")) {
          setError(e.data);
          setRunning(false);
        }
      } else {
        playAudio(e.data);
      }
    };
    
    setWs(socket);
    
    return () => {
      socket.close();
    };
  }, []);

  const handleToggle = () => {
    if (!connected) {
      setError("Not connected to backend");
      return;
    }
    setRunning(r => !r);
  };

  return (
    <main style={{ textAlign: "center", marginTop: 40 }}>
      <div style={{ marginBottom: 20 }}>
        Status: {connected ? "🟢 Connected" : "🔴 Disconnected"}
      </div>
      
      {error && (
        <div style={{ color: "red", marginBottom: 20 }}>
          Error: {error}
        </div>
      )}
      
      <button 
        onClick={handleToggle}
        disabled={!connected}
        style={{
          backgroundColor: connected ? (running ? "red" : "green") : "gray",
          color: "white",
          padding: "10px 20px",
          border: "none",
          borderRadius: "5px",
          cursor: connected ? "pointer" : "not-allowed"
        }}
      >
        {running ? "Stop" : "Start"}
      </button>
      
      {running && (
        <div style={{ marginTop: 20, color: "blue" }}>
          🎤 Recording... Speak to Feynman!
        </div>
      )}
    </main>
  );
}
