# Segurança da API local

O serviço local pode operar sem autenticação quando executado apenas no
localhost. Para qualquer exposição além do processo local, configure
`COGMAX_API_TOKEN` antes de iniciar o serviço.

Quando essa variável existe:

- `/health` continua público para health checks;
- `/learn` e `/recall` exigem `Authorization: Bearer <token>`;
- token ausente ou inválido retorna `401 Unauthorized`;
- o token nunca é incluído em logs, respostas ou memórias;
- a autorização não amplia o escopo solicitado: usuário e projeto continuam
  determinados pelo request e pelas regras de isolamento do serviço.

Para ativar o limite de identidade, configure também `COGMAX_API_USER`. O
usuário autenticado só pode acessar `user:<COGMAX_API_USER>` e seus projetos;
tentativas de acessar outro usuário retornam `403 Forbidden`. Projetos com
barra ou escopo inválido são rejeitados.

As rotas protegidas usam uma janela de 60 segundos. O limite padrão é 120
requisições e pode ser ajustado com `COGMAX_RATE_LIMIT`. Ao exceder o limite,
a API retorna `429 Too Many Requests`. A rota pública de health check não
consome essa cota.

O endpoint público de aprendizado aceita somente memórias com autoridade
`Explicit`; memórias inferidas devem entrar pelo importador local controlado.

As operações autenticadas `POST /supersede` e `POST /revoke` também exigem o
Bearer token e verificam o escopo do usuário antes de alterar uma memória.

Exemplo de chamada autenticada:

```text
Authorization: Bearer $COGMAX_API_TOKEN
```

Em produção, o token deve ser emitido e armazenado pelo ambiente de execução,
nunca versionado no repositório ou colocado na Skill distribuída.
