# Implementation Tasks

## Phase 1 — Repository bootstrap

- [ ] Create Rust workspace.
- [ ] Create `apps/cms-web`.
- [ ] Create `apps/player-web`.
- [ ] Create `crates/api`.
- [ ] Add `.env.example`.
- [ ] Add compose/Podman development environment.
- [ ] Add formatting and linting configuration.
- [ ] Add CI-style local commands.

Acceptance criteria:
- backend compiles
- both frontends build
- PostgreSQL starts
- API health endpoint responds

---

## Phase 2 — Database and domain

- [x] Add SQLx migrations.
- [x] Create admin users.
- [x] Create home_items.
- [x] Create content.
- [x] Create media.
- [x] Add timestamps and UUIDs.
- [x] Add publication state.

Acceptance criteria:
- clean DB can migrate from zero
- migrations tested against PostgreSQL
- repository/domain tests pass

---

## Phase 3 — API

- [x] `/api/v1/health`
- [x] admin authentication
- [x] HOME CRUD
- [x] content CRUD
- [x] media upload
- [x] player HOME endpoint
- [x] player content endpoint
- [x] structured errors
- [x] tracing

Acceptance criteria:
- admin routes authenticated
- player routes local/public
- invalid uploads rejected
- integration tests pass

---

## Phase 4 — CMS

- [ ] Login.
- [ ] Dashboard.
- [ ] HOME editor.
- [ ] Content list.
- [ ] Content editor.
- [ ] Media library.
- [ ] Ordering.
- [ ] Publish/archive.
- [ ] Kiosk preview.

Acceptance criteria:
- admin can create HOME buttons
- admin can create/edit content
- player reflects published changes

---

## Phase 5 — Player

- [ ] Portrait-first HOME.
- [ ] Large buttons.
- [ ] Internal router.
- [ ] Section listings.
- [ ] Content detail.
- [ ] HOME button.
- [ ] BACK button.
- [ ] inactivity timer.
- [ ] automatic HOME reset.
- [ ] no external navigation.
- [ ] controlled offline/local API error state.

Acceptance criteria:
- usable entirely via touch
- no hover dependency
- returns HOME after timeout
- navigation cannot escape application

---

## Phase 6 — Rocky Linux deployment

- [ ] Create kiosk OS user.
- [ ] Configure automatic graphical login.
- [ ] Configure Chromium kiosk launch.
- [ ] Disable sleep/screensaver.
- [ ] Add systemd services.
- [ ] Add Podman/Quadlet units if used.
- [ ] Configure local firewall.
- [ ] Restrict PostgreSQL to localhost.
- [ ] Document Tailscale setup.
- [ ] Document ZeroTier alternative.
- [ ] Verify reboot recovery.

Acceptance criteria:
- cold boot reaches HOME unattended
- unplugging Wi-Fi does not break player
- restoring Wi-Fi restores remote administration
- killing Chromium causes recovery
- restarting API causes recovery
