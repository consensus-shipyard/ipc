import { useState, useEffect, useRef } from "react";

const FONTS = `
  @import url('https://fonts.googleapis.com/css2?family=Syne:wght@700;800&family=DM+Sans:wght@300;400;500&family=DM+Mono:wght@400;500&display=swap');
  * { box-sizing: border-box; margin: 0; padding: 0; }
  @keyframes fadeUp   { from { opacity:0; transform:translateY(10px); } to { opacity:1; transform:translateY(0); } }
  @keyframes pulse    { 0%,100%{opacity:1;} 50%{opacity:0.4;} }
  @keyframes flowDash { to { stroke-dashoffset: -24; } }
  @keyframes flowDashFast { to { stroke-dashoffset: -16; } }
  @keyframes nodePulse { 0%,100%{box-shadow:0 0 0 0 rgba(0,201,167,0.0);} 60%{box-shadow:0 0 0 6px rgba(0,201,167,0.10);} }
  .fa  { animation: fadeUp 0.35s ease both; }
  .np  { animation: nodePulse 2s ease-in-out infinite; }
`;

const C = {
  bg:"#070810", surf:"#0d0f1a", surf2:"#111421", surf3:"#161b2e",
  bdr:"#1c2138", bdrHi:"#2a3258",
  teal:"#00c9a7", tealDim:"rgba(0,201,167,0.08)", tealBdr:"rgba(0,201,167,0.22)", tealGlow:"rgba(0,201,167,0.14)",
  amber:"#f5a623", amberDim:"rgba(245,166,35,0.08)", amberBdr:"rgba(245,166,35,0.22)",
  red:"#e05252", redDim:"rgba(224,82,82,0.08)", redBdr:"rgba(224,82,82,0.22)",
  blue:"#4d9de0", blueDim:"rgba(77,157,224,0.08)", blueBdr:"rgba(77,157,224,0.22)",
  violet:"#9b8cf8", violetDim:"rgba(155,140,248,0.08)", violetBdr:"rgba(155,140,248,0.22)",
  txt:"#dde3f0", txt2:"#6b7a99", txt3:"#323c58",
  mono:'"DM Mono","JetBrains Mono",monospace',
  display:'"Syne",sans-serif',
  sans:'"DM Sans",system-ui,sans-serif',
};

const sc  = v => v >= 75 ? C.teal : v >= 50 ? C.amber : C.red;
const sbg = v => v >= 75 ? C.tealDim : v >= 50 ? C.amberDim : C.redDim;
const sbd = v => v >= 75 ? C.tealBdr : v >= 50 ? C.amberBdr : C.redBdr;

const DEV = {
  name:"Karel van Troost", handle:"karelvantroost", role:"Engineering Coordinator · IPC", av:"KV",
  score:84, tier:"senior", period:"2026-Q1",
  effPRs:11, totalPRs:14, rawCommits:83, wCommits:51,
  rawLines:4821, effLines:2190, infPct:55,
  cid:"bafybeig7xq2m...r4p9wk", block:412847,
  dims:[["Problem-solving depth",91],["Code quality signals",87],["Review responsiveness",82],["Consistency over time",79],["Scope of contribution",74],["Documentation habit",61]],
  prs:[
    { t:"Refactor auth middleware to JWT RS256", s:91, w:1.0, a:312, d:287,
      verdict:"Identified a security vulnerability in the existing symmetric JWT implementation and migrated to asymmetric RS256. Demonstrates threat modelling and backward compatibility constraints. Net line reduction despite new test coverage is a strong positive signal." },
    { t:"Retry logic with exponential backoff", s:78, w:1.0, a:94, d:12,
      verdict:"Implements exponential backoff with jitter for transient failures. Solid handling of idempotency edge cases. Somewhat narrow scope but well-executed. Review feedback addressed thoughtfully." },
    { t:"Move helpers to lib/ directory", s:4, w:0.05, a:340, d:340, flagged:true,
      flag:"94% token overlap between source and destination",
      verdict:"Pure relocation without modification. 94% token similarity detected. No tests added, no interfaces changed. Weight: 0.05×" },
    { t:"Fix race condition in queue worker", s:85, w:1.0, a:61, d:44,
      verdict:"Resolved a subtle race condition where concurrent workers could process the same job twice. Correct use of optimistic locking with appropriate TTL handling." },
  ],
};

const BOARD = [
  { h:"0xGameMaster", av:"GM", c:847, s:23, bad:true },
  { h:"CodeSpammer99", av:"CS", c:612, s:31, bad:true },
  { h:"linefiller_dev", av:"LF", c:441, s:18, bad:true },
  { h:"karelvantroost", av:"KV", c:83, s:84, bad:false },
  { h:"sergeyp_dev",   av:"SP", c:71,  s:89, bad:false },
];

const STEPS = [
  { id:"fetch",  label:"Fetching GitHub data",           ms:1800, layer:0,
    out:"GET /users/karelvantroost/repos\nGET .../commits?author=karelvantroost&since=90d\nGET .../pulls?state=closed&creator=karelvantroost\nGET .../pulls/{n}/files   ×14\nGET .../pulls/{n}/reviews ×14\n\n→ 5 repos  83 commits  14 merged PRs\n→ 31 review threads received" },
  { id:"p0",     label:"Analysing: JWT RS256 refactor",  ms:2600, layer:0, pr:0 },
  { id:"p1",     label:"Analysing: Retry + backoff",     ms:1800, layer:0, pr:1 },
  { id:"p2",     label:"Analysing: Move helpers to lib/",ms:2000, layer:0, pr:2 },
  { id:"p3",     label:"Analysing: Fix race condition",  ms:1600, layer:0, pr:3 },
  { id:"gaming", label:"Anti-gaming detection pass",     ms:1600, layer:0,
    out:"Scanning 14 PRs...\n\n[FLAG] PR #41: 94% token overlap → 0.05×\n[FLAG] 6 commits: formatter-only → excluded\n[FLAG] PR #38: generated schema → 0.1×\n\nRaw lines:      4,821\nEffective:      2,190\nInflation:     -55%" },
  { id:"score",  label:"Computing weighted score",       ms:1200, layer:0,
    out:"Weighted PR score:   81.4\nConsistency score:   79.0\n\n(0.82 × 81.4) + (0.18 × 79.0)\n= 66.7 + 14.2\n= 84   tier: senior" },
  { id:"storage",label:"Writing evidence to IPC Storage",ms:1000, layer:1,
    out:"POST reputation_evidence_karelvantroost_2026Q1.json\nSize: 68.4 KB   Replicas: 3/3\n\nCID: bafybeig7xq2m...r4p9wk\nStatus: confirmed ✓" },
  { id:"chain",  label:"WASM actor: on-chain commit",    ms:2600, layer:2, f3:true },
];

const LAYERS = [
  { id:0, label:"Off-chain agent",  sub:"GitHub API · Claude AI",       color:C.blue,   dimColor:C.blueDim,   bdrColor:C.blueBdr,
    items:["Fetches GitHub data","Analyses PR diffs","Detects gaming patterns","Computes score"] },
  { id:1, label:"IPC Storage",      sub:"Content-addressed · Replicated", color:C.violet, dimColor:C.violetDim, bdrColor:C.violetBdr,
    items:["Stores evidence doc (68 KB)","Returns CID","3× replicated","Permanently retrievable"] },
  { id:2, label:"IPC Chain",         sub:"WASM actor · F3 finality",      color:C.teal,   dimColor:C.tealDim,   bdrColor:C.tealBdr,
    items:["Validates agent signature","Writes on-chain record","F3 consensus","Registry queryable"] },
];

const APPS = [
  { title:"DAO governance", badge:"voting weight = score/100",
    desc:"Contribution replaces capital as the basis for influence. Voting power proportional to verified reputation.",
    code:"Proposal #47  quorum: 60%\nkarelvantroost  0.84× weight\nsergeyp_dev     0.89× weight\n0xGameMaster    0.23× weight",
    q:"How would IPC reputation scores work for weighted DAO governance voting?" },
  { title:"Bounty platform", badge:"score gates eligibility",
    desc:"Bounties gated by minimum reputation. No KYC — the on-chain record is the credential.",
    code:"Bounty #447 — 500 FIL\nRequired score: 70+\n\nkarelvantroost  84 → eligible\n0xGameMaster    23 → blocked",
    q:"How would IPC reputation scores work as a gating mechanism for a web3 bounty platform?" },
  { title:"Grant allocation", badge:"score-weighted distribution",
    desc:"Weight FIL+ grants by contribution quality, not first-come-first-served.",
    code:"Round 12 — 10,000 FIL pool\nAllocation = (score/total) × pool\n\nkarelvantroost  84 → 840 FIL\nsergeyp_dev     89 → 890 FIL",
    q:"How could IPC reputation scores weight grant allocation in the Filecoin ecosystem?" },
  { title:"Hiring credential", badge:"verifiable on-chain",
    desc:"A portable, tamper-proof developer profile. The IPC Storage CID lets anyone audit every verdict.",
    code:"karelvantroost  senior  84/100\nPeriod: 2026-Q1\nVerified: IPC Chain · Filecoin Calibration\nEvidence: bafybeig7xq2m...r4p9wk",
    q:"What makes an IPC on-chain reputation score better than a GitHub profile for hiring?" },
];

const CHECKS = [
  { l:"Content integrity", e:"keccak256(fetch(output_cid)) == result_hash", r:"0x9c3a88f2...f291a3 == 0x9c3a88f2...f291a3  ✓" },
  { l:"Agent identity",    e:"ecrecover(result_hash, signature) == agent_address", r:"recovered 0xd8e2...9f1c == registered agent  ✓" },
  { l:"Block timestamp",   e:"block.timestamp verified at block 412,847", r:"2026-03-17T11:24:07Z confirmed  ✓" },
];

const NAV = ["The problem","Live pipeline","Reputation profile","Audit trail","Applications"];

function sendPrompt(prompt) {
  if (typeof window === "undefined" || !prompt) return;
  const q = encodeURIComponent(prompt);
  window.open(`https://chatgpt.com/?q=${q}`, "_blank", "noopener,noreferrer");
}

/* ── IPC LAYER DIAGRAM ─────────────────────────────────── */
function IPCLayerDiagram({ activeLayer, completedLayers }) {
  // connector 0->1 flows when layer 0 done or layer 1 active
  const flow01 = completedLayers.has(0) || activeLayer === 1;
  const flow12 = completedLayers.has(1) || activeLayer === 2;

  return (
    <div style={{ marginBottom:14, padding:"14px 16px", background:C.surf, border:`1px solid ${C.bdr}`, borderRadius:12 }}>
      <div style={{ fontSize:9, fontFamily:C.mono, color:C.txt3, letterSpacing:"0.1em", marginBottom:12 }}>IPC ARCHITECTURE</div>
      <div style={{ display:"flex", alignItems:"center", gap:0 }}>
        {LAYERS.map((l, i) => {
          const isActive    = activeLayer === l.id;
          const isCompleted = completedLayers.has(l.id);
          const isLit       = isActive || isCompleted;
          const col         = isLit ? l.color : C.txt3;
          const bg          = isLit ? l.dimColor : "transparent";
          const bdr         = isLit ? l.bdrColor : C.bdr;
          const isLast      = i === LAYERS.length - 1;
          const flowActive  = i === 0 ? flow01 : flow12;

          return (
            <div key={l.id} style={{ display:"flex", alignItems:"center", flex:1 }}>
              {/* Node */}
              <div className={isActive ? "np" : ""} style={{
                flex:1, padding:"10px 12px", borderRadius:10,
                border:`1px solid ${bdr}`,
                background: bg,
                transition:"all 0.5s ease",
              }}>
                {/* Header row */}
                <div style={{ display:"flex", alignItems:"center", gap:7, marginBottom:7 }}>
                  <LayerIcon id={l.id} color={col} active={isLit} />
                  <div>
                    <div style={{ fontSize:11.5, fontWeight:500, color: isLit ? col : C.txt2, transition:"color 0.5s", lineHeight:1.2 }}>{l.label}</div>
                    <div style={{ fontSize:9.5, fontFamily:C.mono, color: isLit ? col+"99" : C.txt3, marginTop:1, transition:"color 0.5s" }}>{l.sub}</div>
                  </div>
                  {isActive && (
                    <div style={{ marginLeft:"auto", width:6, height:6, borderRadius:"50%", background:col, animation:"pulse 1s ease-in-out infinite", flexShrink:0 }} />
                  )}
                  {isCompleted && (
                    <div style={{ marginLeft:"auto", fontFamily:C.mono, fontSize:9, color:col, flexShrink:0 }}>✓</div>
                  )}
                </div>
                {/* Items */}
                <div style={{ borderTop:`1px solid ${isLit ? bdr : C.bdr}`, paddingTop:7 }}>
                  {l.items.map((item, j) => (
                    <div key={j} style={{ display:"flex", alignItems:"center", gap:5, marginBottom: j < l.items.length-1 ? 3 : 0 }}>
                      <div style={{ width:4, height:4, borderRadius:"50%", flexShrink:0, background: isLit ? col : C.txt3, opacity: isLit ? 0.6 : 0.3, transition:"all 0.5s" }} />
                      <span style={{ fontSize:10, color: isLit ? C.txt2 : C.txt3, transition:"color 0.5s" }}>{item}</span>
                    </div>
                  ))}
                </div>
              </div>

              {/* Connector arrow (between nodes) */}
              {!isLast && (
                <div style={{ width:32, flexShrink:0, display:"flex", flexDirection:"column", alignItems:"center", gap:3 }}>
                  <svg width="32" height="18" style={{ overflow:"visible" }}>
                    <line x1="2" y1="9" x2="28" y2="9"
                      stroke={flowActive ? LAYERS[i+1].color : C.bdr}
                      strokeWidth={flowActive ? 1.5 : 1}
                      strokeDasharray={flowActive ? "4 4" : "3 4"}
                      style={{ transition:"stroke 0.5s",
                        animation: flowActive ? `flowDash 0.6s linear infinite` : "none" }}
                    />
                    <polygon points="24,5 30,9 24,13"
                      fill={flowActive ? LAYERS[i+1].color : C.bdr}
                      style={{ transition:"fill 0.5s" }}
                    />
                  </svg>
                  <span style={{ fontSize:8.5, fontFamily:C.mono, color: flowActive ? LAYERS[i+1].color+"88" : C.txt3, transition:"color 0.5s", letterSpacing:"0.02em" }}>
                    {i === 0 ? "signed doc" : "CID + sig"}
                  </span>
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

function LayerIcon({ id, color, active }) {
  const dim = 24;
  const c = active ? color : C.txt3;
  const bg = active ? `${color}18` : C.surf2;
  const bd = active ? `${color}40` : C.bdr;

  return (
    <div style={{ width:dim, height:dim, borderRadius:6, background:bg, border:`1px solid ${bd}`, display:"flex", alignItems:"center", justifyContent:"center", flexShrink:0, transition:"all 0.5s" }}>
      {id === 0 && (
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
          <circle cx="6" cy="4" r="2.5" stroke={c} strokeWidth="1.2"/>
          <path d="M1 11c0-2.76 2.24-5 5-5s5 2.24 5 5" stroke={c} strokeWidth="1.2" strokeLinecap="round"/>
        </svg>
      )}
      {id === 1 && (
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
          <rect x="1.5" y="2.5" width="9" height="7" rx="1.5" stroke={c} strokeWidth="1.2"/>
          <path d="M1.5 5h9" stroke={c} strokeWidth="1.2"/>
          <circle cx="4" cy="3.75" r="0.6" fill={c}/>
          <circle cx="6" cy="3.75" r="0.6" fill={c}/>
        </svg>
      )}
      {id === 2 && (
        <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
          <polygon points="6,1 11,4 11,8 6,11 1,8 1,4" stroke={c} strokeWidth="1.2" fill="none"/>
          <circle cx="6" cy="6" r="1.5" fill={c} opacity="0.7"/>
        </svg>
      )}
    </div>
  );
}

/* ── SHARED ───────────────────────────────────────────── */
function Card({ children, style, glow }) {
  return (
    <div style={{ background:C.surf, border:`1px solid ${glow ? C.tealBdr : C.bdr}`, borderRadius:12, padding:"1rem 1.25rem", boxShadow: glow ? `0 0 28px ${C.tealGlow}` : "none", ...style }}>{children}</div>
  );
}
function Badge({ children, color=C.teal, style }) {
  return <span style={{ fontFamily:C.mono, fontSize:10, padding:"2px 7px", borderRadius:4, border:`1px solid ${color}40`, background:`${color}12`, color, letterSpacing:"0.03em", ...style }}>{children}</span>;
}
function ScoreChip({ score }) {
  return <span style={{ fontFamily:C.mono, fontSize:11, fontWeight:500, padding:"2px 8px", borderRadius:4, background:sbg(score), border:`1px solid ${sbd(score)}`, color:sc(score) }}>{score}</span>;
}
function Btn({ children, onClick, style }) {
  const [h, setH] = useState(false);
  return (
    <button onClick={onClick} onMouseEnter={() => setH(true)} onMouseLeave={() => setH(false)} style={{ fontFamily:C.sans, fontSize:12, padding:"7px 16px", borderRadius:8, border:`1px solid ${C.tealBdr}`, background: h ? C.tealDim : "transparent", color:C.teal, cursor:"pointer", transition:"background 0.15s", ...style }}>{children}</button>
  );
}

/* ── APP ──────────────────────────────────────────────── */
export default function App() {
  const [scene, setScene] = useState(0);
  const [k, setK] = useState(0);
  function go(i) { setScene(i); setK(n => n+1); }
  return (
    <div style={{ background:C.bg, minHeight:"100vh", fontFamily:C.sans, color:C.txt, padding:"1.25rem" }}>
      <style>{FONTS}</style>
      <div style={{ display:"flex", gap:4, marginBottom:"1.5rem", flexWrap:"wrap" }}>
        {NAV.map((l,i) => {
          const active = scene===i, done = scene>i;
          return <button key={i} onClick={() => go(i)} style={{ padding:"5px 14px", fontSize:11, fontFamily:C.sans, borderRadius:20, cursor:"pointer", transition:"all 0.2s", border: active ? `1px solid ${C.teal}` : done ? `1px solid ${C.tealBdr}` : `1px solid ${C.bdr}`, background: active ? C.tealDim : "transparent", color: active ? C.teal : done ? C.teal+"88" : C.txt2 }}>{done?"✓ ":`${i+1}. `}{l}</button>;
        })}
      </div>
      <div key={k} className="fa">
        {scene===0 && <Problem next={() => go(1)} />}
        {scene===1 && <Pipeline next={() => go(2)} />}
        {scene===2 && <Profile next={() => go(3)} />}
        {scene===3 && <Audit next={() => go(4)} />}
        {scene===4 && <Applications />}
      </div>
    </div>
  );
}

/* ── SCENE 1: PROBLEM ─────────────────────────────────── */
function Problem({ next }) {
  const [show, setShow] = useState(false);
  useEffect(() => { const t = setTimeout(() => setShow(true), 900); return () => clearTimeout(t); }, []);
  const ipc = [...BOARD].sort((a,b) => b.s - a.s);
  return (
    <div>
      <div style={{ textAlign:"center", marginBottom:"1.5rem" }}>
        <div style={{ fontFamily:C.display, fontSize:26, color:C.txt, marginBottom:6, letterSpacing:"-0.02em" }}>The commit leaderboard is broken</div>
        <div style={{ fontSize:13, color:C.txt2, maxWidth:480, margin:"0 auto" }}>Volume beats quality. Developers who game the system outrank developers who do real work.</div>
      </div>
      <div style={{ display:"grid", gridTemplateColumns:"1fr 1fr", gap:12, marginBottom:14 }}>
        <div style={{ borderRadius:12, overflow:"hidden", border:`1px solid ${C.redBdr}` }}>
          <div style={{ padding:"9px 14px", background:C.redDim, borderBottom:`1px solid ${C.redBdr}`, fontSize:10, fontWeight:500, color:C.red, fontFamily:C.mono, letterSpacing:"0.08em" }}>GITHUB — RANKED BY RAW COMMITS</div>
          {BOARD.map((d,i) => (
            <div key={d.h} style={{ display:"flex", alignItems:"center", gap:9, padding:"9px 14px", borderBottom:i<BOARD.length-1?`1px solid ${C.bdr}`:"none", background:d.bad?C.redDim:"transparent" }}>
              <span style={{ fontSize:10, color:C.txt3, fontFamily:C.mono, width:14, flexShrink:0 }}>#{i+1}</span>
              <div style={{ width:28, height:28, borderRadius:"50%", flexShrink:0, background:d.bad?C.redDim:C.tealDim, display:"flex", alignItems:"center", justifyContent:"center", fontSize:9, fontWeight:500, color:d.bad?C.red:C.teal, border:`1px solid ${d.bad?C.redBdr:C.tealBdr}`, fontFamily:C.mono }}>{d.av}</div>
              <span style={{ flex:1, fontSize:11.5, color:d.bad?C.red:C.txt }}>@{d.h}</span>
              <span style={{ fontFamily:C.mono, fontSize:12, color:d.bad?C.red:C.txt }}>{d.c}</span>
              {d.bad && <Badge color={C.red}>GAMED</Badge>}
            </div>
          ))}
        </div>
        <div style={{ borderRadius:12, overflow:"hidden", border:`1px solid ${show?C.tealBdr:C.bdr}`, opacity:show?1:0.1, transition:"all 1s ease", boxShadow:show?`0 0 32px ${C.tealGlow}`:"none" }}>
          <div style={{ padding:"9px 14px", background:show?C.tealDim:C.surf2, borderBottom:`1px solid ${show?C.tealBdr:C.bdr}`, fontSize:10, fontWeight:500, color:show?C.teal:C.txt2, fontFamily:C.mono, letterSpacing:"0.08em", transition:"all 1s" }}>IPC REPUTATION — VERIFIED AI SCORE</div>
          {ipc.map((d,i) => (
            <div key={d.h} style={{ display:"flex", alignItems:"center", gap:9, padding:"9px 14px", borderBottom:i<ipc.length-1?`1px solid ${C.bdr}`:"none" }}>
              <span style={{ fontSize:10, color:C.txt3, fontFamily:C.mono, width:14, flexShrink:0 }}>#{i+1}</span>
              <div style={{ width:28, height:28, borderRadius:"50%", flexShrink:0, background:C.tealDim, display:"flex", alignItems:"center", justifyContent:"center", fontSize:9, fontWeight:500, color:C.teal, border:`1px solid ${C.tealBdr}`, fontFamily:C.mono }}>{d.av}</div>
              <span style={{ flex:1, fontSize:11.5 }}>@{d.h}</span>
              <span style={{ fontFamily:C.mono, fontSize:14, fontWeight:500, color:sc(d.s) }}>{d.s}</span>
              <span style={{ fontSize:10, color:C.txt3 }}>/100</span>
            </div>
          ))}
        </div>
      </div>
      {show && (
        <div style={{ padding:"11px 16px", background:C.surf2, borderRadius:10, borderLeft:`3px solid ${C.teal}`, marginBottom:14, fontSize:12.5, color:C.txt2, lineHeight:1.7 }} className="fa">
          <span style={{ color:C.red, fontFamily:C.mono }}>0xGameMaster</span> has 847 commits and an IPC score of <span style={{ color:C.red, fontFamily:C.mono }}>23</span>.&nbsp;
          <span style={{ color:C.teal, fontFamily:C.mono }}>karelvantroost</span> has 83 commits and a score of <span style={{ color:C.teal, fontFamily:C.mono }}>84</span>. The IPC agent reads the actual diff — it can tell the difference between moving files around and writing real code.
        </div>
      )}
      <div style={{ display:"flex", justifyContent:"flex-end" }}><Btn onClick={next}>Watch IPC score a developer →</Btn></div>
    </div>
  );
}

/* ── SCENE 2: PIPELINE ────────────────────────────────── */
function Pipeline({ next }) {
  const [step, setStep]     = useState(-1);
  const [done, setDone]     = useState(new Set());
  const [running, setRun]   = useState(false);
  const [f3t, setF3t]       = useState(0);
  const [f3fin, setF3fin]   = useState(false);
  const tm  = useRef(null);
  const f3r = useRef(null);
  const allDone = done.has(STEPS.length - 1);

  // Derive which layer is currently active and which are completed
  const curLayer = step >= 0 && step < STEPS.length ? STEPS[step].layer : -1;
  // A layer is only "completed" if ALL its steps are done
  const layerComplete = new Set();
  [0,1,2].forEach(lid => {
    const layerSteps = STEPS.map((s,i) => ({s,i})).filter(({s}) => s.layer === lid);
    if (layerSteps.every(({i}) => done.has(i))) layerComplete.add(lid);
  });

  function start() { setRun(true); setStep(0); }

  useEffect(() => {
    if (!running || step < 0 || step >= STEPS.length) return;
    tm.current = setTimeout(() => {
      setDone(p => new Set([...p, step]));
      if (step + 1 < STEPS.length) setStep(step + 1); else setRun(false);
    }, STEPS[step].ms);
    return () => clearTimeout(tm.current);
  }, [step, running]);

  useEffect(() => {
    if (!running || step !== STEPS.length - 1) return;
    let t = 0;
    f3r.current = setInterval(() => {
      t = parseFloat(Math.min(t + 0.1, 2.4).toFixed(1));
      setF3t(t);
      if (t >= 2.4) { clearInterval(f3r.current); setF3fin(true); }
    }, 70);
    return () => clearInterval(f3r.current);
  }, [step, running]);

  const curS  = step >= 0 && step < STEPS.length ? STEPS[step] : null;
  const curPR = curS && curS.pr !== undefined ? DEV.prs[curS.pr] : null;
  const activeLayer = running ? curLayer : -1;

  return (
    <div>
      {/* IPC Architecture diagram — always visible, lights up as pipeline progresses */}
      <IPCLayerDiagram activeLayer={activeLayer} completedLayers={layerComplete} />

      {/* Stepper + output */}
      <div style={{ display:"grid", gridTemplateColumns:"200px 1fr", gap:14 }}>
        <div>
          <div style={{ fontSize:9, color:C.txt3, fontFamily:C.mono, letterSpacing:"0.06em", padding:"0 4px", marginBottom:10 }}>@KARELVANTROOST</div>
          {STEPS.map((s,i) => {
            const isDone = done.has(i), isActive = i===step && !isDone && running;
            const layerCol = LAYERS[s.layer].color;
            return (
              <div key={s.id} style={{ display:"flex", alignItems:"center", gap:8, padding:"5px 8px", borderRadius:8, background:isActive ? `${layerCol}10` : "transparent", marginBottom:1, border:isActive ? `1px solid ${layerCol}30` : "1px solid transparent" }}>
                <div style={{ width:15, height:15, borderRadius:"50%", flexShrink:0, border:`1px solid ${isDone ? layerCol : isActive ? layerCol : C.bdrHi}`, background:isDone ? layerCol : "transparent", display:"flex", alignItems:"center", justifyContent:"center", fontSize:8, color:isDone ? C.bg : layerCol, fontFamily:C.mono, animation:isActive ? "pulse 1s ease-in-out infinite" : "none" }}>
                  {isDone ? "✓" : ""}
                </div>
                <span style={{ fontSize:10, lineHeight:1.35, color:isDone ? layerCol : isActive ? layerCol : C.txt3 }}>{s.label}</span>
              </div>
            );
          })}
        </div>

        <div>
          <div style={{ background:C.surf, border:`1px solid ${C.bdr}`, borderRadius:12, padding:14, minHeight:200, display:"flex", flexDirection:"column" }}>
            {!running && step < 0 ? (
              <div style={{ flex:1, display:"flex", flexDirection:"column", alignItems:"center", justifyContent:"center", gap:12 }}>
                <div style={{ fontFamily:C.mono, fontSize:11, color:C.txt2 }}>Ready to analyse @karelvantroost</div>
                <Btn onClick={start}>Run pipeline →</Btn>
              </div>
            ) : curS ? <StepOut step={curS} pr={curPR} f3t={f3t} f3fin={f3fin} /> : null}
          </div>
          {allDone && (
            <div style={{ display:"flex", justifyContent:"flex-end", marginTop:10 }} className="fa">
              <Btn onClick={next}>View reputation profile →</Btn>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function StepOut({ step, pr, f3t, f3fin }) {
  if (step.f3) return (
    <div style={{ display:"flex", flexDirection:"column" }}>
      <pre style={{ fontFamily:C.mono, fontSize:10.5, lineHeight:1.9, color:C.txt2, whiteSpace:"pre-wrap", marginBottom:16 }}>{`ReputationRegistry.setScore({\n  developer:    "0xd8e2...9f1c",\n  score:        84,\n  tier:         "senior",\n  evidence_cid: "bafybeig7xq2m...r4p9wk",\n  period:       "2026-Q1"\n})`}</pre>
      <div style={{ textAlign:"center", padding:"10px 0" }}>
        <div style={{ fontFamily:C.mono, fontSize:9, color:C.txt3, letterSpacing:"0.1em", marginBottom:8 }}>F3 FAST FINALITY</div>
        <div style={{ fontFamily:C.display, fontSize:60, lineHeight:1, color:f3fin ? C.teal : C.txt, transition:"color 0.5s", marginBottom:12 }}>{f3t.toFixed(1)}s</div>
        {f3fin
          ? <Badge>FINALIZED ✓</Badge>
          : <span style={{ fontSize:11, color:C.txt3, fontFamily:C.mono }}>waiting for F3 consensus...</span>}
      </div>
    </div>
  );
  if (pr) return (
    <div style={{ border:`1px solid ${pr.flagged ? C.amberBdr : C.bdrHi}`, borderRadius:10, padding:"12px 14px", background:pr.flagged ? C.amberDim : C.surf2 }}>
      <div style={{ display:"flex", justifyContent:"space-between", alignItems:"flex-start", gap:10, marginBottom:6 }}>
        <span style={{ fontSize:13, fontWeight:500, lineHeight:1.35, color:C.txt }}>{pr.t}</span>
        <ScoreChip score={pr.s} />
      </div>
      <div style={{ fontFamily:C.mono, fontSize:10, color:C.txt2, marginBottom:pr.flagged ? 8 : 10 }}>+{pr.a} −{pr.d} &nbsp;·&nbsp; weight {pr.w}×</div>
      {pr.flagged && <div style={{ fontFamily:C.mono, fontSize:10.5, color:C.amber, background:C.amberDim, border:`1px solid ${C.amberBdr}`, borderRadius:6, padding:"5px 10px", marginBottom:10 }}>⚠ FLAGGED: {pr.flag}</div>}
      <div style={{ fontSize:11.5, color:C.txt2, lineHeight:1.6, borderLeft:`2px solid ${pr.flagged ? C.amberBdr : C.tealBdr}`, paddingLeft:10, fontStyle:"italic" }}>{pr.verdict}</div>
    </div>
  );
  return <pre style={{ fontFamily:C.mono, fontSize:10.5, lineHeight:1.9, color:C.txt2, whiteSpace:"pre-wrap" }}>{step.out}</pre>;
}

/* ── SCENE 3: PROFILE ─────────────────────────────────── */
function Profile({ next }) {
  const [disp, setDisp] = useState(0);
  const [bars, setBars] = useState(false);
  const size=140, stroke=8, r=(size-stroke)/2, circ=2*Math.PI*r;
  useEffect(() => {
    let n = 0;
    const id = setInterval(() => {
      n = Math.min(n+1.6, DEV.score); setDisp(Math.round(n));
      if (n >= DEV.score) { clearInterval(id); setTimeout(() => setBars(true), 200); }
    }, 18);
    return () => clearInterval(id);
  }, []);
  return (
    <div>
      <Card style={{ marginBottom:12 }}>
        <div style={{ display:"flex", alignItems:"center", gap:16, paddingBottom:16, marginBottom:16, borderBottom:`1px solid ${C.bdr}` }}>
          <div style={{ position:"relative", flexShrink:0 }}>
            <svg width={size} height={size} style={{ transform:"rotate(-90deg)" }}>
              <circle cx={size/2} cy={size/2} r={r} fill="none" stroke={C.bdr} strokeWidth={stroke} />
              <circle cx={size/2} cy={size/2} r={r} fill="none" stroke={sc(disp)} strokeWidth={stroke} strokeDasharray={circ} strokeDashoffset={circ*(1-disp/100)} strokeLinecap="round" style={{ transition:"stroke-dashoffset 0.05s linear, stroke 0.5s" }} />
            </svg>
            <div style={{ position:"absolute", inset:0, display:"flex", flexDirection:"column", alignItems:"center", justifyContent:"center" }}>
              <div style={{ fontFamily:C.display, fontSize:36, lineHeight:1, color:sc(disp) }}>{disp}</div>
              <div style={{ fontSize:9, color:C.txt2, fontFamily:C.mono, marginTop:2 }}>/ 100</div>
            </div>
          </div>
          <div style={{ flex:1 }}>
            <div style={{ fontFamily:C.display, fontSize:22, color:C.txt, letterSpacing:"-0.02em", marginBottom:2 }}>{DEV.name}</div>
            <div style={{ fontSize:12, color:C.txt2, marginBottom:10 }}>{DEV.role}</div>
            <div style={{ display:"flex", gap:6, flexWrap:"wrap" }}>
              <Badge>{DEV.tier}</Badge>
              <Badge color={C.txt2}>{DEV.period}</Badge>
              <Badge color={C.txt2}>block {DEV.block.toLocaleString()}</Badge>
            </div>
          </div>
        </div>
        <div style={{ display:"grid", gridTemplateColumns:"repeat(4,minmax(0,1fr))", gap:8, marginBottom:16 }}>
          {[["Effective PRs",`${DEV.effPRs}/${DEV.totalPRs}`],["Weighted commits",`${DEV.wCommits}/${DEV.rawCommits}`],["Inflation removed",`${DEV.infPct}%`],["IPC Storage CID",DEV.cid]].map(([l,v]) => (
            <div key={l} style={{ background:C.surf2, borderRadius:8, padding:"9px 11px", border:`1px solid ${C.bdr}` }}>
              <div style={{ fontSize:10, color:C.txt2, marginBottom:5 }}>{l}</div>
              <div style={{ fontFamily:C.mono, fontSize:11.5, fontWeight:500, wordBreak:"break-all", color:C.txt }}>{v}</div>
            </div>
          ))}
        </div>
        <div style={{ marginBottom:16 }}>
          {DEV.dims.map(([l,s]) => (
            <div key={l} style={{ display:"flex", alignItems:"center", gap:10, marginBottom:7 }}>
              <span style={{ fontSize:11, color:C.txt2, width:188, flexShrink:0 }}>{l}</span>
              <div style={{ flex:1, height:4, background:C.surf3, borderRadius:2, overflow:"hidden" }}>
                <div style={{ height:"100%", borderRadius:2, background:sc(s), width:bars?s+"%":"0%", transition:"width 1.1s cubic-bezier(0.16,1,0.3,1)" }} />
              </div>
              <span style={{ fontFamily:C.mono, fontSize:11, fontWeight:500, color:sc(s), width:24, textAlign:"right" }}>{s}</span>
            </div>
          ))}
        </div>
        <div style={{ borderTop:`1px solid ${C.bdr}`, paddingTop:12 }}>
          {DEV.prs.map((pr,i) => (
            <div key={i} style={{ display:"flex", justifyContent:"space-between", alignItems:"center", padding:"7px 0", borderBottom:i<DEV.prs.length-1?`1px solid ${C.bdr}`:"none" }}>
              <span style={{ fontSize:11.5, color:pr.flagged?C.amber:C.txt, flex:1, overflow:"hidden", textOverflow:"ellipsis", whiteSpace:"nowrap", marginRight:12 }}>{pr.flagged?"⚠ ":""}{pr.t}</span>
              <div style={{ display:"flex", alignItems:"center", gap:8, flexShrink:0 }}>
                <span style={{ fontFamily:C.mono, fontSize:10, color:C.txt3 }}>+{pr.a}/−{pr.d}</span>
                <ScoreChip score={pr.s} />
                <span style={{ fontFamily:C.mono, fontSize:9.5, color:C.txt3 }}>{pr.w}×</span>
              </div>
            </div>
          ))}
        </div>
        <div style={{ marginTop:12, paddingTop:12, borderTop:`1px solid ${C.bdr}`, display:"flex", justifyContent:"space-between", alignItems:"center" }}>
          <span style={{ fontFamily:C.mono, fontSize:10, color:C.txt3 }}>F3 finalized · IPC Chain · Filecoin Calibration</span>
          <span style={{ fontFamily:C.mono, fontSize:10, color:C.teal+"88" }}>{DEV.cid}</span>
        </div>
      </Card>
      <div style={{ display:"flex", justifyContent:"flex-end" }}><Btn onClick={next}>Verify the evidence →</Btn></div>
    </div>
  );
}

/* ── SCENE 4: AUDIT ───────────────────────────────────── */
function Audit({ next }) {
  const [states, setStates] = useState(["idle","idle","idle"]);
  const [done, setDone]     = useState(false);
  function run() {
    if (done || states[0] !== "idle") return;
    CHECKS.forEach((_,i) => {
      setTimeout(() => {
        setStates(p => { const n=[...p]; n[i]="run"; return n; });
        setTimeout(() => {
          setStates(p => { const n=[...p]; n[i]="ok"; return n; });
          if (i===2) setDone(true);
        }, 700 + Math.random()*300);
      }, i*1000);
    });
  }
  return (
    <div>
      <div style={{ fontSize:12.5, color:C.txt2, lineHeight:1.75, marginBottom:14 }}>Three independent cryptographic checks. Anyone can run them — no trusted party required. The math either checks out or it doesn't.</div>
      {states[0]==="idle" && <Btn onClick={run} style={{ width:"100%", marginBottom:14, textAlign:"center" }}>Run verification →</Btn>}
      {CHECKS.map((c,i) => {
        const st = states[i];
        return (
          <div key={i} style={{ display:"flex", alignItems:"flex-start", gap:12, padding:"12px 14px", marginBottom:8, borderRadius:10, border:`1px solid ${st==="ok"?C.tealBdr:st==="run"?C.amberBdr:C.bdr}`, background:st==="ok"?C.tealDim:st==="run"?C.amberDim:C.surf, transition:"all 0.35s" }}>
            <div style={{ width:20, height:20, borderRadius:"50%", flexShrink:0, marginTop:1, border:`1px solid ${st==="ok"?C.teal:st==="run"?C.amber:C.bdrHi}`, background:st==="ok"?C.teal:"transparent", display:"flex", alignItems:"center", justifyContent:"center", fontSize:10, color:st==="ok"?C.bg:C.txt3, animation:st==="run"?"pulse 0.8s ease-in-out infinite":"none" }}>
              {st==="ok"?"✓":""}
            </div>
            <div>
              <div style={{ fontSize:13.5, fontWeight:500, marginBottom:4, color:C.txt }}>{c.l}</div>
              <div style={{ fontFamily:C.mono, fontSize:10.5, color:st==="ok"?C.teal:C.txt2, lineHeight:1.5 }}>{st==="ok"?c.r:c.e}</div>
            </div>
          </div>
        );
      })}
      {done && (
        <div className="fa">
          <div style={{ padding:"12px 16px", background:C.tealDim, border:`1px solid ${C.tealBdr}`, borderRadius:10, textAlign:"center", fontFamily:C.mono, fontSize:12, color:C.teal, marginBottom:12, letterSpacing:"0.03em" }}>ALL CHECKS PASSED — RECORD CRYPTOGRAPHICALLY VERIFIED</div>
          <div style={{ display:"flex", justifyContent:"flex-end" }}><Btn onClick={next}>See what this enables →</Btn></div>
        </div>
      )}
    </div>
  );
}

/* ── SCENE 5: APPLICATIONS ────────────────────────────── */
function Applications() {
  return (
    <div>
      <div style={{ fontSize:12.5, color:C.txt2, lineHeight:1.75, marginBottom:14 }}>
        Any contract on IPC can now call <span style={{ fontFamily:C.mono, fontSize:11, color:C.teal }}>ReputationRegistry.getScore(address)</span> and get a trustless, auditable developer score. Four things that become immediately possible:
      </div>
      <div style={{ display:"grid", gridTemplateColumns:"1fr 1fr", gap:10 }}>
        {APPS.map((a,i) => (
          <div key={i} style={{ background:C.surf, border:`1px solid ${C.bdr}`, borderRadius:12, padding:"14px 16px", display:"flex", flexDirection:"column", gap:10, transition:"border-color 0.2s" }}
            onMouseEnter={e => e.currentTarget.style.borderColor=C.tealBdr}
            onMouseLeave={e => e.currentTarget.style.borderColor=C.bdr}>
            <div>
              <div style={{ fontFamily:C.display, fontSize:16, color:C.txt, letterSpacing:"-0.01em", marginBottom:5 }}>{a.title}</div>
              <div style={{ fontSize:12, color:C.txt2, lineHeight:1.6 }}>{a.desc}</div>
            </div>
            <pre style={{ fontFamily:C.mono, fontSize:10, color:C.txt2, background:C.surf2, borderRadius:8, padding:"9px 11px", lineHeight:1.85, border:`1px solid ${C.bdr}`, whiteSpace:"pre-wrap" }}>{a.code}</pre>
            <div style={{ display:"flex", justifyContent:"space-between", alignItems:"center" }}>
              <Badge>{a.badge}</Badge>
              <button onClick={() => sendPrompt(a.q)} style={{ fontFamily:C.sans, fontSize:11, padding:"4px 10px", borderRadius:6, border:`1px solid ${C.tealBdr}`, background:"transparent", color:C.teal, cursor:"pointer", transition:"all 0.15s" }}
                onMouseEnter={e => e.currentTarget.style.background=C.tealDim}
                onMouseLeave={e => e.currentTarget.style.background="transparent"}>Explore this ↗</button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
