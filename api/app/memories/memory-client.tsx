"use client";

import { useState } from "react";

export default function MemoryClient() {
  const [content, setContent] = useState("");
  const [query, setQuery] = useState("");
  const [message, setMessage] = useState("");
  const [results, setResults] = useState<unknown[]>([]);

  async function learn() {
    const response = await fetch("/api/learn", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ eventId: crypto.randomUUID(), scope: "personal", kind: "Note", content }) });
    const data = await response.json();
    setMessage(response.ok ? "Memória salva." : data.error ?? "Falha ao salvar.");
    if (response.ok) setContent("");
  }

  async function recall() {
    const response = await fetch("/api/recall", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ scope: "personal", query }) });
    const data = await response.json();
    setResults(response.ok ? data.memories : []);
    setMessage(response.ok ? `${data.memories.length} memória(s) encontrada(s).` : data.error ?? "Falha ao consultar.");
  }

  return <section><label>Nova memória<br /><textarea value={content} onChange={(e) => setContent(e.target.value)} /></label><br /><button onClick={learn} disabled={!content.trim()}>Salvar memória</button><hr /><label>Consultar<br /><input value={query} onChange={(e) => setQuery(e.target.value)} /></label> <button onClick={recall} disabled={!query.trim()}>Consultar</button><p>{message}</p><pre>{JSON.stringify(results, null, 2)}</pre></section>;
}
