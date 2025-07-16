import { useEffect, useRef } from "react";

export function useMic(ws: WebSocket, running: boolean) {
  const ctxRef = useRef<AudioContext | null>(null);
  const srcRef = useRef<MediaStreamAudioSourceNode | null>(null);

  useEffect(() => {
    if (!running) return;

    
    (async () => {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      const ctx = new AudioContext({ sampleRate: 48_000 });
      ctxRef.current = ctx;
      srcRef.current = ctx.createMediaStreamSource(stream);

      const processor = ctx.createScriptProcessor(4096, 1, 1);
      processor.onaudioprocess = e => {
        if (!running || ws.readyState !== WebSocket.OPEN) return;
        const pcm = e.inputBuffer.getChannelData(0);
        ws.send(new Float32Array(pcm).buffer);   // 48 kHz mono PCM
      };

      srcRef.current.connect(processor);
      processor.connect(ctx.destination);
    })();

    return () => { ctxRef.current?.close(); };
  }, [running, ws]);
}
