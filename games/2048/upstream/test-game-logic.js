// test-game-logic.js
// Node-runnable verification of pure 2048 logic.
// Run: node test-game-logic.js

const assert = require('assert');
const GameLogic = require('./game.js');

function makeBoard(rows) {
  return rows.map((r) => [...r]);
}

function deepClone(b) {
  return b.map((r) => [...r]);
}

function countNonZero(board) {
  let n = 0;
  for (const row of board) {
    for (const v of row) {
      if (v !== 0) n++;
    }
  }
  return n;
}

console.log('=== Running 2048 Logic Verification Tests ===\n');

// 1. Empty board
let empty = GameLogic.createEmptyBoard();
assert.deepStrictEqual(empty, [
  [0, 0, 0, 0],
  [0, 0, 0, 0],
  [0, 0, 0, 0],
  [0, 0, 0, 0],
]);
assert.strictEqual(GameLogic.SIZE, 4);
console.log('✓ createEmptyBoard works');

// 2. slideRowLeft: merge once
let row = [2, 2, 2, 2];
let res = GameLogic.slideRowLeft(row);
assert.deepStrictEqual(res.newRow, [4, 4, 0, 0]);
assert.strictEqual(res.addedScore, 8);
assert.strictEqual(res.changed, true);
assert.deepStrictEqual(res.mergeCols, [0, 1]);
console.log('✓ slideRowLeft 2 2 2 2 -> 4 4 0 0, score +8, two merges');

// 3. Spec case: [4,2,2,2] left -> [4,4,2,0]
row = [4, 2, 2, 2];
res = GameLogic.slideRowLeft(row);
assert.deepStrictEqual(res.newRow, [4, 4, 2, 0]);
assert.strictEqual(res.addedScore, 4);
assert.deepStrictEqual(res.mergeCols, [1]);
console.log('✓ slideRowLeft 4 2 2 2 -> 4 4 2 0 (merge once), score +4');

// 4. Another merge case
row = [2, 2, 4, 4];
res = GameLogic.slideRowLeft(row);
assert.deepStrictEqual(res.newRow, [4, 8, 0, 0]);
assert.strictEqual(res.addedScore, 12);
console.log('✓ slideRowLeft 2 2 4 4 -> 4 8 0 0, score +12');

// 5. No change
row = [4, 8, 16, 32];
res = GameLogic.slideRowLeft(row);
assert.deepStrictEqual(res.newRow, [4, 8, 16, 32]);
assert.strictEqual(res.changed, false);
assert.deepStrictEqual(res.mergeCols, []);
console.log('✓ slideRowLeft no-change when packed and no merges');

// 6. moveLeft
let b = makeBoard([
  [2, 2, 0, 0],
  [0, 0, 0, 0],
  [0, 0, 0, 0],
  [0, 0, 0, 0],
]);
let m = GameLogic.moveLeft(b);
assert.deepStrictEqual(m.newBoard[0], [4, 0, 0, 0]);
assert.strictEqual(m.scoreDelta, 4);
assert.strictEqual(m.moved, true);
assert.deepStrictEqual(m.merges, [{ r: 0, c: 0 }]);
console.log('✓ moveLeft merges and reports merge positions');

// 7. moveRight
b = makeBoard([
  [2, 0, 0, 0],
  [0, 0, 0, 0],
  [0, 0, 0, 0],
  [0, 0, 0, 0],
]);
m = GameLogic.moveRight(b);
assert.deepStrictEqual(m.newBoard[0], [0, 0, 0, 2]);
assert.strictEqual(m.moved, true);
console.log('✓ moveRight works');

// 8. moveUp
b = makeBoard([
  [0, 2, 0, 0],
  [0, 2, 0, 0],
  [0, 0, 0, 0],
  [0, 0, 0, 0],
]);
m = GameLogic.moveUp(b);
assert.deepStrictEqual(m.newBoard, [
  [0, 4, 0, 0],
  [0, 0, 0, 0],
  [0, 0, 0, 0],
  [0, 0, 0, 0],
]);
assert.strictEqual(m.scoreDelta, 4);
assert.deepStrictEqual(m.merges, [{ r: 0, c: 1 }]);
console.log('✓ moveUp merges vertically with correct merge coords');

// 9. moveDown
b = makeBoard([
  [0, 0, 0, 0],
  [0, 0, 0, 0],
  [0, 4, 0, 0],
  [0, 4, 0, 0],
]);
m = GameLogic.moveDown(b);
assert.deepStrictEqual(m.newBoard, [
  [0, 0, 0, 0],
  [0, 0, 0, 0],
  [0, 0, 0, 0],
  [0, 8, 0, 0],
]);
assert.strictEqual(m.scoreDelta, 8);
assert.deepStrictEqual(m.merges, [{ r: 3, c: 1 }]);
console.log('✓ moveDown works with correct merge coords');

// 10. Invalid move
b = makeBoard([
  [2, 4, 8, 16],
  [32, 64, 128, 256],
  [512, 1024, 2, 4],
  [8, 16, 32, 64],
]);
m = GameLogic.move(b, 'left');
assert.strictEqual(m.moved, false);
console.log('✓ move reports moved=false when invalid');

const before = deepClone(b);
let perf = GameLogic.performMove(b, 'left');
assert(GameLogic.boardsEqual(perf.board, before));
assert.strictEqual(perf.moved, false);
assert.strictEqual(perf.scoreDelta, 0);
assert.strictEqual(perf.spawned, null);
console.log('✓ performMove does NOT change board or spawn on invalid move');

// 11. Win detection
const winBoard = makeBoard([
  [0, 0, 0, 0],
  [0, 2048, 0, 0],
  [0, 0, 0, 0],
  [0, 0, 0, 0],
]);
assert.strictEqual(GameLogic.isGameWon(winBoard), true);
assert.strictEqual(GameLogic.isGameWon(GameLogic.createEmptyBoard()), false);

const higherWin = makeBoard([
  [4096, 0, 0, 0],
  [0, 0, 0, 0],
  [0, 0, 0, 0],
  [0, 0, 0, 0],
]);
assert.strictEqual(GameLogic.isGameWon(higherWin), true);
console.log('✓ isGameWon detects 2048 and higher');

// 12. Lose detection
const loseBoard = makeBoard([
  [2, 4, 8, 16],
  [4, 8, 16, 32],
  [8, 16, 32, 64],
  [16, 32, 64, 128],
]);
assert.strictEqual(GameLogic.isGameOver(loseBoard), true);
assert.strictEqual(GameLogic.hasPossibleMoves(loseBoard), false);
console.log('✓ isGameOver true when full + no adjacent matches');

const almostLose = makeBoard([
  [2, 4, 8, 16],
  [4, 8, 16, 32],
  [8, 16, 32, 64],
  [16, 32, 2, 2],
]);
assert.strictEqual(GameLogic.isGameOver(almostLose), false);
assert.strictEqual(GameLogic.hasPossibleMoves(almostLose), true);
console.log('✓ hasPossibleMoves detects adjacent same even when full');

// 13. Spawn with deterministic rng
let spawnBoard = GameLogic.createEmptyBoard();
let mockRng = (() => {
  let calls = 0;
  const seq = [0.0, 0.1]; // first empty, value 2
  return () => seq[calls++];
})();
let addRes = GameLogic.addRandomTile(spawnBoard, mockRng);
assert.strictEqual(addRes.added, true);
assert.strictEqual(addRes.value, 2);
assert.deepStrictEqual(addRes.position, [0, 0]);
assert.strictEqual(addRes.newBoard[0][0], 2);
console.log('✓ addRandomTile uses rng for position and 2/4 value (90% 2)');

mockRng = (() => {
  let calls = 0;
  const seq = [0.2, 0.95]; // value 4
  return () => seq[calls++];
})();
addRes = GameLogic.addRandomTile(addRes.newBoard, mockRng);
assert.strictEqual(addRes.value, 4);
assert.strictEqual(addRes.added, true);
assert.strictEqual(countNonZero(addRes.newBoard), 2);
console.log('✓ second spawn correctly placed 4');

// 14. Initial board has exactly 2 tiles
const initB = GameLogic.createInitialBoard(() => 0.0);
assert.strictEqual(countNonZero(initB), 2);
console.log('✓ createInitialBoard places exactly TWO tiles');

// 15. performMove: merge + spawn + score + metadata
const start = GameLogic.createEmptyBoard();
start[0][0] = 2;
start[0][1] = 2;
const perfRes = GameLogic.performMove(start, 'left', () => 0.3);
assert(perfRes.moved);
assert.strictEqual(perfRes.scoreDelta, 4);
assert.strictEqual(perfRes.board[0][0], 4);
assert.deepStrictEqual(perfRes.merges, [{ r: 0, c: 0 }]);
assert(perfRes.spawned);
assert(Array.isArray(perfRes.spawned.position));
assert.strictEqual(countNonZero(perfRes.board), 2);
console.log('✓ performMove: merge + spawn + score + spawned metadata');

// 16. boardsEqual
assert(
  GameLogic.boardsEqual(
    [
      [2, 0, 0, 0],
      [0, 0, 0, 0],
      [0, 0, 0, 0],
      [0, 0, 0, 0],
    ],
    [
      [2, 0, 0, 0],
      [0, 0, 0, 0],
      [0, 0, 0, 0],
      [0, 0, 0, 0],
    ]
  )
);
assert(
  !GameLogic.boardsEqual(
    [
      [2, 0, 0, 0],
      [0, 0, 0, 0],
      [0, 0, 0, 0],
      [0, 0, 0, 0],
    ],
    [
      [4, 0, 0, 0],
      [0, 0, 0, 0],
      [0, 0, 0, 0],
      [0, 0, 0, 0],
    ]
  )
);
console.log('✓ boardsEqual utility');

// 17. moveRight merge coords
b = makeBoard([
  [0, 0, 2, 2],
  [0, 0, 0, 0],
  [0, 0, 0, 0],
  [0, 0, 0, 0],
]);
m = GameLogic.moveRight(b);
assert.deepStrictEqual(m.newBoard[0], [0, 0, 0, 4]);
assert.deepStrictEqual(m.merges, [{ r: 0, c: 3 }]);
console.log('✓ moveRight merge coords mapped correctly');

// 18. Chain does not double-merge: 2 2 2 0 left -> 4 2 0 0
row = [2, 2, 2, 0];
res = GameLogic.slideRowLeft(row);
assert.deepStrictEqual(res.newRow, [4, 2, 0, 0]);
assert.strictEqual(res.addedScore, 4);
console.log('✓ no cascade merge: 2 2 2 0 -> 4 2 0 0');

// 19. Invalid direction
m = GameLogic.move(GameLogic.createEmptyBoard(), 'diagonal');
assert.strictEqual(m.moved, false);
assert.strictEqual(m.scoreDelta, 0);
console.log('✓ invalid direction is a no-op');

// 20. moveLeft per-tile movement metadata (moves)
b = makeBoard([
  [2, 2, 0, 2],
  [0, 0, 0, 0],
  [0, 0, 0, 0],
  [4, 0, 4, 0],
]);
m = GameLogic.moveLeft(b);
// Row 0: [2,2,0,2] -> [4,2,0,0]; sources of the merge are cols 0,1 -> col 0,
// lone 2 from col 3 -> col 1.
assert.deepStrictEqual(m.moves[0], { from: [0, 0], to: [0, 0] });
assert.deepStrictEqual(m.moves[1], { from: [0, 1], to: [0, 0] });
assert.deepStrictEqual(m.moves[2], { from: [0, 3], to: [0, 1] });
// Row 3: [4,0,4,0] -> [8,0,0,0]
assert.deepStrictEqual(m.moves[3], { from: [3, 0], to: [3, 0] });
assert.deepStrictEqual(m.moves[4], { from: [3, 2], to: [3, 0] });
console.log('✓ moveLeft reports per-tile from/to movement metadata');

// 21. moves coordinate mapping for right/up/down
b = makeBoard([
  [2, 0, 0, 2],
  [0, 0, 0, 0],
  [0, 2, 0, 0],
  [0, 0, 2, 0],
]);
m = GameLogic.moveRight(b);
assert.deepStrictEqual(m.newBoard[0], [0, 0, 0, 4]);
assert.deepStrictEqual(
  m.moves
    .filter((mv) => mv.from[0] === 0 && mv.to[0] === 0)
    .sort((x, y) => x.from[1] - y.from[1]),
  [
    { from: [0, 0], to: [0, 3] },
    { from: [0, 3], to: [0, 3] },
  ]
);
m = GameLogic.moveUp(b);
assert.deepStrictEqual(m.newBoard, [
  [2, 2, 2, 2],
  [0, 0, 0, 0],
  [0, 0, 0, 0],
  [0, 0, 0, 0],
]);
const upMoves = m.moves.filter((mv) => mv.to[0] === 0 && mv.from[0] === 2);
assert.deepStrictEqual(upMoves, [{ from: [2, 1], to: [0, 1] }]);
m = GameLogic.moveDown(b);
assert.deepStrictEqual(m.newBoard, [
  [0, 0, 0, 0],
  [0, 0, 0, 0],
  [0, 0, 0, 0],
  [2, 2, 2, 2],
]);
const downMoves = m.moves.filter((mv) => mv.from[0] === 3 || mv.to[0] === 3);
assert(downMoves.length >= 1);
console.log('✓ moveRight/moveUp/moveDown map move coords correctly');

// 22. performMove includes moves metadata
const startB2 = GameLogic.createEmptyBoard();
startB2[0][0] = 2;
startB2[0][1] = 2;
const perfRes2 = GameLogic.performMove(startB2, 'left', () => 0.3);
assert(Array.isArray(perfRes2.moves));
assert.deepStrictEqual(perfRes2.moves, [
  { from: [0, 0], to: [0, 0] },
  { from: [0, 1], to: [0, 0] },
]);
console.log('✓ performMove passes through moves metadata');

// 23. Regression: screenshot board — the two 128s must merge vertically.
// Column is [128, 128, 256, 2], so Up/Down must merge 128+128 (+256).
const screenshotBoard = makeBoard([
  [2, 128, 2, 32],
  [16, 128, 32, 16],
  [4, 256, 16, 2],
  [2, 2, 0, 4],
]);
let upRes = GameLogic.performMove(screenshotBoard, 'up', () => 0);
assert.strictEqual(upRes.moved, true);
assert.strictEqual(upRes.scoreDelta, 256);
assert.deepStrictEqual(
  upRes.board.map((row) => row[1]).slice(0, 3),
  [256, 256, 2]
);
let downRes = GameLogic.performMove(screenshotBoard, 'down', () => 0);
assert.strictEqual(downRes.moved, true);
assert.strictEqual(downRes.scoreDelta, 256);
console.log('✓ screenshot board: adjacent 128s merge on up/down (+256)');

console.log('\n=== ALL TESTS PASSED ===');
console.log(
  'Core logic verified: slide, merge-once, score, spawn, win/lose, no-add-on-invalid, merge coords.'
);
