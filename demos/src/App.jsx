import { useMemo, useState } from "react";
import ReputationDemo from "../dev-reputation-mock/ipc_reputation_demo.jsx";
import SPAnalysisDemo from "../sp-analysis/ipc_compute_demo.jsx";

const DEMOS = {
  reputation: {
    label: "dev-reputation-mock",
    component: ReputationDemo,
  },
  sp: {
    label: "sp-analysis",
    component: SPAnalysisDemo,
  },
};

function getDemoFromUrl() {
  const params = new URLSearchParams(window.location.search);
  return params.get("demo") || "reputation";
}

function setDemoInUrl(demo) {
  const url = new URL(window.location.href);
  url.searchParams.set("demo", demo);
  window.history.replaceState(null, "", url.toString());
}

export default function App() {
  const initialDemo = useMemo(() => {
    const requested = getDemoFromUrl();
    return DEMOS[requested] ? requested : "reputation";
  }, []);
  const [demoKey, setDemoKey] = useState(initialDemo);
  const DemoComponent = DEMOS[demoKey].component;

  return (
    <div>
      <div className="demo-switcher">
        {Object.entries(DEMOS).map(([key, { label }]) => (
          <button
            key={key}
            className={key === demoKey ? "active" : ""}
            onClick={() => {
              setDemoInUrl(key);
              setDemoKey(key);
            }}
            type="button"
          >
            {label}
          </button>
        ))}
      </div>

      <DemoComponent />
    </div>
  );
}
