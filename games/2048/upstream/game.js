// game.js — Classic 2048: pure logic + UI + input
// No external dependencies. Logic is exported for Node tests.

// ==================== CONSTANTS ====================

const SIZE = 4;
const WIN_TILE = 2048;
const SPAWN_TWO_CHANCE = 0.9;

const DIRECTIONS = {
  left: 'left',
  right: 'right',
  up: 'up',
  down: 'down',
};

// ==================== PURE GAME LOGIC ====================

function createEmptyBoard() {
  return Array.from({ length: SIZE }, () => Array(SIZE).fill(0));
}

function cloneBoard(board) {
  return board.map((row) => row.slice());
}

function arraysEqual(a, b) {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) return false;
  }
  return true;
}

function boardsEqual(a, b) {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) {
    if (!arraysEqual(a[i], b[i])) return false;
  }
  return true;
}

/**
 * Slide one row left with classic merge-once rules.
 * Returns { newRow, addedScore, changed, mergeCols, groups } where mergeCols are
 * column indices in the resulting row that were produced by a merge, and
 * groups describes each surviving output tile as
 * { value, toCol, fromCols } (fromCols has 2 entries when a merge occurred).
 */
function slideRowLeft(row) {
  const packed = [];
  for (let c = 0; c < SIZE; c++) {
    if (row[c] !== 0) packed.push({ value: row[c], fromCols: [c] });
  }

  const newRow = [];
  const mergeCols = [];
  const groups = [];
  let addedScore = 0;
  let i = 0;

  while (i < packed.length) {
    if (i + 1 < packed.length && packed[i].value === packed[i + 1].value) {
      const merged = packed[i].value * 2;
      mergeCols.push(newRow.length);
      groups.push({
        value: merged,
        toCol: newRow.length,
        fromCols: [...packed[i].fromCols, ...packed[i + 1].fromCols],
      });
      newRow.push(merged);
      addedScore += merged;
      i += 2;
    } else {
      groups.push({
        value: packed[i].value,
        toCol: newRow.length,
        fromCols: packed[i].fromCols,
      });
      newRow.push(packed[i].value);
      i += 1;
    }
  }

  while (newRow.length < SIZE) newRow.push(0);

  return {
    newRow,
    addedScore,
    changed: !arraysEqual(row, newRow),
    mergeCols,
    groups,
  };
}

/** Flatten row groups into board-coordinate move entries. */
function groupsToMoves(groups, r, mapCoord) {
  const moves = [];
  for (const g of groups) {
    for (const fromCol of g.fromCols) {
      moves.push({
        from: mapCoord(r, fromCol),
        to: mapCoord(r, g.toCol),
      });
    }
  }
  return moves;
}

function reverseRow(row) {
  return row.slice().reverse();
}

function reverseRows(board) {
  return board.map(reverseRow);
}

function transpose(board) {
  const out = createEmptyBoard();
  for (let r = 0; r < SIZE; r++) {
    for (let c = 0; c < SIZE; c++) {
      out[c][r] = board[r][c];
    }
  }
  return out;
}

function moveLeft(board) {
  const newBoard = [];
  const merges = [];
  const moves = [];
  let scoreDelta = 0;
  let moved = false;

  for (let r = 0; r < SIZE; r++) {
    const res = slideRowLeft(board[r]);
    newBoard.push(res.newRow);
    scoreDelta += res.addedScore;
    moved = moved || res.changed;
    for (const c of res.mergeCols) {
      merges.push({ r, c });
    }
    moves.push(...groupsToMoves(res.groups, r, (row, col) => [row, col]));
  }

  return { newBoard, scoreDelta, moved, merges, moves };
}

function moveRight(board) {
  const rev = reverseRows(board);
  const res = moveLeft(rev);
  const flipC = ({ r, c }) => ({ r, c: SIZE - 1 - c });
  const mapCoord = (r, c) => [r, SIZE - 1 - c];
  const moves = res.moves.map(({ from, to }) => ({
    from: mapCoord(from[0], from[1]),
    to: mapCoord(to[0], to[1]),
  }));
  return {
    newBoard: reverseRows(res.newBoard),
    scoreDelta: res.scoreDelta,
    moved: res.moved,
    merges: res.merges.map(flipC),
    moves,
  };
}

function moveUp(board) {
  const t = transpose(board);
  const res = moveLeft(t);
  const swap = ([r, c]) => [c, r];
  return {
    newBoard: transpose(res.newBoard),
    scoreDelta: res.scoreDelta,
    moved: res.moved,
    merges: res.merges.map(({ r, c }) => ({ r: c, c: r })),
    moves: res.moves.map(({ from, to }) => ({ from: swap(from), to: swap(to) })),
  };
}

function moveDown(board) {
  const t = transpose(board);
  const res = moveRight(t);
  const swap = ([r, c]) => [c, r];
  return {
    newBoard: transpose(res.newBoard),
    scoreDelta: res.scoreDelta,
    moved: res.moved,
    merges: res.merges.map(({ r, c }) => ({ r: c, c: r })),
    moves: res.moves.map(({ from, to }) => ({ from: swap(from), to: swap(to) })),
  };
}

function move(board, direction) {
  switch (direction) {
    case DIRECTIONS.left:
      return moveLeft(board);
    case DIRECTIONS.right:
      return moveRight(board);
    case DIRECTIONS.up:
      return moveUp(board);
    case DIRECTIONS.down:
      return moveDown(board);
    default:
      return {
        newBoard: cloneBoard(board),
        scoreDelta: 0,
        moved: false,
        merges: [],
        moves: [],
      };
  }
}

function getEmptyPositions(board) {
  const empties = [];
  for (let r = 0; r < SIZE; r++) {
    for (let c = 0; c < SIZE; c++) {
      if (board[r][c] === 0) empties.push([r, c]);
    }
  }
  return empties;
}

/**
 * Place one random tile (90% → 2, 10% → 4) on an empty cell.
 * Optional rng for deterministic tests.
 */
function addRandomTile(board, rng = Math.random) {
  const empties = getEmptyPositions(board);
  if (empties.length === 0) {
    return { newBoard: cloneBoard(board), added: false, position: null, value: null };
  }

  const idx = Math.floor(rng() * empties.length);
  const [r, c] = empties[idx];
  const value = rng() < SPAWN_TWO_CHANCE ? 2 : 4;
  const newBoard = cloneBoard(board);
  newBoard[r][c] = value;

  return {
    newBoard,
    added: true,
    position: [r, c],
    value,
  };
}

/**
 * Full turn: slide/merge, then spawn if anything moved.
 * Returns { board, scoreDelta, moved, merges, spawned }.
 * spawned is { position: [r,c], value } or null.
 */
function performMove(board, direction, rng = Math.random) {
  const moveRes = move(board, direction);
  if (!moveRes.moved) {
    return {
      board: cloneBoard(board),
      scoreDelta: 0,
      moved: false,
      merges: [],
      moves: [],
      spawned: null,
    };
  }

  const addRes = addRandomTile(moveRes.newBoard, rng);
  return {
    board: addRes.newBoard,
    scoreDelta: moveRes.scoreDelta,
    moved: true,
    merges: moveRes.merges,
    moves: moveRes.moves,
    spawned: addRes.added
      ? { position: addRes.position, value: addRes.value }
      : null,
  };
}

function createInitialBoard(rng = Math.random) {
  let board = createEmptyBoard();
  board = addRandomTile(board, rng).newBoard;
  board = addRandomTile(board, rng).newBoard;
  return board;
}

function isGameWon(board) {
  for (let r = 0; r < SIZE; r++) {
    for (let c = 0; c < SIZE; c++) {
      if (board[r][c] >= WIN_TILE) return true;
    }
  }
  return false;
}

function hasPossibleMoves(board) {
  if (getEmptyPositions(board).length > 0) return true;

  for (let r = 0; r < SIZE; r++) {
    for (let c = 0; c < SIZE - 1; c++) {
      if (board[r][c] !== 0 && board[r][c] === board[r][c + 1]) return true;
    }
  }

  for (let c = 0; c < SIZE; c++) {
    for (let r = 0; r < SIZE - 1; r++) {
      if (board[r][c] !== 0 && board[r][c] === board[r + 1][c]) return true;
    }
  }

  return false;
}

function isGameOver(board) {
  return !hasPossibleMoves(board);
}

// ==================== EXPORTS ====================

const GameLogic = {
  SIZE,
  WIN_TILE,
  DIRECTIONS,
  createEmptyBoard,
  cloneBoard,
  boardsEqual,
  slideRowLeft,
  moveLeft,
  moveRight,
  moveUp,
  moveDown,
  move,
  getEmptyPositions,
  addRandomTile,
  performMove,
  createInitialBoard,
  isGameWon,
  isGameOver,
  hasPossibleMoves,
};
if (typeof module !== 'undefined' && module.exports) {
  module.exports = GameLogic;
}

if (typeof window !== 'undefined') {
  window.GameLogic = GameLogic;
}

// ==================== UI LAYER ====================
// Only runs in the browser (skipped under Node when document is missing).

const UI = (() => {
  if (typeof document === 'undefined') return null;

  // Classic desktop layout (matches original 2048 proportions)
  const BASE_TILE = 107;
  const BASE_GAP = 15;
  const BASE_BOARD = SIZE * BASE_TILE + (SIZE + 1) * BASE_GAP; // 503
  const BEST_SCORE_KEY = '2048-best-score';
  const STATE_KEY = '2048-game-state';
  const MOVE_MS = 120;
  const UNDO_LIMIT = 20;

  // Exact classic tile palette
  const TILE_STYLES = {
    2: { bg: '#eee4da', color: '#776e65' },
    4: { bg: '#ede0c8', color: '#776e65' },
    8: { bg: '#f2b179', color: '#f9f6f2' },
    16: { bg: '#f59563', color: '#f9f6f2' },
    32: { bg: '#f67c5f', color: '#f9f6f2' },
    64: { bg: '#f65e3b', color: '#f9f6f2' },
    128: { bg: '#edcf72', color: '#f9f6f2' },
    256: { bg: '#edcc61', color: '#f9f6f2' },
    512: { bg: '#edc850', color: '#f9f6f2' },
    1024: { bg: '#edc53f', color: '#f9f6f2' },
    2048: { bg: '#edc22e', color: '#f9f6f2' },
  };
  const SUPER_TILE = { bg: '#3c3a32', color: '#f9f6f2' };

  const state = {
    board: null,
    score: 0,
    best: 0,
    over: false,
    won: false, // currently showing win overlay
    continuedAfterWin: false, // kept playing past 2048
    history: [], // undo stack: { board, score }
    animGen: 0, // invalidates pending animation timeouts (see animateMove)
    tileSize: BASE_TILE,
    tileGap: BASE_GAP,
    tileIdCounter: 0,
    tileList: [], // { id, value, r, c, el }
  };

  const dom = {
    grid: null,
    tiles: null,
    score: null,
    best: null,
    scoreContainer: null,
    status: null,
    overlay: null,
    overlayTitle: null,
    overlaySubtitle: null,
    overlayButtons: null,
    gameBoard: null,
    container: null,
    undoBtn: null,
  };

  // ---- helpers ----

  function tileStyle(value) {
    return TILE_STYLES[value] || SUPER_TILE;
  }

  function fontSizeFor(value) {
    let base;
    if (value >= 1024) base = 35;
    else if (value >= 128) base = 45;
    else base = 55;
    const scale = state.tileSize / BASE_TILE;
    return `${Math.max(12, Math.round(base * scale))}px`;
  }

  function positionTile(el, r, c) {
    const step = state.tileSize + state.tileGap;
    el.style.transform = `translate(${c * step}px, ${r * step}px)`;
  }

  function announce(text) {
    if (dom.status) dom.status.textContent = text;
  }

  function loadBest() {
    try {
      const n = parseInt(localStorage.getItem(BEST_SCORE_KEY), 10);
      return Number.isFinite(n) && n > 0 ? n : 0;
    } catch {
      return 0;
    }
  }

  function saveBest(value) {
    try {
      localStorage.setItem(BEST_SCORE_KEY, String(value));
    } catch {
      /* private mode / blocked storage */
    }
  }

  function saveState() {
    try {
      localStorage.setItem(
        STATE_KEY,
        JSON.stringify({
          board: state.board,
          score: state.score,
          continuedAfterWin: state.continuedAfterWin,
        })
      );
    } catch {
      /* private mode / blocked storage */
    }
  }

  function loadState() {
    try {
      const raw = localStorage.getItem(STATE_KEY);
      if (!raw) return null;
      const data = JSON.parse(raw);
      const b = data && data.board;
      if (
        !Array.isArray(b) ||
        b.length !== SIZE ||
        b.some((row) => !Array.isArray(row) || row.length !== SIZE ||
          row.some((v) => typeof v !== 'number' || v < 0))
      ) {
        return null;
      }
      if (typeof data.score !== 'number' || data.score < 0) return null;
      return {
        board: b.map((row) => row.map((v) => Math.floor(v))),
        score: Math.floor(data.score),
        continuedAfterWin: !!data.continuedAfterWin,
      };
    } catch {
      return null;
    }
  }

  function clearSavedState() {
    try {
      localStorage.removeItem(STATE_KEY);
    } catch {
      /* noop */
    }
  }

  // ---- layout / responsive ----

  function updateLayout() {
    const margin = 32;
    const target =
      window.innerWidth - margin >= BASE_BOARD
        ? BASE_BOARD
        : Math.max(240, window.innerWidth - margin);

    const scale = target / BASE_BOARD;
    state.tileSize = Math.max(16, Math.round(BASE_TILE * scale));
    state.tileGap = Math.max(2, Math.round(BASE_GAP * scale));
    const boardSize = SIZE * state.tileSize + (SIZE + 1) * state.tileGap;

    const root = document.documentElement;
    root.style.setProperty('--tile-size', `${state.tileSize}px`);
    root.style.setProperty('--tile-gap', `${state.tileGap}px`);
    root.style.setProperty('--board-size', `${boardSize}px`);

    if (dom.gameBoard) {
      dom.gameBoard.style.width = `${boardSize}px`;
      dom.gameBoard.style.height = `${boardSize}px`;
    }

    if (dom.grid) {
      dom.grid.style.width = `${boardSize}px`;
      dom.grid.style.height = `${boardSize}px`;
      dom.grid.style.padding = `${state.tileGap}px`;
      dom.grid.style.gap = `${state.tileGap}px`;
      dom.grid.style.gridTemplateColumns = `repeat(${SIZE}, ${state.tileSize}px)`;
      dom.grid.style.gridTemplateRows = `repeat(${SIZE}, ${state.tileSize}px)`;
    }

    if (dom.tiles) {
      const inner = boardSize - 2 * state.tileGap;
      dom.tiles.style.top = `${state.tileGap}px`;
      dom.tiles.style.left = `${state.tileGap}px`;
      dom.tiles.style.width = `${inner}px`;
      dom.tiles.style.height = `${inner}px`;
    }

    if (dom.container) {
      if (boardSize < BASE_BOARD) {
        dom.container.style.width = '95vw';
        dom.container.style.maxWidth = `${boardSize + 60}px`;
      } else {
        dom.container.style.width = `${boardSize}px`;
        dom.container.style.maxWidth = `${BASE_BOARD}px`;
      }
    }

    // Reposition existing tiles for the new geometry
    for (const t of state.tileList) sizeTile(t);
  }

  // ---- DOM setup ----

  function cacheDom() {
    dom.grid = document.getElementById('grid-container');
    dom.tiles = document.getElementById('tile-container');
    dom.score = document.getElementById('score');
    dom.best = document.getElementById('best-score');
    dom.scoreContainer = document.querySelector('.score-container');
    dom.status = document.getElementById('game-status');
    dom.overlay = document.getElementById('message-overlay');
    dom.overlayTitle = document.getElementById('message-title');
    dom.overlaySubtitle = document.getElementById('message-subtitle');
    dom.overlayButtons = document.getElementById('message-buttons');
    dom.gameBoard = document.getElementById('game-board');
    dom.container = document.querySelector('.container');
    dom.undoBtn = document.getElementById('undo-btn');
  }

  function buildGrid() {
    if (!dom.grid) return;
    dom.grid.innerHTML = '';
    for (let i = 0; i < SIZE * SIZE; i++) {
      const cell = document.createElement('div');
      cell.className = 'grid-cell';
      dom.grid.appendChild(cell);
    }
  }

  // ---- tile elements ----

  function sizeTile(tile) {
    if (!tile.el || !tile.el.parentNode) return;
    tile.el.style.width = `${state.tileSize}px`;
    tile.el.style.height = `${state.tileSize}px`;
    const inner = tile.el.firstChild;
    if (inner) {
      inner.style.fontSize = fontSizeFor(tile.value);
      inner.style.backgroundColor = tileStyle(tile.value).bg;
      inner.style.color = tileStyle(tile.value).color;
    }
    positionTile(tile.el, tile.r, tile.c);
  }

  function createTileObj(value, r, c) {
    return { id: ++state.tileIdCounter, value, r, c, el: null };
  }

  function buildTileEl(tile, { pop = false, merge = false } = {}) {
    const el = document.createElement('div');
    el.className = 'tile';
    const inner = document.createElement('div');
    inner.className = `tile-inner value-${tile.value}`;
    if (pop) inner.classList.add('tile-pop');
    if (merge) inner.classList.add('tile-merge');

    const colors = tileStyle(tile.value);
    inner.style.backgroundColor = colors.bg;
    inner.style.color = colors.color;
    inner.style.fontSize = fontSizeFor(tile.value);
    inner.textContent = String(tile.value);

    el.appendChild(inner);
    el.style.width = `${state.tileSize}px`;
    el.style.height = `${state.tileSize}px`;
    positionTile(el, tile.r, tile.c);
    tile.el = el;
    return el;
  }

  /** Rebuild every tile from state.board. opts.merges replays the merge pulse. */
  function renderFull(opts = {}) {
    if (!dom.tiles || !state.board) return;
    dom.tiles.innerHTML = '';
    state.tileList = [];

    const mergeKeys = new Set(
      (opts.merges || []).map(({ r, c }) => `${r}-${c}`)
    );

    for (let r = 0; r < SIZE; r++) {
      for (let c = 0; c < SIZE; c++) {
        const value = state.board[r][c];
        if (value === 0) continue;
        const tile = createTileObj(value, r, c);
        dom.tiles.appendChild(
          buildTileEl(tile, { merge: mergeKeys.has(`${r}-${c}`) })
        );
        state.tileList.push(tile);
      }
    }

    updateScoreDisplay();
    updateUndoButton();
  }

  /**
   * Animate one move using per-tile movement metadata:
   * slide every source element to its destination, resolve merges
   * (swap the two sources for a pulsing merged tile), then pop the spawn.
   *
   * `gen` is the animation generation: if a newer move / undo / new game
   * happened since, every deferred step below bails out so stale timeouts
   * can never corrupt the live board. After the slide finishes, the DOM is
   * reconciled with the authoritative state.board, so tiles can never
   * desync (stuck tiles, phantom failed merges) no matter how fast moves
   * are queued.
   */
  function animateMove(result, gen) {
    if (!dom.tiles) return;

    const byPos = new Map();
    for (const t of state.tileList) byPos.set(`${t.r}-${t.c}`, t);

    const mergeTargets = []; // { sources: [tile, tile], to: [r, c], value }
    const spawnedTiles = [];

    for (const mv of result.moves) {
      const src = byPos.get(`${mv.from[0]}-${mv.from[1]}`);
      if (!src) continue;
      byPos.delete(`${mv.from[0]}-${mv.from[1]}`);

      const destKey = `${mv.to[0]}-${mv.to[1]}`;
      const other = byPos.get(destKey);

      if (other && other.mergingInto) {
        // Third tile arriving at an occupied merge target cannot happen
        // under 2048 rules; treat defensively as a plain move.
        src.r = mv.to[0];
        src.c = mv.to[1];
        positionTile(src.el, src.r, src.c);
        byPos.set(destKey, src);
        continue;
      }

      if (other) {
        // Merge: slide both sources into the target cell
        src.r = mv.to[0];
        src.c = mv.to[1];
        positionTile(src.el, src.r, src.c);
        other.r = mv.to[0];
        other.c = mv.to[1];
        positionTile(other.el, other.r, other.c);
        other.mergingInto = true;
        mergeTargets.push({ sources: [other, src], to: mv.to });
        byPos.set(destKey, other);
      } else {
        src.r = mv.to[0];
        src.c = mv.to[1];
        positionTile(src.el, src.r, src.c);
        byPos.set(destKey, src);
      }
    }

    // After the slide finishes, replace each merged pair with one new tile
    for (const m of mergeTargets) {
      setTimeout(() => {
        if (gen !== state.animGen) return;
        if (!m.sources.every((t) => t.el && t.el.parentNode)) return;
        for (const t of m.sources) t.el.remove();

        const merged = createTileObj(m.sources[0].value * 2, m.to[0], m.to[1]);
        dom.tiles.appendChild(buildTileEl(merged, { merge: true }));
        const idx = state.tileList.indexOf(m.sources[0]);
        if (idx !== -1) state.tileList.splice(idx, 1, merged);
        state.tileList = state.tileList.filter((t) => t !== m.sources[1]);
      }, MOVE_MS);
    }

    // Spawn
    if (result.spawned) {
      const [sr, sc] = result.spawned.position;
      const tile = createTileObj(result.spawned.value, sr, sc);
      dom.tiles.appendChild(buildTileEl(tile, { pop: true }));
      state.tileList.push(tile);
      setTimeout(() => {
        if (gen !== state.animGen) return;
        const inner = tile.el && tile.el.firstChild;
        if (inner) inner.classList.remove('tile-pop');
      }, 250);
    }

    // Reconcile: rebuild the DOM from the authoritative board so any
    // drift (rapid moves, interrupted merges) is corrected, keeping the
    // merge pulse. This is what prevents stuck / unmergeable tiles.
    setTimeout(() => {
      if (gen !== state.animGen) return;
      renderFull({ merges: result.merges });
      setTimeout(() => {
        if (gen !== state.animGen || !dom.tiles) return;
        dom.tiles.querySelectorAll('.tile-merge').forEach((el) => {
          el.classList.remove('tile-merge');
        });
      }, 250);
    }, MOVE_MS + 60);
  }

  function updateScoreDisplay() {
    if (dom.score) dom.score.textContent = String(state.score);
    if (dom.best) dom.best.textContent = String(state.best);
  }

  function showScorePopup(delta) {
    if (!delta || !dom.scoreContainer) return;
    const el = document.createElement('div');
    el.className = 'score-popup';
    el.setAttribute('aria-hidden', 'true');
    el.textContent = `+${delta}`;
    dom.scoreContainer.appendChild(el);
    setTimeout(() => el.remove(), 700);
  }

  function updateUndoButton() {
    if (dom.undoBtn) dom.undoBtn.disabled = state.history.length === 0;
  }

  // ---- overlay ----

  let lastFocused = null;

  function updateOverlay() {
    if (!dom.overlay) return;

    if (state.over) {
      showOverlay('Game over!', '', [
        { id: 'try-again-btn', label: 'Try again', onClick: newGame },
      ]);
      dom.overlay.className = 'message-overlay game-over';
      return;
    }

    if (state.won) {
      showOverlay('You win!', '', [
        {
          id: 'keep-going-btn',
          label: 'Keep going',
          className: 'game-button keep-going',
          onClick: keepGoing,
        },
        { id: 'try-again-btn', label: 'Try again', onClick: newGame },
      ]);
      dom.overlay.className = 'message-overlay game-win';
      return;
    }

    hideOverlay();
  }

  function showOverlay(title, subtitle, buttons) {
    lastFocused = document.activeElement;
    dom.overlay.style.display = 'flex';
    dom.overlayTitle.textContent = title;
    dom.overlaySubtitle.textContent = subtitle;
    dom.overlayButtons.innerHTML = '';

    for (const btn of buttons) {
      const el = document.createElement('button');
      el.id = btn.id;
      el.className = btn.className || 'game-button';
      el.textContent = btn.label;
      el.addEventListener('click', btn.onClick);
      dom.overlayButtons.appendChild(el);
    }

    const first = dom.overlayButtons.querySelector('button');
    if (first) first.focus();
  }

  function hideOverlay() {
    if (!dom.overlay) return;
    const wasVisible = dom.overlay.style.display !== 'none';
    dom.overlay.style.display = 'none';
    dom.overlay.className = 'message-overlay';
    if (dom.overlayButtons) dom.overlayButtons.innerHTML = '';
    if (wasVisible && lastFocused && typeof lastFocused.focus === 'function') {
      lastFocused.focus();
    }
    lastFocused = null;
  }

  // ---- game actions ----

  function canAcceptInput() {
    // Block during game over or while the win dialog is up
    return !state.over && !state.won && !!state.board;
  }

  function applyScore(delta) {
    if (delta <= 0) return;
    state.score += delta;
    if (state.score > state.best) {
      state.best = state.score;
      saveBest(state.best);
    }
  }

  function pushHistory() {
    state.history.push({
      board: GameLogic.cloneBoard(state.board),
      score: state.score,
    });
    if (state.history.length > UNDO_LIMIT) state.history.shift();
  }

  function executeMove(direction) {
    if (!canAcceptInput()) return;

    const result = GameLogic.performMove(state.board, direction);
    if (!result.moved) return;

    pushHistory();
    state.board = result.board;
    applyScore(result.scoreDelta);
    showScorePopup(result.scoreDelta);

    const gen = ++state.animGen;
    animateMove(result, gen);
    saveState();
    updateScoreDisplay();
    updateUndoButton();
    announce(`Moved ${direction}. Score ${state.score}.`);

    // Win: first time reaching 2048 this run (before "keep going")
    if (!state.continuedAfterWin && GameLogic.isGameWon(state.board)) {
      state.won = true;
      announce('You win! You reached 2048.');
      updateOverlay();
      return;
    }

    if (GameLogic.isGameOver(state.board)) {
      state.over = true;
      announce(`Game over. Final score ${state.score}.`);
      updateOverlay();
    }
  }

  function undo() {
    if (!state.history.length) return;
    const prev = state.history.pop();
    state.board = prev.board;
    state.score = prev.score;
    state.over = false;
    state.won = false;
    state.animGen++; // cancel any in-flight move animation
    // Only stay "past the win" if the restored board is still won
    state.continuedAfterWin =
      state.continuedAfterWin && GameLogic.isGameWon(state.board);

    hideOverlay();
    renderFull();
    saveState();
    announce(`Undid move. Score ${state.score}.`);
  }

  function keepGoing() {
    state.won = false;
    state.continuedAfterWin = true;
    hideOverlay();
  }

  function newGame() {
    state.board = GameLogic.createInitialBoard();
    state.score = 0;
    state.over = false;
    state.won = false;
    state.continuedAfterWin = false;
    state.history = [];
    state.animGen++; // cancel any in-flight move animation

    hideOverlay();
    updateLayout();
    renderFull();
    saveState();
    announce('New game started.');
  }

  // ---- input ----

  const KEY_MAP = {
    ArrowLeft: DIRECTIONS.left,
    a: DIRECTIONS.left,
    A: DIRECTIONS.left,
    ArrowRight: DIRECTIONS.right,
    d: DIRECTIONS.right,
    D: DIRECTIONS.right,
    ArrowUp: DIRECTIONS.up,
    w: DIRECTIONS.up,
    W: DIRECTIONS.up,
    ArrowDown: DIRECTIONS.down,
    s: DIRECTIONS.down,
    S: DIRECTIONS.down,
  };

  const UNDO_KEYS = new Set(['u', 'U', 'z', 'Z']);

  function onKeydown(e) {
    if ((e.ctrlKey || e.metaKey) && UNDO_KEYS.has(e.key)) {
      // Ctrl/Cmd+Z works even from overlays (undo past game over)
      e.preventDefault();
      undo();
      return;
    }
    if (e.ctrlKey || e.metaKey || e.altKey) return;

    const direction = KEY_MAP[e.key];
    if (direction) {
      if (!canAcceptInput()) return;
      e.preventDefault();
      executeMove(direction);
      return;
    }

    if (UNDO_KEYS.has(e.key)) {
      e.preventDefault();
      undo();
    }
  }

  let touchActive = false;
  let touchStartX = 0;
  let touchStartY = 0;
  let touchStartTime = 0;

  function onTouchStart(e) {
    if (!canAcceptInput() || e.touches.length !== 1) return;
    touchActive = true;
    touchStartX = e.touches[0].clientX;
    touchStartY = e.touches[0].clientY;
    touchStartTime = Date.now();
  }

  function onTouchEnd(e) {
    if (!touchActive || !canAcceptInput()) {
      touchActive = false;
      return;
    }
    touchActive = false;

    const endX = e.changedTouches[0].clientX;
    const endY = e.changedTouches[0].clientY;
    const dx = endX - touchStartX;
    const dy = endY - touchStartY;
    const absDx = Math.abs(dx);
    const absDy = Math.abs(dy);
    const elapsed = Date.now() - touchStartTime;

    const minDist = 30;
    if (Math.max(absDx, absDy) < minDist || elapsed > 800) return;

    const direction =
      absDx > absDy
        ? dx > 0
          ? DIRECTIONS.right
          : DIRECTIONS.left
        : dy > 0
          ? DIRECTIONS.down
          : DIRECTIONS.up;

    executeMove(direction);
  }

  function onTouchCancel() {
    touchActive = false;
  }

  // ---- init ----

  let resizeTimer = null;

  function bindEvents() {
    document.addEventListener('keydown', onKeydown);

    if (dom.gameBoard) {
      dom.gameBoard.addEventListener('touchstart', onTouchStart, { passive: true });
      dom.gameBoard.addEventListener('touchend', onTouchEnd, { passive: true });
      dom.gameBoard.addEventListener('touchcancel', onTouchCancel, { passive: true });
    }

    const newBtn = document.getElementById('new-game-btn');
    if (newBtn) newBtn.addEventListener('click', newGame);
    if (dom.undoBtn) dom.undoBtn.addEventListener('click', undo);

    window.addEventListener('resize', () => {
      clearTimeout(resizeTimer);
      resizeTimer = setTimeout(updateLayout, 80);
    });
  }

  function init() {
    cacheDom();
    buildGrid();
    state.best = loadBest();
    updateLayout();
    bindEvents();

    const saved = loadState();
    if (saved) {
      state.board = saved.board;
      state.score = saved.score;
      state.continuedAfterWin = saved.continuedAfterWin;
      renderFull();
      announce('Restored your previous game.');

      // A saved game can be finished or won — resurface the right overlay
      if (GameLogic.isGameOver(state.board)) {
        state.over = true;
        updateOverlay();
      } else if (!state.continuedAfterWin && GameLogic.isGameWon(state.board)) {
        state.won = true;
        updateOverlay();
      }
    } else {
      newGame();
    }
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }

  return { init, newGame, executeMove, undo, state };
})();
