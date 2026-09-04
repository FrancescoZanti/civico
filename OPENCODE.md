# OpenCode Instructions

Use `AGENTS.md` as the primary project contract.

Read, in order:

1. `AGENTS.md`
2. `ARCHITECTURE.md`
3. `REQUIREMENTS.md`
4. `TASKS.md`

Then inspect the repository before making changes.

## Initial objective

Implement Phase 1 only unless explicitly instructed to continue.

Do not create a distributed multi-kiosk architecture.

The CMS, API, database, media storage, and player all run on the same Rocky Linux machine.

Remote administration uses Tailscale, ZeroTier, or an equivalent private overlay VPN.

Wi-Fi is not considered reliable and must not be required for local kiosk operation.

## Coding expectations

Backend:
- Rust stable
- Axum
- Tokio
- SQLx
- PostgreSQL
- Serde
- tracing

Frontend:
- React
- TypeScript
- Vite

Deployment:
- Rocky Linux
- Podman where useful
- systemd
- Chromium kiosk mode

## Work style

For each implementation increment:

1. state what will change;
2. implement the smallest coherent increment;
3. add or update tests;
4. run relevant checks;
5. report what passed and what remains.

Do not claim success without running the checks.

Do not leave placeholder code pretending to implement required behavior.

Prefer simple production-quality code over abstractions with no current use.
