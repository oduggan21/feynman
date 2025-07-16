import { useEffect, useRef} from "react";

export function useMic(ws: WebSocket, running: boolean) {
  const ctxRef = useRef<AudioContext | null>(null);
  const srcRef = useRef<MediaStreamAudioSourceNode | null>(null);

  useEffect(() => {
    if (!running) return;

    let cancelled = false;

    (async () => {
      try {
        // Wait for user to grant mic access
        const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
        if (cancelled) return;

        // Only create AudioContext after permission is granted
        const ctx = new AudioContext({ sampleRate: 48000 });
        ctxRef.current = ctx;
        srcRef.current = ctx.createMediaStreamSource(stream);

        const processor = ctx.createScriptProcessor(4096, 1, 1);
        processor.onaudioprocess = e => {
          if (!running || ws.readyState !== WebSocket.OPEN) return;
          const pcm = e.inputBuffer.getChannelData(0);
          const buf = new ArrayBuffer(pcm.length * 2);
          const view = new DataView(buf);
          for (let i = 0; i < pcm.length; i++) {
            let s = Math.max(-1, Math.min(1, pcm[i]));
            view.setInt16(i * 2, s * 32767, true); // little-endian
          }
          ws.send(buf);
        };

        srcRef.current.connect(processor);
        processor.connect(ctx.destination);
      } catch (err) {
        console.error("Failed to initialize audio context or get mic:", err);
      }
       })();

    return () => {
      cancelled = true;
      ctxRef.current?.close();
    };
  }, [running, ws]);
}