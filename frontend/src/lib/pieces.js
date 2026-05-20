// 장기 기물 정의. side: 'cho'(초/아래) | 'han'(한/위)
// kind: K=궁, A=사, R=차, C=포, H=마, E=상, P=졸/병
// 한자 표기는 일반적인 한장기 표기 — 초(楚)는 청색, 한(漢)은 적색.

export const KIND_NAMES = {
  K: { cho: '楚', han: '漢' },
  A: { cho: '士', han: '士' },
  R: { cho: '車', han: '車' },
  C: { cho: '包', han: '包' },
  H: { cho: '馬', han: '馬' },
  E: { cho: '象', han: '象' },
  P: { cho: '卒', han: '兵' }
};

// 표준 초기 배치 (마상마상, 위=한, 아래=초)
// row 0~9, col 0~8
export function initialBoard() {
  const empty = () => Array.from({ length: 10 }, () => Array(9).fill(null));
  const b = empty();
  // 한측 (위, row 0~3)
  b[0][0] = { side: 'han', kind: 'R' };
  b[0][1] = { side: 'han', kind: 'H' };
  b[0][2] = { side: 'han', kind: 'E' };
  b[0][3] = { side: 'han', kind: 'A' };
  b[0][5] = { side: 'han', kind: 'A' };
  b[0][6] = { side: 'han', kind: 'E' };
  b[0][7] = { side: 'han', kind: 'H' };
  b[0][8] = { side: 'han', kind: 'R' };
  b[1][4] = { side: 'han', kind: 'K' };
  b[2][1] = { side: 'han', kind: 'C' };
  b[2][7] = { side: 'han', kind: 'C' };
  for (let c = 0; c < 9; c += 2) b[3][c] = { side: 'han', kind: 'P' };
  // 초측 (아래, row 6~9)
  for (let c = 0; c < 9; c += 2) b[6][c] = { side: 'cho', kind: 'P' };
  b[7][1] = { side: 'cho', kind: 'C' };
  b[7][7] = { side: 'cho', kind: 'C' };
  b[8][4] = { side: 'cho', kind: 'K' };
  b[9][0] = { side: 'cho', kind: 'R' };
  b[9][1] = { side: 'cho', kind: 'H' };
  b[9][2] = { side: 'cho', kind: 'E' };
  b[9][3] = { side: 'cho', kind: 'A' };
  b[9][5] = { side: 'cho', kind: 'A' };
  b[9][6] = { side: 'cho', kind: 'E' };
  b[9][7] = { side: 'cho', kind: 'H' };
  b[9][8] = { side: 'cho', kind: 'R' };
  return b;
}
