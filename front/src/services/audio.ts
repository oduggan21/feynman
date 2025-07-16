// Global audio context instance to avoid recreating
let audioContext: AudioContext | null = null;

export async function initializeAudioContext(): Promise<AudioContext> {
  if (!audioContext) {
    audioContext = new (window.AudioContext || (window as any).webkitAudioContext)();
  }
  
  // Resume the audio context if it's suspended (browser autoplay policy)
  if (audioContext.state === 'suspended') {
    console.log('AudioContext suspended, resuming...');
    await audioContext.resume();
    console.log(`AudioContext resumed, state: ${audioContext.state}`);
  }
  
  return audioContext;
}

export async function playAudio(data: ArrayBuffer) {
  try {
    console.log(`Attempting to play audio: ${data.byteLength} bytes`);
    
    // Initialize and resume audio context if needed
    const ctx = await initializeAudioContext();
    
    // The data is raw PCM16 from OpenAI, not encoded audio
    // We need to create an AudioBuffer from the raw PCM data
    const pcmData = new Int16Array(data);
    const sampleRate = 16000; // OpenAI uses 16kHz sample rate for PCM16
    const channels = 1; // Mono audio
    
    console.log(`Audio details: ${pcmData.length} samples, ${sampleRate}Hz, ${channels} channel(s)`);
    
    if (pcmData.length === 0) {
      console.warn('Received empty audio data');
      return;
    }
    
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
    
    // Add event listeners for debugging
    source.onended = () => {
      console.log('Audio playback ended');
    };
    
    console.log(`AudioContext state before playing: ${ctx.state}`);
    source.start();
    console.log(`Audio playback started: ${pcmData.length} samples at ${sampleRate}Hz, duration: ${audioBuffer.duration.toFixed(2)}s`);
    
  } catch (error) {
    console.error("Error playing audio:", error);
    console.error("AudioContext state:", audioContext?.state);
  }
}
