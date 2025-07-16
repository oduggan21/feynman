export function playAudio(data: ArrayBuffer) {
  try {
    const ctx = new (window.AudioContext || (window as any).webkitAudioContext)();
    
    // The data is raw PCM16 from OpenAI, not encoded audio
    // We need to create an AudioBuffer from the raw PCM data
    const pcmData = new Int16Array(data);
    const sampleRate = 16000; // OpenAI uses 16kHz sample rate for PCM16
    const channels = 1; // Mono audio
    
    // Create an AudioBuffer
    const audioBuffer = ctx.createBuffer(channels, pcmData.length, sampleRate);
    const channelData = audioBuffer.getChannelData(0);
    
    // Convert PCM16 to float32 for Web Audio API
    for (let i = 0; i < pcmData.length; i++) {
      channelData[i] = pcmData[i] / 32768.0; // Convert from int16 to float32 range [-1, 1]
    }
    
    // Create source and play the audio
    const source = ctx.createBufferSource();
    source.buffer = audioBuffer;
    source.connect(ctx.destination);
    source.start();
    
    console.log(`Playing audio: ${pcmData.length} samples at ${sampleRate}Hz`);
  } catch (error) {
    console.error("Error playing audio:", error);
  }
}
