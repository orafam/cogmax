# Compatibilidade e release gate

Cada alteração deve passar por três gates:

- Rust: `cargo fmt --all -- --check`, `cargo test --workspace` e `cargo clippy --workspace --all-targets -- -D warnings`;
- Worker: `npm ci && npm run typecheck` em `cloudflare/worker`;
- Skill: instalação/execução fora do checkout, com onboarding, confirmação e segunda execução idempotente.

Releases publicados precisam conter os quatro binários suportados, `SHA256SUMS`
e o pacote `cogmax-skill-<versão>.tar.gz`. A atualização deve preservar o
diretório de dados e não executar rebuild destrutivo automaticamente.
