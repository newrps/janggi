async function post(path, body) {
  const res = await fetch(path, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body)
  });
  if (!res.ok) throw new Error(`${path} ${res.status}`);
  return res.json();
}

export const aiMove = (board, aiSide, difficulty) =>
  post('/api/ai-move', { board, ai_side: aiSide, difficulty });
