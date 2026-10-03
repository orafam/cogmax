"use client";

import { useState } from "react";
import "./styles.css";

const setupMessage = "Install Cogmax memory for your agent. Use the learn tool to store decisions, preferences and project context, then use recall to retrieve them when needed. Authentication will open automatically when the skill needs it.";

export default function Home() {
  const [copied, setCopied] = useState(false);
  async function copySetup() {
    await navigator.clipboard.writeText(setupMessage);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 2200);
  }
  return <main className="landing">
    <div className="ambient ambient-one" /><div className="ambient ambient-two" />
    <section className="connect-card" aria-labelledby="page-title">
      <header className="card-header">
        <div className="header-label"><span>Paste into agents</span><div className="agent-icons" aria-label="Cogmax memory tools"><span className="agent-icon cogmax-icon"><img src="/brain-logo.png" alt="" /></span><span className="agent-icon">M</span><span className="agent-icon">C</span><span className="agent-icon">A</span></div></div>
        <div className="header-actions"><span className="free-label">100% free</span><button className="login-button header-copy" onClick={copySetup}>{copied ? "Copied" : "Copy skill"} &nbsp;↗</button></div>
      </header>
      <div className="card-body">
        <div className="brand-lockup"><img src="/brain-logo.png" alt="Cogmax" /><span>cogmax</span></div>
        <h1 id="page-title">Give your agents a memory.</h1>
        <p className="description">Cogmax stores the decisions, preferences and project context your agents should remember — and brings them back when they need it.</p>
        <div className="instruction-row"><p>{setupMessage}</p><button className="copy-button" onClick={copySetup}>{copied ? "Copied" : "Copy"} <span>{copied ? "✓" : "↗"}</span></button></div>
        <div className="tool-row"><span className="tool-label">MCP tools</span><code>learn</code><code>recall</code><span className="separator">·</span><span className="tool-label">Secure by default</span></div>
      </div>
    </section>
    <footer><span>cogmax vault</span><span>Memory infrastructure for agents</span></footer>
  </main>;
}
