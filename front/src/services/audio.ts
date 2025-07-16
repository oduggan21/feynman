export function playAudio(data: ArrayBuffer) {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const ctx = new (window.AudioContext || (window as any).webkitAudioContext)();
  ctx.decodeAudioData(data.slice(0)).then(buf => {
    const src = ctx.createBufferSource();
    src.buffer = buf;
    src.connect(ctx.destination);
    src.start();
  });
}
