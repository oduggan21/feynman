import { useState, useEffect } from "react";
import { openRelay } from "./services/ws";
import { useMic } from "./hooks/useMic";
import { playAudio } from "./services/audio";

export default function App() {
  const [ws] = useState(() => openRelay());
  const [running, setRunning] = useState(false);

  useMic(ws, running);

  useEffect(() => {
    ws.onmessage = e => {
      if (typeof e.data !== "string") playAudio(e.data);
    };
  }, [ws]);

  return (
    <main style={{ textAlign: "center", marginTop: 40 }}>
      <button onClick={() => setRunning(r => !r)}>
        {running ? "Stop" : "Start"}
      </button>
    </main>
  );
}
