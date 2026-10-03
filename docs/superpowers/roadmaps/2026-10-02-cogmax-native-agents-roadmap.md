# Cogmax Native-Agent Implementation Roadmap

Este roadmap reorganiza o plano técnico em fatias verticais. Cada fatia deve
ser implementada por um agente nativo separado, revisada antes da próxima e
deixar o sistema em estado executável.

## Estratégia de execução

- Um agente implementa uma fatia por vez.
- Um agente revisor verifica aderência à especificação e qualidade da fatia.
- O próximo agente recebe apenas o contrato e os artefatos aprovados da fatia
  anterior, não o histórico inteiro da conversa.
- Nenhum agente publica, faz push ou altera infraestrutura externa sem uma
  decisão explícita fora deste roadmap.
- O caminho é offline-first: o runtime local é a implementação de referência e
  deve funcionar sem rede, Cloudflare, Git remoto ou modelo externo.
- Cada adapter posterior precisa passar pelos mesmos testes de contrato do
  runtime local.

## Fatia 0 — Fundação do workspace

**Objetivo:** criar o esqueleto Rust do Cogmax e os contratos mínimos de domínio.

**Agente:** implementação mecânica, modelo rápido.

**Entrega:**

- workspace Cargo;
- `cogmax-domain`;
- tipos `Memory`, `MemoryCandidate`, `MemoryScope` e estados do ciclo de vida;
- serialização estável;
- testes de identidade, escopo e idempotência.

**Aceite:** `cargo test --workspace` passa e nenhum código conhece Cloudflare,
Git ou um provedor de modelo.

## Fatia 1 — Memória local funcional

**Objetivo:** provar o comportamento central do produto em um processo local.

**Agente:** implementação de domínio e persistência, modelo padrão.

**Entrega:**

- storage SQLite local;
- `recall`, `learn`, `consolidate` e `forget`;
- ranking lexical determinístico;
- versões, conflitos, validade e escopos;
- testes de isolamento e deduplicação.

**Aceite:** uma correção aprendida em uma sessão é recuperada em uma nova
sessão, sem duplicação e sem atravessar escopos.

## Fatia 2 — Skill-first end-to-end

**Objetivo:** tornar a experiência utilizável somente pela ativação da skill.

**Agente:** integração de produto, modelo padrão.

**Entrega:**

- `skill/SKILL.md`;
- comportamento silencioso de recall e learn;
- adapter HTTP/Axum interno;
- CLI apenas para desenvolvimento e diagnóstico;
- fluxo E2E local que simula duas sessões.

**Aceite:** o usuário precisa conhecer apenas a skill; endpoints, storage e
jobs não aparecem na instrução da skill.

## Fatia 3 — Pipeline assíncrono de aprendizado

**Objetivo:** separar observação, candidato e memória canônica.

**Agente:** pipeline e contratos de modelo, modelo padrão ou mais capaz se
forem necessárias decisões de schema.

**Entrega:**

- `ModelProvider` abstrato;
- schema validado de candidatos;
- extração e consolidação assíncronas;
- retries, idempotência e falhas terminais;
- fixtures de modelo sem dependência de fornecedor.

**Aceite:** mensagens duplicadas não criam memórias duplicadas; saída inválida
do modelo não contamina a memória ativa.

## Fatia 4 — Portabilidade local e Git offline

**Objetivo:** tornar a memória auditável, exportável e restaurável.

**Agente:** storage/exportação, modelo padrão.

**Entrega:**

- snapshot determinístico;
- Markdown legível;
- Parquet para histórico;
- manifesto com hashes e versão do schema;
- restore local;
- sincronização com um repositório Git local, agrupada e assíncrona;
- nenhum provider remoto nesta fatia.

**Aceite:** um snapshot pode ser comparado, restaurado e sincronizado depois de
uma indisponibilidade do Git sem interromper recall.

## Fatia 5 — Runtime local distribuível

**Objetivo:** tornar o Cogmax utilizável offline como um produto local completo.

**Agente:** runtime e release, modelo padrão.

**Entrega:**

- servidor local persistente;
- diretório de dados configurável;
- snapshots e restore offline;
- Git local opcional;
- comando de diagnóstico;
- smoke test sem acesso à rede.

**Aceite:** uma instalação limpa completa o fluxo inteiro sem variáveis de
Cloudflare, credenciais Git ou conexão externa.

## Fatia 6 — Cloudflare runtime

**Objetivo:** executar o mesmo domínio no ambiente distribuído escolhido.

**Agente:** integração Cloudflare, modelo mais capaz.

**Entrega:**

- Worker de entrada;
- Durable Object por escopo de agente;
- SQLite-backed storage no objeto;
- routing e autenticação;
- Queue consumer;
- R2 para eventos e snapshots;
- testes locais com Wrangler/workerd.

**Aceite:** recall e learn funcionam no Worker; falha de Queue, R2 ou Git não
derruba o caminho síncrono de recall.

## Fatia 7 — Distribuição da skill e release

**Objetivo:** empacotar o Cogmax como produto instalável, sem expor sua
infraestrutura.

**Agente:** release e experiência de instalação, modelo padrão.

**Entrega:**

- pacote da Cogmax Skill;
- instruções de instalação mínimas;
- modo local/offline;
- imagem/container ou artefato de execução;
- documentação de troubleshooting orientada ao usuário;
- smoke test de instalação e ativação.

**Aceite:** uma instalação limpa consegue ativar a skill e completar o fluxo
recall → learn → nova sessão → recall.

## Fatia 8 — Revisão final independente

**Agente:** modelo mais capaz, sem implementar funcionalidades novas.

**Escopo:**

- revisão da especificação contra o comportamento real;
- segurança de escopos, logs e dados sensíveis;
- idempotência e recuperação;
- diferença entre evidência local e deploy remoto;
- tamanho e clareza da skill;
- custos e limites Cloudflare;
- regressões entre modo local e edge.

**Saída:** relatório de achados classificados e, se necessário, uma única
rodada de correção seguida de re-review.

## Dependências entre fatias

```text
0 → 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8
```

Não paralelizar fatias que alteram os mesmos contratos. A única paralelização
permitida é preparar fixtures/documentação de uma fatia futura enquanto o
agente principal aguarda, sem modificar interfaces compartilhadas.

## Critério de parada

O roadmap termina quando a Fatia 7 estiver validada offline e a Fatia 8 não
tiver achados críticos. Deploy Cloudflare, sincronização com um provedor Git e
integração com um modelo real devem ser reportados como evidências separadas,
nunca inferidos dos testes locais.
