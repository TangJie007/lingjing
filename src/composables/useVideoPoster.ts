const posterCache = new Map<string, string>();

function cacheKey(id: string, src: string) {
  return `${id}:${src}`;
}

export function getCachedVideoPoster(id: string, src: string): string | null {
  return posterCache.get(cacheKey(id, src)) ?? null;
}

export function cacheVideoPoster(id: string, src: string, dataUrl: string) {
  if (!dataUrl) return;
  posterCache.set(cacheKey(id, src), dataUrl);
}

export function forgetVideoPoster(id: string, src?: string) {
  if (src) {
    posterCache.delete(cacheKey(id, src));
    return;
  }
  for (const key of posterCache.keys()) {
    if (key.startsWith(`${id}:`)) posterCache.delete(key);
  }
}

export function captureVideoFrame(video: HTMLVideoElement): string | null {
  try {
    const w = video.videoWidth;
    const h = video.videoHeight;
    if (!w || !h) return null;
    const canvas = document.createElement("canvas");
    const scale = Math.min(1, 640 / w);
    canvas.width = Math.max(1, Math.round(w * scale));
    canvas.height = Math.max(1, Math.round(h * scale));
    const ctx = canvas.getContext("2d");
    if (!ctx) return null;
    ctx.drawImage(video, 0, 0, canvas.width, canvas.height);
    return canvas.toDataURL("image/jpeg", 0.82);
  } catch {
    return null;
  }
}

export function isBenignPlayError(err: unknown) {
  const name = err instanceof Error ? err.name : "";
  const msg = err instanceof Error ? err.message : String(err || "");
  const hay = `${name} ${msg}`.toLowerCase();
  return hay.includes("aborterror") || hay.includes("interrupted by a new load");
}
