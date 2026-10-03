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

Exemplo de chamada autenticada:

```text
Authorization: Bearer $COGMAX_API_TOKEN
```

Em produção, o token deve ser emitido e armazenado pelo ambiente de execução,
nunca versionado no repositório ou colocado na Skill distribuída.
