# Requirements

## Functional requirements

### FR-001 Home

The kiosk must open on a configurable HOME screen.

The HOME screen contains large touch-friendly buttons.

Each button has:

- UUID
- title
- optional subtitle
- icon
- optional image
- order
- enabled flag
- destination
- optional styling metadata

The HOME configuration must be editable from the CMS.

---

### FR-002 Navigation

The player must support:

- HOME
- BACK
- section navigation
- content detail pages

Navigation must remain entirely inside the kiosk application.

---

### FR-003 Automatic reset

After 90 seconds of inactivity, the kiosk must automatically return to HOME.

The timeout must be configurable.

Touch, click, scroll, and navigation events reset the timeout.

---

### FR-004 Content types

MVP must support:

- pages
- news
- events
- places
- contacts
- galleries

The model should permit new content types later without a complete rewrite.

---

### FR-005 Media

CMS must support image uploads.

Initial allowed formats:

- JPEG
- PNG
- WebP

SVG should remain disabled unless sanitization is implemented.

Video support may be implemented after the base MVP.

---

### FR-006 Publication status

Content must support at least:

- draft
- published
- archived

Optional scheduling fields:

- publish_from
- publish_until

---

### FR-007 CMS authentication

Administrative CMS requires authentication.

The player must not require authentication for local public content delivery.

Public and admin endpoints must remain separated.

---

### FR-008 Local-first operation

The kiosk must continue functioning without:

- Wi-Fi
- Internet
- Tailscale
- ZeroTier

All required content and media must be stored locally.

---

### FR-009 Remote administration

Administrators must be able to connect remotely through a private overlay VPN.

Supported deployment examples:

- Tailscale
- ZeroTier

The application must not depend on one specific VPN provider.

---

### FR-010 Startup

After power-on, the kiosk must automatically start the application and enter fullscreen kiosk mode without manual interaction.

---

## Non-functional requirements

### NFR-001 Security

- PostgreSQL local-only.
- CMS reachable only through trusted network path.
- No arbitrary web browsing from player.
- No secrets in repository.
- Validate uploads.
- Harden browser session.
- Separate kiosk OS user from administrative users where practical.

### NFR-002 Reliability

- systemd supervised services
- restart on failure
- structured logs
- graceful DB reconnect
- controlled player error handling

### NFR-003 Performance

The UI should feel immediate on local hardware.

Target:
- typical route transitions under 200 ms after initial load
- local API requests under 100 ms in normal conditions
- lazy-load heavy images
- generate optimized media variants later if necessary

### NFR-004 Maintainability

- Rust workspace
- explicit modules
- SQL migrations
- frontend TypeScript strict mode
- shared API types where useful
- clear README
- documented deployment

### NFR-005 Accessibility

- large touch targets
- readable typography
- high contrast
- semantic markup
- keyboard support where reasonable, despite touch-first design
