// useAudio.ts
// 灵镜 · 侧栏点击音效（Web Audio 合成，不引入音频素材）

import { ref } from "vue";

type AudioCtxCtor = typeof AudioContext;

let audioCtx: AudioContext | null = null;

function getCtx(): AudioContext | null {
  if (typeof window === "undefined") return null;
  const W = window as unknown as { AudioContext?: AudioCtxCtor; webkitAudioContext?: AudioCtxCtor };
  const Ctor = W.AudioContext ?? W.webkitAudioContext;
  if (!Ctor) return null;
  if (!audioCtx) audioCtx = new Ctor();
  return audioCtx;
}

// 导航项的固定频率（Hz）
export const NAV_FREQ: Record<string, number> = {
  在线: 523.25,
  本地: 587.33,
  我的: 659.25,
  桌宠: 698.46,
  设置: 783.99,
  关于: 880.0,
};

// 全局音效开关（被设置页 toggle 控制）
export const soundOn = ref(true);

export function playClick(freq: number) {
  if (!soundOn.value) return;
  const ctx = getCtx();
  if (!ctx) return;
  try {
    if (ctx.state === "suspended") ctx.resume();
    const t = ctx.currentTime;
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.type = "triangle";
    osc.frequency.setValueAtTime(freq, t);
    osc.frequency.exponentialRampToValueAtTime(freq * 0.82, t + 0.08);
    gain.gain.setValueAtTime(0.0001, t);
    gain.gain.exponentialRampToValueAtTime(0.06, t + 0.008);
    gain.gain.exponentialRampToValueAtTime(0.0001, t + 0.12);
    osc.connect(gain).connect(ctx.destination);
    osc.start(t);
    osc.stop(t + 0.13);
  } catch {
    /* noop */
  }
}

export function useAudio() {
  return { playClick, soundOn, NAV_FREQ };
}
