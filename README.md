# Cogmax

Memória persistente, local e orientada a Skills para agentes de IA.

O usuário ativa uma Skill. A Skill cuida do restante: runtime local, descoberta
de memórias, importação, recall, decisões e sincronização opcional. MCP, API,
SQLite, Git e Cloudflare são detalhes internos — não fazem parte do onboarding.

## O que o Cogmax faz

- Mantém memória local mesmo sem internet.
- Importa memórias de agentes como Codex, Claude, Kimi e fontes genéricas.
- Preserva usuário e projeto como parte do escopo da memória.
- Classifica decisões, projetos, procedimentos, correções e referências.
- Preserva o motivo conhecido de uma decisão sem inventar justificativas.
- Evita ativar candidatos de baixa confiança.
- Mantém decisões conflitantes visíveis até uma resolução explícita.
- Explica por que uma memória foi retornada.
- Cria snapshots Markdown, JSON e Parquet verificáveis.
- Permite restore local e sincronização Git opcional, sem `push` automático.
- Possui um adaptador Cloudflare/Durable Objects opcional para sincronização remota.

## Experiência do usuário

```text
baixar a Skill → adicionar ao agente → ativar → conversar normalmente
```

Na primeira ativação, a Skill encontra fontes locais e mostra um resumo natural.
O agente pede confirmação antes de importar em lote. O usuário não precisa abrir
um terminal, configurar MCP, criar banco ou conhecer a CLI.

## Instalação

Baixe a Skill correspondente à release em:

[github.com/orafam/cogmax/releases](https://github.com/orafam/cogmax/releases)

Depois, adicione o pacote ao seu agente e ative a Skill. A instalação deve
validar o `SHA256SUMS` da release antes de executar qualquer binário.

O fluxo oficial não exige Kubernetes, banco externo, kubectl, kubeconfig ou
infraestrutura remota.

## Desenvolvimento local

Requisitos:

- Rust estável;
- Node.js 22+ apenas para o adaptador Cloudflare;
- `jq` e `curl` para o smoke test da Skill.

Validar o núcleo Rust:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Validar o Worker local:

```bash
cd cloudflare/worker
npm ci
npm run typecheck
npx wrangler dev --local --var COGMAX_API_TOKEN:test-token
```

O Worker local expõe `/health` publicamente e protege eventos com Bearer token.
Eventos repetidos pelo mesmo `event_id` são idempotentes.

## Componentes

```text
Skill
  └── runtime Rust local
        ├── discovery de agentes
        ├── memória e recall
        ├── SQLite offline-first
        └── export/restore/sync opcionais

Worker Cloudflare
  └── Durable Object por usuário
```

O SQLite local continua sendo a fonte de verdade do modo offline. Uma falha de
sincronização remota não impede recall nem apaga memória local.

## Segurança e privacidade

- Segredos, tokens e chaves privadas não são importados.
- A API autenticada exige `COGMAX_API_TOKEN`.
- `COGMAX_API_USER` restringe acesso ao próprio usuário e seus projetos.
- `/health` é público; operações de memória são protegidas quando a API remota
  está habilitada.
- Snapshots alterados são rejeitados pelo manifesto SHA-256.
- Git sync recusa repositórios sujos e snapshots conflitantes.

Mais detalhes: [`docs/security.md`](docs/security.md).

## Estado do projeto

O núcleo offline-first, onboarding da Skill, importação, recall explicável,
supersessão, snapshots, Git sync e adaptador Cloudflare local estão implementados
e cobertos por testes. O deploy Cloudflare de produção e uma identidade remota
OIDC ainda são etapas posteriores.

Consulte:

- [`docs/architecture.md`](docs/architecture.md)
- [`docs/compatibility.md`](docs/compatibility.md)
- [`docs/superpowers/plans/2026-10-05-cogmax-today/00-overview.md`](docs/superpowers/plans/2026-10-05-cogmax-today/00-overview.md)

## Licença

Apache-2.0.
