# AGENTS.md

## Product identity

Product name: **Civico**

Repository name:

```text
civico
```

Use `Civico` in user-facing product names, documentation, page titles, and administrative UI.

Use `civico` as the technical prefix where a component-specific name is required, for example:

```text
civico-api
civico-cms
civico-player
```

Do not introduce alternative product names without an explicit requirement.

## Project

This repository contains an interactive touchscreen kiosk platform for a public information totem.

The physical totem runs Rocky Linux with a graphical session and is connected to the Internet through Wi-Fi.

All runtime components run on the SAME physical machine:

- CMS frontend
- REST API
- database
- media storage
- kiosk player
- local service/agent

Remote administration is performed over a private overlay network such as Tailscale, ZeroTier, or equivalent.

The public touchscreen must never expose the administration interface or the operating system.

---

## Primary goals

1. Provide a reliable touchscreen-first public information kiosk.
2. Show a configurable HOME screen with large buttons.
3. Allow navigation to content such as:
   - events
   - news
   - places
   - services
   - useful contacts
   - galleries
   - generic pages
4. Provide a local CMS for managing all content.
5. Permit remote CMS access only over a trusted private network.
6. Continue operating normally when Internet connectivity is unavailable.
7. Recover cleanly after reboot, network loss, browser crash, or service restart.

---

## Architecture constraint

Do NOT design this as a central cloud CMS plus remote player.

There is currently one all-in-one kiosk machine.

Architecture:

```text
Remote admin laptop
        |
  Tailscale / ZeroTier
        |
        v
+--------------------------------------+
|           Rocky Linux Totem          |
|                                      |
|  CMS Web  --->  REST API             |
|                    |                 |
|               PostgreSQL             |
|                    |                 |
|               Media Storage          |
|                                      |
|  Chromium Kiosk ---> Player Web      |
|                         |            |
|                   localhost API      |
|                                      |
+--------------------------------------+
        |
       Wi-Fi
```

Internet access is useful for remote administration but MUST NOT be required for normal kiosk usage.

---

## Preferred stack

Backend:
- Rust stable
- Axum
- Tokio
- SQLx
- Serde
- tracing
- tower-http

Database:
- PostgreSQL

CMS:
- React
- TypeScript
- Vite

Player:
- React
- TypeScript
- Vite

Deployment:
- Podman / Quadlet where appropriate
- systemd for startup and service supervision
- Chromium in kiosk mode

Avoid Electron unless there is a demonstrated need.

---

## Engineering principles

- Keep the public player and admin CMS logically separated.
- Bind public-local services to localhost when external access is unnecessary.
- Expose the CMS only on the private overlay network.
- Never expose PostgreSQL directly to Wi-Fi or the public Internet.
- Do not assume Wi-Fi or Internet availability.
- Prefer explicit configuration over hidden magic.
- Use migrations for all database schema changes.
- Use UUID primary keys.
- Use structured logging.
- Implement health endpoints.
- Add tests for core domain and API behavior.
- Do not hardcode HOME buttons in the player.
- Do not couple player navigation directly to database structure.
- Do not allow arbitrary external URLs from public content.
- Keep dependencies minimal.

---

## UX requirements

The player is used by anonymous members of the public.

Design for:

- portrait displays
- touchscreen interaction
- large touch targets
- high contrast
- no hover-only controls
- obvious HOME and BACK controls
- automatic return to HOME after inactivity
- no browser chrome
- no access to desktop shell
- no keyboard dependency

Default inactivity timeout: 90 seconds.

Every meaningful user interaction resets the timer.

---

## CMS requirements

CMS must manage:

- home buttons
- categories / sections
- pages
- events
- news
- places
- contacts
- media
- publication visibility
- ordering
- translations later

Do not overbuild a generic website builder in the MVP.

The first objective is a stable, usable kiosk CMS.

---

## Security

Assume the touchscreen is physically accessible to untrusted users.

- Public player must not reveal admin endpoints.
- Admin routes must require authentication.
- CMS should only be reachable via private overlay network.
- Use firewall rules to restrict listening interfaces.
- Never bind PostgreSQL to all interfaces.
- Do not store admin passwords in source code.
- Use environment variables or secrets files with restrictive permissions.
- Disable arbitrary browser navigation.
- Treat uploaded SVG and HTML as potentially dangerous.
- Validate upload MIME type, size, and extension.

---

## Offline behavior

The kiosk is local-first.

The database, API, CMS, player, and media are local.

Therefore:
- loss of Internet must not affect content navigation;
- loss of overlay VPN must only affect remote administration;
- loss of Wi-Fi must not break local kiosk operation.

If remote assets are ever introduced, they must be downloaded and cached locally before publication.

---

## Development behavior for coding agents

Before modifying code:

1. inspect repository structure;
2. inspect existing migrations;
3. inspect configuration;
4. inspect relevant tests;
5. preserve architectural constraints in this file.

For every meaningful change:

- update tests;
- update documentation when behavior changes;
- keep migrations reversible when practical;
- run formatting;
- run linting;
- run type checks;
- run backend tests;
- run frontend build/tests.

Never silently change architectural direction.

If requirements conflict with this file, flag the conflict clearly.

---

## Definition of done

A change is complete only when:

- code builds;
- tests pass;
- formatting passes;
- linting passes;
- DB migrations are valid;
- no secrets are committed;
- public kiosk behavior remains offline-safe;
- restart behavior remains functional;
- documentation reflects important changes.
