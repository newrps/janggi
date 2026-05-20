// 장기 말 두는 소리 — 나무 말이 보드에 "딱" 부딪히는 느낌.
// 낮은 triangle 톤 (180→80Hz) + lowpass 노이즈로 deep한 thud.

let ctx = null;

function getCtx() {
  if (typeof window === 'undefined') return null;
  if (!ctx) {
    const C = window.AudioContext || window.webkitAudioContext;
    if (!C) return null;
    ctx = new C();
  }
  if (ctx.state === 'suspended') ctx.resume();
  return ctx;
}

export function playPiece() {
  const c = getCtx();
  if (!c) return;
  const now = c.currentTime;

  // 메인 톤 — 나무 진동
  const osc = c.createOscillator();
  const g = c.createGain();
  osc.type = 'triangle';
  osc.frequency.setValueAtTime(180, now);
  osc.frequency.exponentialRampToValueAtTime(80, now + 0.07);
  g.gain.setValueAtTime(0.5, now);
  g.gain.exponentialRampToValueAtTime(0.001, now + 0.14);
  osc.connect(g).connect(c.destination);
  osc.start(now);
  osc.stop(now + 0.15);

  // 두꺼운 어택 노이즈 (lowpass)
  const buf = c.createBuffer(1, 1024, c.sampleRate);
  const data = buf.getChannelData(0);
  for (let i = 0; i < 1024; i++) {
    data[i] = (Math.random() * 2 - 1) * Math.exp(-i / 200);
  }
  const noise = c.createBufferSource();
  noise.buffer = buf;
  const filter = c.createBiquadFilter();
  filter.type = 'lowpass';
  filter.frequency.value = 600;
  const ng = c.createGain();
  ng.gain.value = 0.4;
  noise.connect(filter).connect(ng).connect(c.destination);
  noise.start(now);
}
