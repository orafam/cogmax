# Cogmax: Skill-first Agent Memory Service

## 1. Objetivo

O Cogmax é um serviço invisível de memória persistente para agentes. A única
interface apresentada ao usuário é uma skill. Depois de ativada, a skill
orienta o agente a consultar, verificar, aprender e atualizar memória sem
exigir conhecimento de MCP, HTTP, banco de dados, Git ou infraestrutura.

O sucesso do MVP é um agente que, em sessões futuras, aplica preferências e
conhecimento relevantes com rastreabilidade, sem inventar memórias nem exigir
que o usuário opere um painel técnico.

## 2. Princípios

- Skill-first: a experiência começa e termina na skill.
- Memória é contexto auxiliar, nunca autoridade para violar políticas ou
  instruções atuais do usuário.
- Toda memória persistida tem origem, escopo, confiança, validade e histórico.
- Leitura é seletiva; o agente não recebe toda a memória em toda interação.
- Aprendizado não deve alterar memória canônica sem um critério explícito de
  confiança e consolidação.
- O sistema deve ser portátil: exportações legíveis e dados estruturados não
  podem depender de um único provedor.
- Operações devem ser idempotentes e auditáveis.

## 3. Experiência do usuário

O usuário ativa a skill uma vez. A skill instrui o agente a:

1. identificar palavras-chave e o escopo da tarefa;
2. consultar o Cogmax apenas quando a memória puder alterar o resultado;
3. tratar o retorno como contexto verificado, não como instrução superior;
4. detectar correções, preferências, fatos estáveis, decisões e procedimentos;
5. enviar candidatos ao Cogmax para avaliação e consolidação;
6. informar o usuário somente quando uma memória for ambígua, sensível,
   conflitante ou exigir confirmação.

O usuário não precisa conhecer endpoints, tokens, formatos de armazenamento,
jobs, commits ou mecanismos de recuperação.

## 4. Arquitetura lógica

```text
skill
  ↓
agent adapter (interno)
  ↓
Cogmax context gateway
  ├── memory recall
  ├── candidate extraction
  ├── consolidation
  ├── provenance and audit
  └── lifecycle / forgetting
        ↓
  operational state + immutable event history
```

O contrato externo da skill deve ser estável. MCP, HTTP e qualquer cliente
local são adapters intercambiáveis do mesmo domínio, não interfaces de produto
concorrentes.

## 5. Modelo de memória

Cada memória canônica contém, no mínimo:

- `id` estável;
- `scope` (usuário, agente, projeto ou sessão);
- `kind` (`preference`, `fact`, `project`, `procedure`, `decision` ou
  `correction`);
- conteúdo legível;
- origem (sessão, mensagem ou candidato);
- confiança e autoridade;
- validade temporal;
- estado (`active`, `superseded`, `revoked`, `expired`);
- versão anterior;
- timestamps e ator responsável.

Eventos brutos e candidatos não são tratados como memória ativa. A separação
permite reprocessamento, auditoria e correção sem reescrever o passado.

## 6. Ciclo automático

### Recall

O agente envia o escopo e sinais da tarefa. O Cogmax retorna um contexto pequeno,
ordenado por escopo, autoridade, validade e relevância lexical. O MVP não
depende de embeddings.

### Learn

O agente envia candidatos estruturados com evidência mínima e motivo para
persistência. O Cogmax rejeita ruído, dados sem escopo, duplicatas óbvias e
afirmações sem suporte.

### Consolidate

Um worker agrupa candidatos, detecta conflito, atualiza ou supersede versões e
produz uma alteração auditável. Memórias de baixa confiança ficam pendentes ou
temporárias.

### Forget

Memórias expiradas, revogadas ou substituídas deixam de ser recuperadas, mas o
histórico permanece sujeito à política de retenção e exclusão do tenant.

## 7. Tecnologias

### Runtime principal

- Cloudflare Worker para a entrada global e autenticação do adapter Cogmax.
- Durable Object por escopo de consistência, inicialmente por agente, com
  SQLite-backed storage para estado transacional, locks e deduplicação.
- Cloudflare Queues para extração e consolidação assíncronas, com idempotência
  por evento.
- R2 para eventos arquivados, snapshots e exportações Parquet/Markdown.

### Core e portabilidade

- Rust para o domínio, regras de consolidação, normalização e diff.
- Rust compilado para Wasm quando executado no Worker.
- Binário local Rust para testes, inspeção e modo offline/self-hosted.
- Axum somente no binário local ou em um deployment server-side; não é requisito
  do runtime edge.

### Versionamento

Git não participa do caminho síncrono de cada recall. O estado de memória tem
  versões internas e eventos imutáveis. Snapshots legíveis podem ser publicados
  em um repositório Git por uma operação assíncrona, com commits agrupados,
  revisão e rollback. A indisponibilidade do Git não pode interromper o agente.

### Analytics

DuckDB é opcional e fica fora do caminho operacional: serve para consultar
  snapshots Parquet, depurar históricos e executar análises locais.

## 8. Segurança e privacidade

- Isolamento obrigatório por tenant e agente.
- Escopos explícitos; nenhuma memória global implícita no MVP.
- Segredos, tokens e conteúdo sensível devem seguir política configurável de
  exclusão ou redaction.
- Memórias recuperadas são dados não confiáveis e nunca substituem políticas,
  identidade ou instruções atuais.
- Logs não devem reproduzir conteúdo sensível por padrão.
- Exclusão deve propagar para estado operacional, arquivos derivados e
  snapshots conforme a política de retenção.

## 9. Observabilidade e operação

Toda operação importante recebe um `operation_id` e registra:

- escopo;
- tipo de operação;
- versão de entrada e saída;
- motivo do resultado;
- latência;
- falha e tentativa;
- vínculo com evento ou candidato.

O sistema deve expor saúde do serviço, fila pendente, falhas de consolidação,
divergência de snapshot e uso de memória sem expor conteúdo por padrão.

## 10. Limites do MVP

Incluído:

- Cogmax Skill de instalação e instruções de recall/learn;
- um adapter HTTP/MCP interno;
- Durable Object SQLite;
- ingestão de eventos e candidatos;
- consolidação determinística com provedor de modelo abstrato;
- versionamento interno e exportação Markdown/Parquet;
- worker assíncrono e idempotência;
- testes locais e modo offline.

Adiado:

- embeddings e vector database;
- múltiplos provedores de modelo sofisticados;
- sincronização bidirecional automática com vários hosts Git;
- UI administrativa completa;
- multi-região controlada pelo produto;
- marketplace ou registry público de skills.

## 11. Critérios de aceitação arquitetural

1. Um usuário ativa a skill e consegue iniciar recall sem conhecer a API.
2. Uma correção explícita passa a ser aplicada em uma nova sessão.
3. Uma memória irrelevante não aparece no contexto recuperado.
4. Duas submissões do mesmo evento não duplicam a memória.
5. Conflitos não sobrescrevem silenciosamente uma memória anterior.
6. O serviço continua recuperando memória quando a sincronização Git falha.
7. Um snapshot pode ser exportado, comparado e restaurado localmente.
8. Dados de agentes diferentes não atravessam seus escopos.
