// Compare the production Rust slide operation with the unmodified upstream engine.
// Covers every row over {0,2,4,8,16}, all directions, then 256 seeded full boards.
const assert = require('node:assert/strict');
const {spawnSync} = require('node:child_process');
const path = require('node:path');
const upstream = require('./upstream/game.js');
const cases = [];
const dirs = ['left', 'right', 'up', 'down'];
for (let n=0;n<625;n++) {
  let k=n; const row=[];
  for(let i=0;i<4;i++) { row.push([0,2,4,8,16][k%5]); k=Math.floor(k/5); }
  const board=[row,[0,0,0,0],[0,0,0,0],[0,0,0,0]];
  for(const direction of dirs) cases.push({board,direction});
}
let seed=2048;
for(let n=0;n<256;n++) {
  const board=Array.from({length:4},()=>Array.from({length:4},()=>{
    seed=(Math.imul(seed,1664525)+1013904223)>>>0;
    return [0,2,4,8,16,32,64,128,256,512,1024][seed%11];
  }));
  for(const direction of dirs) cases.push({board,direction});
}
const result=spawnSync('cargo',['run','--quiet','--locked','-p','omarchy-2048','--example','parity'],{
  cwd:path.resolve(__dirname,'../..'), input:JSON.stringify(cases),encoding:'utf8',maxBuffer:16*1024*1024,
});
if(result.status!==0) throw new Error(result.stderr || result.error || `cargo exited ${result.status}`);
const actual=JSON.parse(result.stdout);
assert.equal(actual.length,cases.length);
for(let i=0;i<cases.length;i++) {
  const c=cases[i], expected=upstream.move(c.board,c.direction), got=actual[i];
  assert.deepEqual(got.board,expected.newBoard,`board case ${i}`);
  assert.equal(got.score_delta,expected.scoreDelta,`score case ${i}`);
  assert.equal(got.moved,expected.moved,`moved case ${i}`);
  assert.deepEqual(got.merges,expected.merges.map(({r,c})=>[r,c]),`merges case ${i}`);
  assert.deepEqual(got.moves.map(({from,to})=>({from,to})),expected.moves,`motion case ${i}`);
}
console.log(`PASS: ${cases.length} moves match Avi Barit's pinned upstream engine (boards, scores, merge and motion metadata).`);
