"use client";

import { useState } from "react";
import "./styles.css";

const setupMessage = "Download https://github.com/orafam/cogmax/releases/download/v0.1.4/cogmax-skill-v0.1.4.tar.gz, install the Cogmax skill in your agent, and activate it. Then follow the skill's instructions to authenticate and connect to Cogmax Memory.";

export default function Home() {
  const [copied, setCopied] = useState(false);
  async function copySetup() {
    try {
      await navigator.clipboard.writeText(setupMessage);
    } catch {
      const textarea = document.createElement("textarea");
      textarea.value = setupMessage;
      textarea.style.position = "fixed";
      textarea.style.opacity = "0";
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand("copy");
      textarea.remove();
    }
    setCopied(true);
    window.setTimeout(() => setCopied(false), 2200);
  }
  return <main className="landing">
    <section className="connect-card" aria-labelledby="page-title">
      <header className="card-header">
        <div className="header-label"><span>Paste into agents</span><div className="agent-icons" aria-label="Cogmax memory tools"><span className="agent-icon cogmax-icon"><img src="/brain-logo.png" alt="" /></span><span className="agent-icon">M</span><span className="agent-icon">C</span><span className="agent-icon">A</span></div></div>
        <div className="header-actions"><span className="free-label">100% free</span><button className="login-button header-copy" onClick={copySetup}>{copied ? "Copied" : "Copy skill"}</button></div>
      </header>
      <div className="card-body">
        <div className="hero-copy">
          <div className="brand-lockup"><img src="/brain-logo.png" alt="Cogmax" /><span>cogmax</span><span className="brand-note">memory for agent work</span></div>
          <div className="eyebrow"><span className="status-dot" /> persistent context for agents</div>
          <h1 id="page-title">Give your agent a memory that stays.</h1>
          <p className="description">Cogmax remembers what was decided, why it mattered, and where the work belongs — so every new session can pick up with confidence.</p>
          <div className="proof-row"><div><strong>Decisions</strong><span>with their rationale</span></div><div><strong>Projects</strong><span>kept in context</span></div><div><strong>One skill</strong><span>for every agent</span></div></div>
        </div>
        <div className="install-panel">
          <div className="install-heading"><span className="panel-index">01</span><div><strong>Connect Cogmax</strong><span>Paste this into your agent</span></div></div>
          <div className="instruction-row"><div className="instruction-copy"><span className="command-label">Install instruction</span><p>{setupMessage}</p></div><div className="instruction-actions"><button className="copy-button" onClick={copySetup}>{copied ? "Copied" : "Copy instruction"}<span aria-hidden="true">{copied ? "✓" : "＋"}</span></button></div></div>
          <div className="install-note"><span>↳</span> The skill opens authentication after installation.</div>
        </div>
      </div>
    </section>
    <footer><span>cogmax vault</span><span>Memory infrastructure for agents</span></footer>
  </main>;
}
