import { useEffect, useRef, useState } from "react";

export function useMic(ws: WebSocket, running: boolean) {
  const ctxRef = useRef<AudioContext | null>(null);
  const srcRef = useRef<MediaStreamAudioSourceNode | null>(null);
  const [isSpeaking, setIsSpeaking] = useState(false);
  const silenceTimeoutRef = useRef<number | null>(null);

  useEffect(() => {
    if (!running) {
      // Clean up when stopping
      if (ctxRef.current) {
        ctxRef.current.close();
        ctxRef.current = null;
      }
      if (srcRef.current) {
        srcRef.current = null;
      }
      if (silenceTimeoutRef.current) {
        clearTimeout(silenceTimeoutRef.current);
        silenceTimeoutRef.current = null;
      }
      return;
    }

    (async () => {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      const ctx = new AudioContext({ sampleRate: 48_000 });
      ctxRef.current = ctx;
      srcRef.current = ctx.createMediaStreamSource(stream);

      const processor = ctx.createScriptProcessor(4096, 1, 1);
      processor.onaudioprocess = e => {
        if (!running || ws.readyState !== WebSocket.OPEN) return;
        
        const pcm = e.inputBuffer.getChannelData(0);
        
        // Calculate RMS to detect speech
        let rms = 0;
        for (let i = 0; i < pcm.length; i++) {
          rms += pcm[i] * pcm[i];
        }
        rms = Math.sqrt(rms / pcm.length);
        
        const speechThreshold = 0.01; // Adjust this threshold as needed
        const hasSpeech = rms > speechThreshold;
        
        if (hasSpeech) {
          if (!isSpeaking) {
            setIsSpeaking(true);
            console.log("Speech detected, starting audio capture");
          }
          
          // Clear any existing silence timeout
          if (silenceTimeoutRef.current) {
            clearTimeout(silenceTimeoutRef.current);
            silenceTimeoutRef.current = null;
          }
          
          // Send audio data
          const buf = new ArrayBuffer(pcm.length * 2);
          const view = new DataView(buf);
          for (let i = 0; i < pcm.length; i++) {
            let s = Math.max(-1, Math.min(1, pcm[i]));
            view.setInt16(i * 2, s * 32767, true); // true = little-endian
          }
          console.log("Sending audio frame of length", buf.byteLength);
          ws.send(buf);
        } else if (isSpeaking) {
          // Start silence timeout if not already started
          if (!silenceTimeoutRef.current) {
            silenceTimeoutRef.current = setTimeout(() => {
              console.log("Silence detected, committing audio buffer");
              setIsSpeaking(false);
              ws.send("commit_audio");
              silenceTimeoutRef.current = null;
            }, 1000); // 1 second of silence
          }
        }
      };

      srcRef.current.connect(processor);
      processor.connect(ctx.destination);
    })();

    return () => { 
      if (ctxRef.current) {
        ctxRef.current.close(); 
      }
      if (silenceTimeoutRef.current) {
        clearTimeout(silenceTimeoutRef.current);
      }
    };
  }, [running, ws]);
}
