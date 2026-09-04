# Architecture

## Overview

This is an all-in-one interactive kiosk system running on one Rocky Linux machine.

The same host runs:

- PostgreSQL
- Rust API
- CMS frontend
- player frontend
- media files
- Chromium kiosk session

Remote administration uses a trusted private overlay network such as Tailscale or ZeroTier.

Wi-Fi is the physical network transport, but local kiosk operation must not depend on Wi-Fi availability.

---

## Runtime topology

```text
                     PRIVATE OVERLAY VPN
                  Tailscale / ZeroTier
                            |
                            v

                     Admin browser
                            |
                            v
              https://totem-private-name
                            |
+---------------------------------------------------+
|               ROCKY LINUX TOTEM                   |
|                                                   |
|  Admin CMS                                        |
|  React/Vite                                       |
|        |                                          |
|        v                                          |
|  Rust API / Axum                                  |
|        |                                          |
|        +------------------+                       |
|        |                  |                       |
|        v                  v                       |
|  PostgreSQL          Media storage                |
|                                                   |
|                                                   |
|  Chromium --kiosk                                 |
|        |                                          |
|        v                                          |
|  Player Web                                       |
|        |                                          |
|        v                                          |
|  localhost API                                    |
|                                                   |
+---------------------------------------------------+
```

---

## Network model

### Public touchscreen

The player should load from localhost, for example:

```text
http://127.0.0.1:8081
```

It must not rely on DNS, Internet, or the overlay VPN.

### Administration

The CMS may listen on:

- the Tailscale interface
- the ZeroTier interface
- or localhost behind a reverse proxy bound only to the overlay interface

Do NOT expose the admin CMS broadly on `0.0.0.0` unless firewall rules explicitly limit access.

### Database

PostgreSQL should listen only on localhost.

Example conceptual configuration:

```text
listen_addresses = '127.0.0.1'
```

---

## Recommended ports

These are defaults and may be changed.

```text
8080  API
8081  Player
8082  CMS
5432  PostgreSQL (localhost only)
```

A reverse proxy may expose CMS/API through one private HTTPS endpoint.

---

## Application boundaries

### cms-web

Purpose:
- manage content
- configure HOME
- manage media
- preview kiosk
- publish/unpublish

Never render public kiosk UI directly.

### player-web

Purpose:
- render public touchscreen UI
- navigate content
- enforce inactivity reset
- prevent navigation outside the kiosk application

Must be usable without Internet.

### api

Purpose:
- business logic
- persistence
- authentication
- content delivery
- media metadata
- CMS API
- local player API

### database

PostgreSQL is the source of truth for configuration and content.

Because all components are local, the player can query the API locally without requiring a synchronization agent in the MVP.

A dedicated agent can be introduced later for:
- health reporting
- automatic updates
- self-healing
- screenshots
- remote diagnostics

Do not add an agent until it provides concrete value.

### Database schema

Schema is managed exclusively through SQLx versioned migrations in `crates/api/migrations`.
Each migration has a matching `*.down.sql` so schema changes are reversible.

Tables:

- `admin_users` — administrative CMS accounts (username unique, password hash, active flag).
- `media` — uploaded image library (JPEG/PNG/WebP only; other MIME types are rejected by a
  CHECK constraint until SVG sanitization exists). Stores file metadata and a unique
  `storage_path`; CREATED_BY links to `admin_users`.
- `content` — a single table for all content types (pages, news, events, places, contacts,
  galleries). A `content_type` enum discriminator plus a `data` JSONB column for type-specific
  fields keeps the model extensible without a generic website-builder abstraction. Carries
  `publication_status` (draft/published/archived), optional `publish_from`/`publish_until`
  scheduling, a `published_at` timestamp, and references to a hero `image_id` and `created_by`.
- `home_items` — the configurable HOME buttons. Buttons are data, not hardcoded in the player.
  Each button's destination is discriminated by `destination_type`: it points either at a whole
  `section` (naming a `destination_section_type`, a content_type listing) or at a specific
  `content` row via `destination_content_id`. The two are mutually exclusive (CHECK constraints).

Enums enforced at the database layer: `content_type`, `publication_status`, `home_destination_type`.

Timestamps use `TIMESTAMPTZ`; primary keys are UUIDs (default `gen_random_uuid()`).
`publish_until` must be later than `publish_from` when both are set (CHECK constraint).

---

## API

The API is served by the Axum binary `crates/api` and binds to `API_BIND`
(default `127.0.0.1:8080`). All routes are versioned under `/api/v1`.

### Authentication

Admin backup ownership is seeded at runtime via a **bootstrap** step rather than
stored in source: the first `POST /api/v1/admin/auth/bootstrap` succeeds only while
zero admin users exist and returns a signed token. It returns `409` afterwards.

- `POST /api/v1/admin/auth/bootstrap` — create the first admin (public, one-time).
- `POST /api/v1/admin/auth/login` — authenticate and return a token.

Passwords are hashed with **Argon2id** (PHC string). Sessions are stateless
**HS256 JWTs** signed with `JWT_SECRET`, with a 12-hour TTL. There is no logout
endpoint by design (the client simply discards the token).

Every route under `/api/v1/admin` except the two auth routes above requires a
valid `Authorization: Bearer <token>` header and returns `401` otherwise.

### Admin routes (authenticated)

- `GET|POST /api/v1/admin/home-items` · `GET|PUT|DELETE /api/v1/admin/home-items/{id}`
- `GET|POST /api/v1/admin/content` · `GET|PUT|DELETE /api/v1/admin/content/{id}`
- `GET|POST /api/v1/admin/media` · `GET|DELETE /api/v1/admin/media/{id}`

### Player / public routes (unauthenticated)

- `GET /api/v1/player/home` — enabled HOME buttons ordered by position, with
  destinations resolved for the player (`section` → content_type, `content` → slug).
- `GET /api/v1/player/content` — effectively-published content, optional
  `?content_type=` filter (invalid values return a structured `422`).
- `GET /api/v1/player/content/{slug}` — effectively-published content by slug.
- `GET /api/v1/media/{id}/file` — public media streaming (used by the player).

### Effective publication

A content row is public only when `publication_status = 'published'` AND
(`publish_from` is null or in the past) AND (`publish_until` is null or in the
future). The player never sees draft or archived rows, nor rows outside their
scheduled window.

### Media upload

- Only JPEG, PNG and WebP are accepted (SVG stays disabled pending sanitization).
- MIME is validated by **content sniffing** (magic bytes), never by trusting the
  client-declared MIME type or the filename.
- Files are stored under `MEDIA_DIR` at a hash-derived path independent of the
  user-supplied filename, with directory-traversal protection.
- Uploads are bounded by `MAX_UPLOAD_BYTES`; empty bodies are rejected.

### Errors and tracing

Errors use a structured shape — `{"error": {"code", "message", "details"}}` — and
SQL/database internals are never leaked to clients. HTTP requests are traced with
`tower-http` and the API logs with `tracing`.

### Environment variables

| Variable | Default | Purpose |
| --- | --- | --- |
| `DATABASE_URL` | — | PostgreSQL connection string |
| `API_BIND` | `127.0.0.1:8080` | API listen address |
| `JWT_SECRET` | insecure dev default | key for signing admin session tokens |
| `MEDIA_DIR` | `media` | on-disk upload storage directory |
| `MAX_UPLOAD_BYTES` | `10485760` | maximum upload size in bytes |

---

## Boot flow

Target boot behavior:

```text
Power on
  ↓
Rocky Linux
  ↓
systemd services
  ├── PostgreSQL
  ├── API
  ├── CMS
  └── Player
  ↓
Graphical session
  ↓
Automatic kiosk user login
  ↓
Chromium --kiosk http://127.0.0.1:8081
```

If Chromium crashes, it should restart automatically.

If the API is temporarily unavailable, the player should display a controlled local error state and retry.

---

## Reliability

The totem must survive:

- Wi-Fi loss
- Internet loss
- VPN loss
- power loss
- API restart
- browser restart
- database restart

Use systemd restart policies.

Example:

```text
Restart=always
RestartSec=3
```

Do not create restart loops without rate limiting.

---

## Future evolution

Possible later additions:

- multiple kiosks
- central management server
- local content replication
- remote health dashboard
- automatic screenshots
- OS/app updates
- multiple languages
- weather integrations
- event feeds
- QR-code handoff
- accessibility modes

Do not prematurely design the MVP as a distributed fleet platform.
