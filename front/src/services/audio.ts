export function playAudio(data: ArrayBuffer) {
  console.log(`Received audio data: ${data.byteLength} bytes`);
  
  // Resume audio context if it's in suspended state (required for modern browsers)
  const ctx = new (window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext)();
  if (ctx.state === 'suspended') {
    ctx.resume();
  }
  
  // Check if this is PCM16 data (raw audio) or encoded audio
  // PCM16 data will be much smaller than encoded audio for the same duration
  if (isPCM16Data(data)) {
    console.log("Playing as PCM16 audio");
    playPCM16Audio(ctx, data);
  } else {
    console.log("Attempting to decode as encoded audio");
    // Fallback to decoding as encoded audio
    ctx.decodeAudioData(data.slice(0)).then(buf => {
      const src = ctx.createBufferSource();
      src.buffer = buf;
      src.connect(ctx.destination);
      src.start();
      console.log("Successfully played encoded audio");
    }).catch(err => {
      console.error("Failed to decode and play audio:", err);
      // As last resort, try as PCM16 anyway
      console.log("Trying as PCM16 as fallback");
      try {
        playPCM16Audio(ctx, data);
      } catch (pcmError) {
        console.error("Failed to play as PCM16 fallback:", pcmError);
      }
    });
  }
}

function isPCM16Data(data: ArrayBuffer): boolean {
  // PCM16 data characteristics:
  // - Size should be even (2 bytes per sample)
  // - Typically much smaller than encoded audio
  // - Usually contains audio samples in expected range
  
  if (data.byteLength % 2 !== 0) {
    return false; // PCM16 should have even byte count
  }
  
  // If it's small enough to be a short audio clip in PCM16 format, assume it's PCM16
  // This is a heuristic - PCM16 audio is typically sent in small chunks
  if (data.byteLength <= 4096) { // Up to ~85ms at 24kHz
    return true;
  }
  
  // For larger data, check if it looks like PCM16 by examining sample values
  const samples = new Int16Array(data, 0, Math.min(100, data.byteLength / 2));
  let validSamples = 0;
  for (let i = 0; i < samples.length; i++) {
    // Check if sample is in reasonable range (not too extreme)
    if (Math.abs(samples[i]) <= 32767) {
      validSamples++;
    }
  }
  
  return validSamples > samples.length * 0.8; // 80% of samples should be valid
}

function playPCM16Audio(ctx: AudioContext, data: ArrayBuffer) {
  // Convert PCM16 data to AudioBuffer
  const pcmData = new Int16Array(data);
  const sampleRate = 24000; // OpenAI uses 24kHz for PCM16
  const numberOfChannels = 1; // Mono
  const length = pcmData.length;
  
  if (length === 0) {
    console.warn("Empty PCM16 audio data received");
    return;
  }
  
  // Create an audio buffer
  const audioBuffer = ctx.createBuffer(numberOfChannels, length, sampleRate);
  const channelData = audioBuffer.getChannelData(0);
  
  // Convert PCM16 to float32 and copy to audio buffer
  for (let i = 0; i < length; i++) {
    channelData[i] = pcmData[i] / 32768.0; // Convert from 16-bit int to float32 (-1 to 1)
  }
  
  // Play the audio
  const source = ctx.createBufferSource();
  source.buffer = audioBuffer;
  source.connect(ctx.destination);
  source.start();
  
  const durationMs = (length / sampleRate) * 1000;
  console.log(`Playing PCM16 audio: ${length} samples at ${sampleRate}Hz (${durationMs.toFixed(1)}ms)`);
}
