"use client";

import { useState } from "react";
import "./styles.css";

const setupMessage = "Download https://github.com/orafam/cogmax/releases/download/v0.1.4/cogmax-skill-v0.1.4.tar.gz, install the Cogmax skill in your agent, and activate it. Then follow the skill's instructions to authenticate and connect to Cogmax Memory.";
const skillUrl = "https://github.com/orafam/cogmax/releases/download/v0.1.4/cogmax-skill-v0.1.4.tar.gz";

export default function Home() {
  const [copied, setCopied] = useState(false);
  async function copySetup() {
    await navigator.clipboard.writeText(setupMessage);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 2200);
  }
  return <main className="landing">
    <section className="connect-card" aria-labelledby="page-title">
      <header className="card-header">
        <div className="header-label"><span>Paste into agents</span><div className="agent-icons" aria-label="Cogmax memory tools"><span className="agent-icon cogmax-icon"><img src="/brain-logo.png" alt="" /></span><span className="agent-icon">M</span><span className="agent-icon">C</span><span className="agent-icon">A</span></div></div>
        <div className="header-actions"><span className="free-label">100% free</span><button className="login-button header-copy" onClick={copySetup}>{copied ? "Copied" : "Copy skill"} &nbsp;↗</button></div>
      </header>
      <div className="card-body">
        <div className="brand-lockup"><img src="/brain-logo.png" alt="Cogmax" /><span>cogmax</span><span className="brand-note">memory for agent work</span></div>
        <h1 id="page-title">Your agent should not start from zero.</h1>
        <p className="description">Cogmax keeps the context that makes work continuous: what was decided, why it was decided, and which project it belongs to.</p>
        <div className="instruction-row"><p>{setupMessage}</p><div className="instruction-actions"><button className="copy-button" onClick={copySetup}>{copied ? "Copied" : "Copy instruction"} <span>{copied ? "✓" : "↗"}</span></button></div></div>
        <div className="proof-row"><div><strong>Decisions</strong><span>with their rationale</span></div><div><strong>Projects</strong><span>kept in context</span></div><div><strong>One skill</strong><span>for every agent</span></div></div>
        <div className="tool-row"><span className="tool-label">Works quietly in the background</span><span className="separator">·</span><span className="tool-label">Local-first by default</span></div>
      </div>
    </section>
    <footer><span>cogmax vault</span><span>Memory infrastructure for agents</span></footer>
  </main>;
}
