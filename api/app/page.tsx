"use client";
import { useState } from "react";
import "./styles.css";

const steps = [["01", "Escolha seu foco", "Defina o que você quer aprender ou melhorar."], ["02", "Faça um diagnóstico", "Um mapa rápido mostra por onde começar."], ["03", "Pratique no seu ritmo", "Pequenas sessões que cabem na sua rotina."], ["04", "Veja sua evolução", "Acompanhe o que mudou e ajuste o próximo passo."]];
const stories = [["Marina, 29", "Eu finalmente encontrei um jeito leve de estudar sem abandonar no meio."], ["Rafael, 34", "As sessões curtas transformaram meus intervalos em momentos de progresso."], ["Lia, 41", "É simples, bonito e dá vontade de voltar todos os dias."]];

export default function Home() {
  const [menuOpen, setMenuOpen] = useState(false);
  const closeMenu = () => setMenuOpen(false);
  return <main className="site-shell">
    <div className="floating-doodle doodle-a">✦</div><div className="floating-doodle doodle-b">◌</div><div className="floating-doodle doodle-c">⌁</div>
    <div className="page-frame">
      <header className="nav-bar"><a className="brand" href="#top" onClick={closeMenu}><img src="/brain-logo.png" alt="Cogmax" /><span>cogmax</span></a><button className="menu-toggle" onClick={() => setMenuOpen(!menuOpen)} aria-label="Abrir menu"><span /><span /></button><nav className={menuOpen ? "nav-links is-open" : "nav-links"}><a href="#como-funciona" onClick={closeMenu}>Como funciona</a><a href="#beneficios" onClick={closeMenu}>Benefícios</a><a href="#historias" onClick={closeMenu}>Histórias</a><a className="nav-login" href="/auth/login">Entrar</a><a className="button button-small" href="#comece" onClick={closeMenu}>Começar grátis</a></nav></header>
      <section className="hero" id="top"><div className="hero-copy"><p className="eyebrow">um espaço para sua mente</p><h1>Aprender pode ser<br /><em>mais leve.</em></h1><p className="hero-text">Construa clareza, curiosidade e confiança com experiências curtas feitas para a vida real.</p><a className="button" href="#comece">Começar agora <span>↗</span></a></div><div className="hero-art" aria-hidden="true"><div className="sun-ring" /><img src="/brain-logo.png" alt="" /><p className="art-note">ideias<br />em movimento</p><div className="scribble">✧</div></div></section>
      <section className="intro-band" id="beneficios"><div className="section-heading"><p className="eyebrow">por que cogmax?</p><h2>Um jeito mais humano<br />de <em>continuar curioso.</em></h2></div><div className="benefit-list"><article><strong>01</strong><div><h3>Aprenda em qualquer lugar</h3><p>Conteúdos rápidos para encaixar entre uma coisa e outra.</p></div></article><article><strong>02</strong><div><h3>Avance sem pressa</h3><p>Um ritmo possível, com espaço para pensar e tentar de novo.</p></div></article><article><strong>03</strong><div><h3>Encontre seu caminho</h3><p>Recomendações que acompanham os seus interesses de verdade.</p></div></article></div></section>
      <section className="process" id="como-funciona"><div className="section-heading process-heading"><p className="eyebrow">seu próximo passo</p><h2>Como a jornada<br /><em>acontece.</em></h2></div><div className="steps">{steps.map(([number, title, text]) => <article className="step" key={number}><span>{number}</span><div><h3>{title}</h3><p>{text}</p></div><b>↗</b></article>)}</div></section>
      <section className="stories" id="historias"><div className="stories-header"><div><p className="eyebrow">quem já começou</p><h2>Histórias<br /><em>de dentro.</em></h2></div><span className="quote-mark">“</span></div><div className="story-grid">{stories.map(([name, text], index) => <article className={`story story-${index + 1}`} key={name}><div className="portrait">{index === 0 ? "M" : index === 1 ? "R" : "L"}</div><p>“{text}”</p><strong>{name}</strong><small>membro Cogmax</small></article>)}</div></section>
      <section className="cta" id="comece"><img src="/brain-logo.png" alt="" /><div><p className="eyebrow">a primeira sessão é sua</p><h2>Comece com uma<br /><em>boa pergunta.</em></h2></div><a className="button button-light" href="/auth/login">Criar minha conta <span>↗</span></a></section>
      <footer><span>cogmax</span><p>Feito para mentes curiosas.</p><a href="#top">Voltar ao topo ↑</a></footer>
    </div>
  </main>;
}
