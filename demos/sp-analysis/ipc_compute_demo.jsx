import { useState, useEffect, useRef } from "react";

const FONTS = `
  @import url('https://fonts.googleapis.com/css2?family=Syne:wght@700;800&family=DM+Sans:wght@300;400;500&family=DM+Mono:wght@400;500&display=swap');
  * { box-sizing: border-box; margin: 0; padding: 0; }
  @keyframes fadeUp   { from{opacity:0;transform:translateY(8px)} to{opacity:1;transform:translateY(0)} }
  @keyframes pulse    { 0%,100%{opacity:1} 50%{opacity:0.35} }
  @keyframes flowDash { to{stroke-dashoffset:-20} }
  @keyframes nodePop  { 0%{transform:scale(1)} 50%{transform:scale(1.03)} 100%{transform:scale(1)} }
  @keyframes scanLine { 0%{transform:translateY(-100%);opacity:0.6} 100%{transform:translateY(400%);opacity:0} }
  .fa { animation: fadeUp 0.3s ease both; }
`;

const C = {
  bg:"#070810", surf:"#0d0f1a", surf2:"#111421", surf3:"#161b2e",
  bdr:"#1c2138", bdrHi:"#2a3258",
  teal:"#00c9a7",   tealDim:"rgba(0,201,167,0.08)",   tealBdr:"rgba(0,201,167,0.22)",   tealGlow:"rgba(0,201,167,0.14)",
  amber:"#f5a623",  amberDim:"rgba(245,166,35,0.08)", amberBdr:"rgba(245,166,35,0.22)", amberGlow:"rgba(245,166,35,0.12)",
  violet:"#9b8cf8", violetDim:"rgba(155,140,248,0.08)",violetBdr:"rgba(155,140,248,0.22)",violetGlow:"rgba(155,140,248,0.10)",
  blue:"#4d9de0",   blueDim:"rgba(77,157,224,0.08)",  blueBdr:"rgba(77,157,224,0.22)",
  red:"#e05252",    redDim:"rgba(224,82,82,0.08)",    redBdr:"rgba(224,82,82,0.22)",
  txt:"#dde3f0", txt2:"#6b7a99", txt3:"#323c58",
  mono:'"DM Mono","Fira Code",monospace', display:'"Syne",sans-serif', sans:'"DM Sans",system-ui,sans-serif',
};

const SPS = [
  { id:"f0116628",  score:106.99, uptime:96.2, deals:106,  power:6.69 },
  { id:"f0149768",  score:105.92, uptime:95.8, deals:83,   power:2.00 },
  { id:"f01393827", score:104.93, uptime:100,  deals:7,    power:0.98 },
  { id:"f0134991",  score:104.55, uptime:89.2, deals:47,   power:6.63 },
  { id:"f03339",    score:103.57, uptime:89.6, deals:941,  power:1.08 },
  { id:"f065103",   score:103.50, uptime:88.8, deals:86,   power:2.16 },
  { id:"f0230200",  score:103.46, uptime:92.3, deals:39,   power:0.72 },
  { id:"f01690643", score:103.03, uptime:100,  deals:675,  power:0.34 },
  { id:"f01694564", score:102.92, uptime:100,  deals:212,  power:0.31 },
  { id:"f044160",   score:102.84, uptime:93.8, deals:1268, power:0.34 },
  { id:"f01690774", score:102.82, uptime:100,  deals:706,  power:0.27 },
  { id:"f018501",   score:102.80, uptime:94.9, deals:461,  power:0.19 },
  { id:"f01652952", score:102.70, uptime:100,  deals:757,  power:0.23 },
  { id:"f01690781", score:102.69, uptime:100,  deals:953,  power:0.23 },
  { id:"f0121768",  score:102.66, uptime:90.4, deals:14,   power:5.05 },
];

const LOGS = [
  "[14:32:01.203]  Container ipc-llm-worker:2.1 starting",
  "[14:32:01.891]  CUDA device: NVIDIA A100 (40 GB)",
  "[14:32:02.114]  Loading meta-llama/Llama-3.1-8B-Instruct",
  "[14:32:04.732]  Model ready  2.6s  14.2B params",
  "[14:32:04.733]  Job received: 0x4a3f...c91b  prompt: 1,024 tokens",
  "[14:32:05.102]  Fetching Filrep API...",
  "[14:32:05.889]  Retrieved 597 storage providers",
  "[14:32:06.221]  Running reliability analysis...",
  "[14:32:07.341]  Scored 597/597 providers",
  "[14:32:07.342]  Top 15 identified  avg score: 103.8",
  "[14:32:08.001]  Generating markdown report...",
  "[14:32:09.221]  Output complete  2,847 tokens",
  "[14:32:09.223]  Saved /output/report.md  (68.4 KB)  exit 0 ✓",
];

const PROMPT_SHORT = "You are a data analyst. Your goal is to identify the most reliable Filecoin Storage Providers — those with a strong track record of consistent uptime, sector maintenance, and deal fulfilment.\n\nSave your final report to report.md containing:\n- Brief methodology note\n- Ranked table of top 15 most reliable SPs\n- 2–3 paragraph summary of what distinguishes top performers\n\n[+ full Filrep dataset: 597 providers]";

const VALIDATORS = [
  { id:"v1", label:"val-1.ipc-fil.io", region:"us-east" },
  { id:"v2", label:"val-2.ipc-fil.io", region:"eu-west", elected:true },
  { id:"v3", label:"val-3.ipc-fil.io", region:"ap-south" },
  { id:"v4", label:"val-4.ipc-fil.io", region:"us-west" },
];

const LAYERS = [
  { id:0, label:"IPC Chain",   sub:"Contract · Job dispatch",   color:C.teal,   dim:C.tealDim,   bdr:C.tealBdr,   glow:C.tealGlow,
    items:["Receives contract call","Validates tx","Dispatches job to compute","Stores job_id on-chain"] },
  { id:1, label:"IPC Compute", sub:"Validator · Docker · LLM",  color:C.amber,  dim:C.amberDim,  bdr:C.amberBdr,  glow:C.amberGlow,
    items:["Validator elected executor","Spins up Docker container","Runs LLM with prompt","Returns structured output"] },
  { id:2, label:"IPC Output",  sub:"IPC Storage · Registry",    color:C.violet, dim:C.violetDim, bdr:C.violetBdr, glow:C.violetGlow,
    items:["Report → IPC Storage (CID)","Scores → SPScoreRegistry","F3 finality confirmed","Queryable by any contract"] },
];

const NAV = ["Contract call","Validator execution","Outputs committed","On-chain registry","Consumer app"];

/* ── ARCH DIAGRAM ─────────────────────────────────────── */
function ArchDiagram({ scene }) {
  const active = scene < 3 ? scene : -1;
  const completed = new Set(Array.from({length: Math.min(scene, 3)}, (_,i) => i));
  return (
    <div style={{ marginBottom:14, padding:"12px 14px", background:C.surf, border:`1px solid ${C.bdr}`, borderRadius:12 }}>
      <div style={{ fontSize:9, fontFamily:C.mono, color:C.txt3, letterSpacing:"0.1em", marginBottom:10 }}>IPC ARCHITECTURE</div>
      <div style={{ display:"flex", alignItems:"center" }}>
        {LAYERS.map((l,i) => {
          const isActive = active === l.id;
          const isDone   = completed.has(l.id) && active !== l.id;
          const lit      = isActive || isDone;
          const col      = lit ? l.color : C.txt3;
          const flowNext = i < LAYERS.length - 1 && (completed.has(l.id) || active > l.id);
          return (
            <div key={l.id} style={{ display:"flex", alignItems:"center", flex:1 }}>
              <div style={{ flex:1, padding:"9px 11px", borderRadius:10, border:`1px solid ${lit ? l.bdr : C.bdr}`, background: lit ? l.dim : "transparent", boxShadow: isActive ? `0 0 18px ${l.glow}` : "none", transition:"all 0.5s" }}>
                <div style={{ display:"flex", alignItems:"center", gap:7, marginBottom:6 }}>
                  <LayerDot id={l.id} color={col} active={lit} pulse={isActive} />
                  <div>
                    <div style={{ fontSize:11, fontWeight:500, color: lit ? col : C.txt2, transition:"color 0.5s" }}>{l.label}</div>
                    <div style={{ fontSize:9, fontFamily:C.mono, color: lit ? col+"88" : C.txt3, marginTop:1 }}>{l.sub}</div>
                  </div>
                  {isDone && <div style={{ marginLeft:"auto", fontFamily:C.mono, fontSize:9, color:col }}>✓</div>}
                  {isActive && <div style={{ marginLeft:"auto", width:5, height:5, borderRadius:"50%", background:col, animation:"pulse 1s ease-in-out infinite", flexShrink:0 }} />}
                </div>
                <div style={{ borderTop:`1px solid ${lit ? l.bdr : C.bdr}`, paddingTop:6 }}>
                  {l.items.map((item,j) => (
                    <div key={j} style={{ display:"flex", alignItems:"center", gap:5, marginBottom: j < l.items.length-1 ? 3 : 0 }}>
                      <div style={{ width:3, height:3, borderRadius:"50%", background: lit ? col : C.txt3, opacity: lit ? 0.6 : 0.3, flexShrink:0, transition:"all 0.5s" }} />
                      <span style={{ fontSize:9.5, color: lit ? C.txt2 : C.txt3, transition:"color 0.5s" }}>{item}</span>
                    </div>
                  ))}
                </div>
              </div>
              {i < LAYERS.length - 1 && (
                <div style={{ width:28, flexShrink:0, display:"flex", flexDirection:"column", alignItems:"center", gap:2 }}>
                  <svg width="28" height="16" style={{ overflow:"visible" }}>
                    <line x1="2" y1="8" x2="24" y2="8" stroke={flowNext ? LAYERS[i+1].color : C.bdr} strokeWidth={flowNext ? 1.5 : 1} strokeDasharray={flowNext ? "4 3" : "3 4"} style={{ transition:"stroke 0.5s", animation: flowNext ? "flowDash 0.55s linear infinite" : "none" }} />
                    <polygon points="20,4 27,8 20,12" fill={flowNext ? LAYERS[i+1].color : C.bdr} style={{ transition:"fill 0.5s" }} />
                  </svg>
                  <span style={{ fontSize:8, fontFamily:C.mono, color: flowNext ? LAYERS[i+1].color+"77" : C.txt3, transition:"color 0.5s" }}>{i===0?"job":"output"}</span>
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

function LayerDot({ id, color, active, pulse }) {
  const icons = [
    <svg width="10" height="10" viewBox="0 0 10 10" fill="none"><polygon points="5,1 9,3.5 9,6.5 5,9 1,6.5 1,3.5" stroke={color} strokeWidth="1.2" fill="none"/><circle cx="5" cy="5" r="1.5" fill={color} opacity="0.7"/></svg>,
    <svg width="10" height="10" viewBox="0 0 10 10" fill="none"><rect x="1" y="1" width="8" height="8" rx="1.5" stroke={color} strokeWidth="1.2" fill="none"/><path d="M3 5h4M5 3v4" stroke={color} strokeWidth="1.1" strokeLinecap="round"/></svg>,
    <svg width="10" height="10" viewBox="0 0 10 10" fill="none"><rect x="1" y="3" width="8" height="5" rx="1.5" stroke={color} strokeWidth="1.2" fill="none"/><circle cx="3.5" cy="5.5" r="0.9" fill={color} opacity="0.7"/><path d="M5.5 5.5h2.5" stroke={color} strokeWidth="1" strokeLinecap="round"/><path d="M5.5 4h2.5" stroke={color} strokeWidth="0.7" strokeLinecap="round" opacity="0.5"/></svg>,
  ];
  return (
    <div style={{ width:22, height:22, borderRadius:6, background:`${color}18`, border:`1px solid ${color}40`, display:"flex", alignItems:"center", justifyContent:"center", flexShrink:0, animation: pulse ? "pulse 1.4s ease-in-out infinite" : "none", transition:"all 0.5s" }}>
      {icons[id]}
    </div>
  );
}

/* ── SHARED ───────────────────────────────────────────── */
function Badge({ children, color=C.teal, style }) {
  return <span style={{ fontFamily:C.mono, fontSize:10, padding:"2px 7px", borderRadius:4, border:`1px solid ${color}40`, background:`${color}12`, color, letterSpacing:"0.03em", ...style }}>{children}</span>;
}
function Btn({ children, onClick, style }) {
  const [h,setH] = useState(false);
  return <button onClick={onClick} onMouseEnter={()=>setH(true)} onMouseLeave={()=>setH(false)} style={{ fontFamily:C.sans, fontSize:12, padding:"7px 16px", borderRadius:8, border:`1px solid ${C.tealBdr}`, background: h?C.tealDim:"transparent", color:C.teal, cursor:"pointer", transition:"background 0.15s", ...style }}>{children}</button>;
}
function ScoreBar({ score }) {
  const pct = Math.min((score - 100) / 8 * 100, 100);
  const col = score >= 105 ? C.teal : score >= 103 ? C.amber : C.violet;
  return (
    <div style={{ display:"flex", alignItems:"center", gap:7 }}>
      <div style={{ width:48, height:3, background:C.surf3, borderRadius:2, overflow:"hidden" }}>
        <div style={{ height:"100%", background:col, width:pct+"%", borderRadius:2 }} />
      </div>
      <span style={{ fontFamily:C.mono, fontSize:11, fontWeight:500, color:col }}>{score.toFixed(2)}</span>
    </div>
  );
}

/* ── APP ──────────────────────────────────────────────── */
export default function App() {
  const [scene, setScene] = useState(0);
  const [k, setK] = useState(0);
  function go(i) { setScene(i); setK(n=>n+1); }
  return (
    <div style={{ background:C.bg, minHeight:"100vh", fontFamily:C.sans, color:C.txt, padding:"1.25rem" }}>
      <style>{FONTS}</style>
      <div style={{ display:"flex", gap:4, marginBottom:"1.25rem", flexWrap:"wrap" }}>
        {NAV.map((l,i) => {
          const active=scene===i, done=scene>i;
          return <button key={i} onClick={()=>go(i)} style={{ padding:"5px 14px", fontSize:11, fontFamily:C.sans, borderRadius:20, cursor:"pointer", transition:"all 0.2s", border:active?`1px solid ${C.teal}`:done?`1px solid ${C.tealBdr}`:`1px solid ${C.bdr}`, background:active?C.tealDim:"transparent", color:active?C.teal:done?C.teal+"88":C.txt2 }}>{done?"✓ ":`${i+1}. `}{l}</button>;
        })}
      </div>
      <ArchDiagram scene={scene} />
      <div key={k} className="fa">
        {scene===0 && <ContractCall next={()=>go(1)} />}
        {scene===1 && <ValidatorExecution next={()=>go(2)} />}
        {scene===2 && <OutputsCommitted next={()=>go(3)} />}
        {scene===3 && <OnChainRegistry next={()=>go(4)} />}
        {scene===4 && <ConsumerApp />}
      </div>
    </div>
  );
}

/* ── SCENE 1: CONTRACT CALL ───────────────────────────── */
function ContractCall({ next }) {
  const [showPrompt, setShowPrompt] = useState(false);
  const [txState, setTxState] = useState("idle");
  const [txHash] = useState("0x4a3f8b2e...c91b7d3a");

  function submit() {
    if (txState !== "idle") return;
    setTxState("submitting");
    setTimeout(() => setTxState("submitted"), 1400);
  }

  return (
    <div>
      <div style={{ fontSize:12.5, color:C.txt2, lineHeight:1.75, marginBottom:14 }}>
        A developer calls a single contract function on the IPC chain. The prompt and job parameters are encoded as calldata — the chain handles everything from here.
      </div>
      <div style={{ background:C.surf, border:`1px solid ${C.bdr}`, borderRadius:12, overflow:"hidden", marginBottom:12 }}>
        <div style={{ padding:"9px 14px", background:C.surf2, borderBottom:`1px solid ${C.bdr}`, display:"flex", alignItems:"center", justifyContent:"space-between" }}>
          <div style={{ display:"flex", alignItems:"center", gap:8 }}>
            <div style={{ width:8, height:8, borderRadius:"50%", background:C.teal }} />
            <span style={{ fontFamily:C.mono, fontSize:11, color:C.teal }}>SPAnalysisJob.sol</span>
            <span style={{ fontFamily:C.mono, fontSize:10, color:C.txt3 }}>· IPC Chain · Filecoin Calibration</span>
          </div>
          <Badge color={C.txt2}>0x7f3c...d4a2</Badge>
        </div>
        <div style={{ padding:"14px 16px" }}>
          <div style={{ fontFamily:C.mono, fontSize:12, color:C.teal, marginBottom:12 }}>AnalyseStorageProviders()</div>
          <div style={{ display:"grid", gap:8, marginBottom:12 }}>
            {[
              ["job_type", '"llm_inference"', C.violet],
              ["model",    '"meta-llama/Llama-3.1-8B-Instruct"', C.amber],
              ["timeout",  "300", C.txt2],
              ["callback", '"SPScoreRegistry.storeResults"', C.teal],
            ].map(([k,v,col]) => (
              <div key={k} style={{ display:"flex", gap:10, alignItems:"baseline" }}>
                <span style={{ fontFamily:C.mono, fontSize:11, color:C.txt3, width:70, flexShrink:0 }}>{k}</span>
                <span style={{ fontFamily:C.mono, fontSize:11, color:col }}>{v}</span>
              </div>
            ))}
            <div style={{ borderTop:`1px solid ${C.bdr}`, paddingTop:10, marginTop:2 }}>
              <div style={{ display:"flex", justifyContent:"space-between", alignItems:"center", marginBottom: showPrompt ? 8 : 0 }}>
                <span style={{ fontFamily:C.mono, fontSize:11, color:C.txt3 }}>prompt</span>
                <button onClick={()=>setShowPrompt(s=>!s)} style={{ fontFamily:C.mono, fontSize:10, color:C.amber, background:"transparent", border:`1px solid ${C.amberBdr}`, borderRadius:4, padding:"1px 7px", cursor:"pointer" }}>{showPrompt?"hide ↑":"view ↓"}</button>
              </div>
              {showPrompt && (
                <pre style={{ fontFamily:C.mono, fontSize:10, color:C.txt2, background:C.surf2, borderRadius:8, padding:"10px 12px", lineHeight:1.75, whiteSpace:"pre-wrap", border:`1px solid ${C.bdr}`, marginTop:2 }} className="fa">{PROMPT_SHORT}</pre>
              )}
            </div>
          </div>
          {txState === "idle" && <Btn onClick={submit} style={{ width:"100%" }}>Submit transaction →</Btn>}
          {txState === "submitting" && (
            <div style={{ padding:"10px 14px", background:C.amberDim, border:`1px solid ${C.amberBdr}`, borderRadius:8, fontFamily:C.mono, fontSize:11, color:C.amber, display:"flex", alignItems:"center", gap:8 }} className="fa">
              <div style={{ width:8, height:8, borderRadius:"50%", background:C.amber, animation:"pulse 0.7s ease-in-out infinite", flexShrink:0 }} />
              Broadcasting to IPC chain...
            </div>
          )}
          {txState === "submitted" && (
            <div className="fa">
              <div style={{ padding:"10px 14px", background:C.tealDim, border:`1px solid ${C.tealBdr}`, borderRadius:8, marginBottom:10 }}>
                <div style={{ fontFamily:C.mono, fontSize:10, color:C.txt3, marginBottom:4 }}>TRANSACTION CONFIRMED</div>
                <div style={{ fontFamily:C.mono, fontSize:11, color:C.teal, marginBottom:2 }}>tx: {txHash}</div>
                <div style={{ fontFamily:C.mono, fontSize:11, color:C.txt2 }}>job_id: <span style={{ color:C.amber }}>0x4a3f...c91b</span> &nbsp;·&nbsp; block: <span style={{ color:C.teal }}>412,847</span></div>
              </div>
              <div style={{ display:"flex", justifyContent:"flex-end" }}><Btn onClick={next}>Watch validator execute →</Btn></div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

/* ── SCENE 2: VALIDATOR EXECUTION ─────────────────────── */
function ValidatorExecution({ next }) {
  const [phase, setPhase] = useState(0);
  const [logLines, setLogLines] = useState([]);
  const logRef = useRef(null);

  useEffect(() => {
    const t0 = setTimeout(() => setPhase(1), 600);
    const t1 = setTimeout(() => setPhase(2), 1400);
    const t2 = setTimeout(() => setPhase(3), 2000);
    return () => { clearTimeout(t0); clearTimeout(t1); clearTimeout(t2); };
  }, []);

  useEffect(() => {
    if (phase !== 3) return;
    let i = 0;
    const id = setInterval(() => {
      if (i >= LOGS.length) { clearInterval(id); setPhase(4); return; }
      setLogLines(p => [...p, LOGS[i]]);
      i++;
    }, 320);
    return () => clearInterval(id);
  }, [phase]);

  useEffect(() => {
    if (logRef.current) logRef.current.scrollTop = logRef.current.scrollHeight;
  }, [logLines]);

  return (
    <div>
      <div style={{ fontSize:12.5, color:C.txt2, lineHeight:1.75, marginBottom:14 }}>
        The IPC chain elects a validator to execute the job. The elected validator spins up a Docker container with the LLM runtime and streams output back to the chain.
      </div>
      {/* Validator grid */}
      <div style={{ display:"grid", gridTemplateColumns:"1fr 1fr", gap:8, marginBottom:12 }}>
        {VALIDATORS.map(v => {
          const elected = v.elected && phase >= 2;
          const electing = v.elected && phase === 1;
          const idle = !v.elected || phase < 1;
          const col = elected ? C.amber : electing ? C.amber : C.txt3;
          return (
            <div key={v.id} style={{ padding:"10px 12px", borderRadius:10, border:`1px solid ${elected ? C.amberBdr : electing ? C.amberBdr : C.bdr}`, background: elected ? C.amberDim : "transparent", boxShadow: elected ? `0 0 18px ${C.amberGlow}` : "none", transition:"all 0.4s" }}>
              <div style={{ display:"flex", alignItems:"center", gap:8 }}>
                <div style={{ width:8, height:8, borderRadius:"50%", background: elected||electing ? C.amber : C.bdrHi, animation: electing ? "pulse 0.5s ease-in-out infinite" : "none", flexShrink:0, transition:"background 0.4s" }} />
                <div style={{ flex:1 }}>
                  <div style={{ fontFamily:C.mono, fontSize:11, color: col, transition:"color 0.4s" }}>{v.label}</div>
                  <div style={{ fontFamily:C.mono, fontSize:9.5, color:C.txt3, marginTop:1 }}>{v.region}</div>
                </div>
                {elected && <Badge color={C.amber}>EXECUTOR</Badge>}
                {!v.elected && phase >= 2 && <span style={{ fontFamily:C.mono, fontSize:9.5, color:C.txt3 }}>standby</span>}
              </div>
            </div>
          );
        })}
      </div>

      {/* Execution log */}
      {phase >= 3 && (
        <div style={{ background:C.surf, border:`1px solid ${C.amberBdr}`, borderRadius:12, overflow:"hidden", marginBottom:12 }} className="fa">
          <div style={{ padding:"8px 12px", background:C.amberDim, borderBottom:`1px solid ${C.amberBdr}`, display:"flex", alignItems:"center", gap:8 }}>
            <div style={{ width:7, height:7, borderRadius:"50%", background:C.amber, animation: phase===3?"pulse 0.8s ease-in-out infinite":"none" }} />
            <span style={{ fontFamily:C.mono, fontSize:10, color:C.amber }}>val-2.ipc-fil.io</span>
            <span style={{ fontFamily:C.mono, fontSize:10, color:C.txt3 }}>·</span>
            <span style={{ fontFamily:C.mono, fontSize:10, color:C.txt3 }}>ipc-llm-worker:2.1</span>
            {phase===4 && <Badge color={C.teal} style={{ marginLeft:"auto" }}>complete ✓</Badge>}
          </div>
          <div ref={logRef} style={{ padding:"10px 12px", maxHeight:190, overflowY:"auto", scrollBehavior:"smooth" }}>
            {logLines.map((l,i) => (
              <div key={i} style={{ fontFamily:C.mono, fontSize:10.5, lineHeight:1.85, color: i===logLines.length-1 ? C.teal : i > logLines.length-3 ? C.txt2 : C.txt3, transition:"color 0.4s" }}>{l}</div>
            ))}
          </div>
        </div>
      )}

      {phase === 4 && (
        <div style={{ display:"flex", justifyContent:"flex-end" }} className="fa">
          <Btn onClick={next}>See outputs committed →</Btn>
        </div>
      )}
    </div>
  );
}

/* ── SCENE 3: OUTPUTS COMMITTED ───────────────────────── */
function OutputsCommitted({ next }) {
  const [storeDone, setStoreDone] = useState(false);
  const [chainDone, setChainDone] = useState(false);
  const [f3t, setF3t] = useState(0);
  const [f3fin, setF3fin] = useState(false);

  useEffect(() => {
    setTimeout(() => setStoreDone(true), 1000);
    setTimeout(() => setChainDone(true), 2200);
  }, []);

  useEffect(() => {
    if (!chainDone) return;
    let t = 0;
    const id = setInterval(() => {
      t = parseFloat(Math.min(t+0.1, 2.4).toFixed(1));
      setF3t(t);
      if (t >= 2.4) { clearInterval(id); setF3fin(true); }
    }, 70);
    return () => clearInterval(id);
  }, [chainDone]);

  return (
    <div>
      <div style={{ fontSize:12.5, color:C.txt2, lineHeight:1.75, marginBottom:14 }}>
        Two outputs are committed in sequence. The full report goes to IPC Storage as a permanent content-addressed document. The scored SP data is written on-chain to the registry contract.
      </div>
      <div style={{ display:"grid", gridTemplateColumns:"1fr 1fr", gap:12, marginBottom:12 }}>

        {/* IPC Storage */}
        <div style={{ border:`1px solid ${storeDone ? C.violetBdr : C.bdr}`, borderRadius:12, padding:"12px 14px", background: storeDone ? C.violetDim : C.surf, transition:"all 0.5s", boxShadow: storeDone ? `0 0 18px ${C.violetGlow}` : "none" }}>
          <div style={{ display:"flex", alignItems:"center", gap:7, marginBottom:10 }}>
            <div style={{ width:8, height:8, borderRadius:"50%", background: storeDone ? C.violet : C.bdrHi, animation: !storeDone ? "pulse 0.8s ease-in-out infinite" : "none", transition:"background 0.5s", flexShrink:0 }} />
            <span style={{ fontSize:11, fontWeight:500, color: storeDone ? C.violet : C.txt2, transition:"color 0.5s" }}>IPC Storage</span>
          </div>
          <pre style={{ fontFamily:C.mono, fontSize:10, color: storeDone ? C.txt2 : C.txt3, lineHeight:1.85, whiteSpace:"pre-wrap", transition:"color 0.5s" }}>{`POST report.md\nSize:     68.4 KB\nReplicas: 3/3`}</pre>
          {storeDone && (
            <div style={{ marginTop:8, paddingTop:8, borderTop:`1px solid ${C.violetBdr}` }} className="fa">
              <div style={{ fontFamily:C.mono, fontSize:9.5, color:C.txt3, marginBottom:3 }}>CID</div>
              <div style={{ fontFamily:C.mono, fontSize:10.5, color:C.violet }}>bafybeig7xq2m...r4p9wk</div>
              <Badge color={C.violet} style={{ marginTop:6 }}>confirmed ✓</Badge>
            </div>
          )}
        </div>

        {/* On-chain registry */}
        <div style={{ border:`1px solid ${chainDone ? C.tealBdr : C.bdr}`, borderRadius:12, padding:"12px 14px", background: chainDone ? C.tealDim : C.surf, transition:"all 0.5s", boxShadow: chainDone ? `0 0 18px ${C.tealGlow}` : "none" }}>
          <div style={{ display:"flex", alignItems:"center", gap:7, marginBottom:10 }}>
            <div style={{ width:8, height:8, borderRadius:"50%", background: chainDone ? C.teal : C.bdrHi, animation: storeDone && !chainDone ? "pulse 0.8s ease-in-out infinite" : "none", transition:"background 0.5s", flexShrink:0 }} />
            <span style={{ fontSize:11, fontWeight:500, color: chainDone ? C.teal : C.txt2, transition:"color 0.5s" }}>SPScoreRegistry</span>
          </div>
          <pre style={{ fontFamily:C.mono, fontSize:10, color: chainDone ? C.txt2 : C.txt3, lineHeight:1.85, whiteSpace:"pre-wrap", transition:"color 0.5s" }}>{`storeResults({\n  f0116628:  106.99\n  f0149768:  105.92\n  f01393827: 104.93\n  ... +12 more\n})`}</pre>
          {chainDone && (
            <div style={{ marginTop:8, paddingTop:8, borderTop:`1px solid ${C.tealBdr}` }} className="fa">
              <div style={{ display:"flex", alignItems:"center", gap:8, marginBottom:6 }}>
                <span style={{ fontFamily:C.mono, fontSize:9.5, color:C.txt3 }}>F3 FINALITY</span>
                <span style={{ fontFamily:C.display, fontSize:22, fontWeight:500, color:f3fin?C.teal:C.txt, transition:"color 0.4s", lineHeight:1 }}>{f3t.toFixed(1)}s</span>
                {f3fin && <Badge>FINAL ✓</Badge>}
              </div>
              <div style={{ fontFamily:C.mono, fontSize:10, color:C.txt3 }}>block 412,848 · 15 providers written</div>
            </div>
          )}
        </div>
      </div>
      {f3fin && (
        <div style={{ display:"flex", justifyContent:"flex-end" }} className="fa">
          <Btn onClick={next}>View on-chain registry →</Btn>
        </div>
      )}
    </div>
  );
}

/* ── SCENE 4: ON-CHAIN REGISTRY ───────────────────────── */
function OnChainRegistry({ next }) {
  const [queryVal, setQueryVal] = useState("");
  const [queryResult, setQueryResult] = useState(null);
  const [querying, setQuerying] = useState(false);

  function runQuery() {
    const id = queryVal.trim() || "f0116628";
    setQuerying(true);
    setQueryResult(null);
    setTimeout(() => {
      const sp = SPS.find(s => s.id === id) || SPS[0];
      setQueryResult(sp);
      setQuerying(false);
    }, 800);
  }

  return (
    <div>
      <div style={{ fontSize:12.5, color:C.txt2, lineHeight:1.75, marginBottom:14 }}>
        The registry is now live on-chain. Any contract or app can call <span style={{ fontFamily:C.mono, fontSize:11, color:C.teal }}>getScore()</span> or <span style={{ fontFamily:C.mono, fontSize:11, color:C.teal }}>getTopN()</span>. The evidence CID traces every score back to the full LLM analysis in IPC Storage.
      </div>

      {/* Table */}
      <div style={{ background:C.surf, border:`1px solid ${C.bdr}`, borderRadius:12, overflow:"hidden", marginBottom:12 }}>
        <div style={{ padding:"8px 12px", background:C.surf2, borderBottom:`1px solid ${C.bdr}`, display:"grid", gridTemplateColumns:"24px 1fr 110px 68px 68px 55px", gap:8 }}>
          {["#","Provider ID","Score","Uptime","Deals","Power"].map(h => (
            <span key={h} style={{ fontSize:9.5, fontFamily:C.mono, color:C.txt3, letterSpacing:"0.05em" }}>{h}</span>
          ))}
        </div>
        {SPS.map((sp,i) => (
          <div key={sp.id} style={{ padding:"7px 12px", borderBottom: i<SPS.length-1?`1px solid ${C.bdr}`:"none", display:"grid", gridTemplateColumns:"24px 1fr 110px 68px 68px 55px", gap:8, alignItems:"center", background: i<3 ? `${LAYERS[0].color}05` : "transparent" }}>
            <span style={{ fontFamily:C.mono, fontSize:10, color:C.txt3 }}>#{i+1}</span>
            <span style={{ fontFamily:C.mono, fontSize:11, color: i<3 ? C.teal : C.txt }}>{sp.id}</span>
            <ScoreBar score={sp.score} />
            <span style={{ fontFamily:C.mono, fontSize:10, color:C.txt2 }}>{sp.uptime.toFixed(1)}%</span>
            <span style={{ fontFamily:C.mono, fontSize:10, color:C.txt2 }}>{sp.deals.toLocaleString()}</span>
            <span style={{ fontFamily:C.mono, fontSize:10, color:C.txt3 }}>{sp.power} PiB</span>
          </div>
        ))}
        <div style={{ padding:"8px 12px", borderTop:`1px solid ${C.bdr}`, fontFamily:C.mono, fontSize:10, color:C.txt3 }}>
          evidence_cid: <span style={{ color:C.violet }}>bafybeig7xq2m...r4p9wk</span> &nbsp;·&nbsp; block 412,848 &nbsp;·&nbsp; job 0x4a3f...c91b
        </div>
      </div>

      {/* Contract read demo */}
      <div style={{ background:C.surf, border:`1px solid ${C.bdr}`, borderRadius:12, padding:"12px 14px", marginBottom:12 }}>
        <div style={{ fontSize:10, fontFamily:C.mono, color:C.txt3, marginBottom:8 }}>CONTRACT READ — SPScoreRegistry.getScore()</div>
        <div style={{ display:"flex", gap:8, alignItems:"center" }}>
          <input value={queryVal} onChange={e=>setQueryVal(e.target.value)} placeholder="f0116628" style={{ flex:1, fontFamily:C.mono, fontSize:11, padding:"6px 10px", background:C.surf2, border:`1px solid ${C.bdr}`, borderRadius:7, color:C.txt, outline:"none" }} />
          <Btn onClick={runQuery} style={{ flexShrink:0 }}>Query →</Btn>
        </div>
        {querying && (
          <div style={{ marginTop:8, fontFamily:C.mono, fontSize:11, color:C.amber, display:"flex", alignItems:"center", gap:6 }} className="fa">
            <div style={{ width:6, height:6, borderRadius:"50%", background:C.amber, animation:"pulse 0.7s ease-in-out infinite" }} />
            Reading from IPC chain...
          </div>
        )}
        {queryResult && !querying && (
          <div style={{ marginTop:8, padding:"10px 12px", background:C.tealDim, border:`1px solid ${C.tealBdr}`, borderRadius:8 }} className="fa">
            <pre style={{ fontFamily:C.mono, fontSize:11, color:C.teal, lineHeight:1.85, whiteSpace:"pre-wrap" }}>{`{\n  id:       "${queryResult.id}",\n  score:    ${queryResult.score},\n  uptime:   ${queryResult.uptime}%,\n  deals:    ${queryResult.deals},\n  power:    ${queryResult.power} PiB,\n  evidence: "bafybeig7xq2m...r4p9wk"\n}`}</pre>
          </div>
        )}
      </div>

      <div style={{ display:"flex", justifyContent:"flex-end" }}>
        <Btn onClick={next}>See a consuming app →</Btn>
      </div>
    </div>
  );
}

/* ── SCENE 5: CONSUMER APP ────────────────────────────── */
function ConsumerApp() {
  const [uploadState, setUploadState] = useState("idle");
  const [providers, setProviders] = useState([]);
  const [uploadPct, setUploadPct] = useState(0);
  const [uploadDone, setUploadDone] = useState(false);

  function startUpload() {
    if (uploadState !== "idle") return;
    setUploadState("routing");
    setTimeout(() => {
      setProviders(SPS.slice(0,3));
      setUploadState("routed");
      setTimeout(() => {
        setUploadState("uploading");
        let p = 0;
        const id = setInterval(() => {
          p = Math.min(p + 2, 100);
          setUploadPct(p);
          if (p >= 100) { clearInterval(id); setUploadDone(true); }
        }, 45);
      }, 900);
    }, 1000);
  }

  return (
    <div>
      <div style={{ fontSize:12.5, color:C.txt2, lineHeight:1.75, marginBottom:14 }}>
        Any app can now route storage decisions using the on-chain scores — without running its own analysis, without trusting a third party. The LLM ran once; the result is permanent infrastructure.
      </div>

      {/* App UI */}
      <div style={{ background:C.surf, border:`1px solid ${C.bdr}`, borderRadius:12, overflow:"hidden", marginBottom:12 }}>
        <div style={{ padding:"9px 14px", background:C.surf2, borderBottom:`1px solid ${C.bdr}`, display:"flex", alignItems:"center", gap:8 }}>
          <div style={{ width:7, height:7, borderRadius:"50%", background:C.teal }} />
          <span style={{ fontSize:11, fontFamily:C.mono, color:C.txt2 }}>storage-app · upload.js</span>
        </div>
        <div style={{ padding:"14px 16px" }}>
          {/* File card */}
          <div style={{ display:"flex", alignItems:"center", gap:12, padding:"10px 12px", background:C.surf2, borderRadius:9, border:`1px solid ${C.bdr}`, marginBottom:12 }}>
            <svg width="28" height="28" viewBox="0 0 28 28" fill="none">
              <rect x="3" y="2" width="18" height="24" rx="3" stroke={C.teal} strokeWidth="1.4" fill={C.tealDim}/>
              <path d="M14 2v7h7" stroke={C.teal} strokeWidth="1.4" fill="none"/>
              <rect x="7" y="13" width="10" height="1.2" rx="0.6" fill={C.teal} opacity="0.5"/>
              <rect x="7" y="16.5" width="7" height="1.2" rx="0.6" fill={C.teal} opacity="0.3"/>
            </svg>
            <div style={{ flex:1 }}>
              <div style={{ fontSize:12, fontWeight:500, color:C.txt }}>filecoin_datasets_2026Q1.tar.gz</div>
              <div style={{ fontSize:11, color:C.txt2, marginTop:1 }}>2.4 GB &nbsp;·&nbsp; 14 files</div>
            </div>
            {uploadState === "idle" && <Btn onClick={startUpload}>Upload →</Btn>}
            {uploadState !== "idle" && !uploadDone && <Badge color={C.amber}>{uploadState}</Badge>}
            {uploadDone && <Badge>stored ✓</Badge>}
          </div>

          {/* Routing step */}
          {uploadState !== "idle" && (
            <div style={{ marginBottom: providers.length ? 10 : 0 }} className="fa">
              <div style={{ fontFamily:C.mono, fontSize:10.5, color:C.txt3, marginBottom:6 }}>
                <span style={{ color:C.teal }}>SPScoreRegistry</span>.getTopN(3) →
              </div>
              {providers.length === 0 && (
                <div style={{ fontFamily:C.mono, fontSize:11, color:C.amber, display:"flex", alignItems:"center", gap:6 }}>
                  <div style={{ width:6, height:6, borderRadius:"50%", background:C.amber, animation:"pulse 0.7s ease-in-out infinite" }} />
                  Reading on-chain scores...
                </div>
              )}
            </div>
          )}

          {/* Provider cards */}
          {providers.length > 0 && (
            <div style={{ display:"grid", gridTemplateColumns:"1fr 1fr 1fr", gap:8, marginBottom:12 }} className="fa">
              {providers.map((sp,i) => (
                <div key={sp.id} style={{ padding:"9px 11px", background:C.tealDim, border:`1px solid ${C.tealBdr}`, borderRadius:9 }}>
                  <div style={{ fontFamily:C.mono, fontSize:10, color:C.teal, marginBottom:4 }}>{sp.id}</div>
                  <div style={{ fontFamily:C.mono, fontSize:13, fontWeight:500, color:C.teal, marginBottom:2 }}>{sp.score.toFixed(2)}</div>
                  <div style={{ fontFamily:C.mono, fontSize:9.5, color:C.txt3 }}>rank #{i+1}</div>
                </div>
              ))}
            </div>
          )}

          {/* Upload progress */}
          {uploadState === "uploading" && (
            <div className="fa">
              <div style={{ display:"flex", justifyContent:"space-between", alignItems:"center", marginBottom:5 }}>
                <span style={{ fontFamily:C.mono, fontSize:10.5, color:C.txt2 }}>Distributing to 3 providers</span>
                <span style={{ fontFamily:C.mono, fontSize:11, color:C.teal }}>{uploadPct}%</span>
              </div>
              <div style={{ height:4, background:C.surf3, borderRadius:2, overflow:"hidden" }}>
                <div style={{ height:"100%", background:C.teal, width:uploadPct+"%", borderRadius:2, transition:"width 0.05s linear" }} />
              </div>
            </div>
          )}

          {uploadDone && (
            <div style={{ padding:"10px 14px", background:C.tealDim, border:`1px solid ${C.tealBdr}`, borderRadius:9, fontFamily:C.mono, fontSize:11, color:C.teal, marginTop:8 }} className="fa">
              Stored across f0116628 · f0149768 · f01393827 &nbsp;·&nbsp; deal IDs on-chain ✓
            </div>
          )}
        </div>
      </div>

      <div style={{ padding:"11px 14px", background:C.surf2, borderRadius:10, borderLeft:`3px solid ${C.amber}`, fontSize:12, color:C.txt2, lineHeight:1.7 }}>
        The LLM analysis ran once on IPC Compute. The scores are permanent on-chain state. Every app that uploads to Filecoin can now route to the best providers — for free, forever, with no API key and no trusted intermediary.
      </div>
    </div>
  );
}
