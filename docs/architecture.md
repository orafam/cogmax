# Arquitetura remota opcional

O Cogmax local permanece offline-first e usa SQLite como fonte de verdade. A
integração Cloudflare é um adaptador opcional para sincronização autenticada.

O Worker valida o bearer token e roteia cada evento para um Durable Object
particionado por usuário. O `event_id` é idempotente: reenvios retornam sucesso
sem criar uma segunda observação. Escopos de projeto continuam dentro do
escopo-base do usuário.

A ordem de reconciliação é: confirmar a escrita local, enviar o evento remoto,
marcar o envio como sincronizado e tentar novamente em caso de falha. Recall
local nunca depende de disponibilidade do Worker. Conflitos não são mesclados
silenciosamente; permanecem como eventos distintos até uma resolução explícita.
