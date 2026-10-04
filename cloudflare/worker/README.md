# Cogmax Cloudflare adapter

Este componente é opcional. O Cogmax local continua sendo a fonte de verdade
offline; o Worker só recebe eventos depois que o cliente local os confirmou.

- `GET /health` é público.
- `POST /v1/events` exige `Authorization: Bearer <COGMAX_API_TOKEN>`.
- o Durable Object é particionado pelo escopo-base do usuário;
- `event_id` é a chave idempotente: reenvios não duplicam eventos;
- não há escrita local bloqueada por falha remota;
- o Worker não deve receber segredos ou snapshots sem validação.
