import { useState, useEffect } from "react";
import { openRelay } from "./services/ws";
import { useMic } from "./hooks/useMic";
import { playAudio } from "./services/audio";

export default function App() {
  const [ws] = useState(() => openRelay());
  const [running, setRunning] = useState(false);
  const [connectionStatus, setConnectionStatus] = useState("Connecting...");
  const [lastMessage, setLastMessage] = useState("");

  useMic(ws, running);

  useEffect(() => {
    ws.onopen = () => {
      setConnectionStatus("Connected to server");
    };

    ws.onclose = () => {
      setConnectionStatus("Disconnected");
    };

    ws.onerror = () => {
      setConnectionStatus("Connection error");
    };

    ws.onmessage = e => {
      if (typeof e.data === "string") {
        setLastMessage(e.data);
        console.log("Received text message:", e.data);
        
        // Update connection status based on message content
        if (e.data.includes("Connected to OpenAI")) {
          setConnectionStatus("Connected to OpenAI - Ready to start");
        } else if (e.data.includes("OpenAI connection failed")) {
          setConnectionStatus("OpenAI connection failed");
        } else if (e.data.includes("TEST_MODE")) {
          setConnectionStatus("Test Mode - Ready to start");
        }
      } else {
        // Handle binary audio data
        playAudio(e.data);
        console.log("Received audio data:", e.data.byteLength, "bytes");
      }
    };
  }, [ws]);

  const handleStartStop = () => {
    if (!running && connectionStatus.includes("Ready to start")) {
      setRunning(true);
    } else if (running) {
      // Send final commit when stopping
      ws.send("commit_audio");
      setRunning(false);
    }
  };

  return (
    <main style={{ textAlign: "center", marginTop: 40, padding: 20 }}>
      <h1>Feynman Tutor</h1>
      <div style={{ marginBottom: 20 }}>
        <p>Status: {connectionStatus}</p>
        {lastMessage && (
          <p style={{ fontSize: 12, color: "#666", marginTop: 10 }}>
            Last message: {lastMessage}
          </p>
        )}
      </div>
      <button 
        onClick={handleStartStop}
        disabled={!connectionStatus.includes("Ready to start") && !running}
        style={{
          padding: "10px 20px",
          fontSize: 16,
          backgroundColor: running ? "#ff4444" : "#44aa44",
          color: "white",
          border: "none",
          borderRadius: 5,
          cursor: connectionStatus.includes("Ready to start") || running ? "pointer" : "not-allowed"
        }}
      >
        {running ? "Stop Teaching" : "Start Teaching"}
      </button>
      {running && (
        <p style={{ marginTop: 20, color: "#666" }}>
          Speak now... AI is listening to your teaching.
        </p>
      )}
    </main>
  );
}
