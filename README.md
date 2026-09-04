# Civico

**Civico** è una piattaforma per totem informativi interattivi: un unico software, installato su una sola macchina, che trasforma un monitor touchscreen in un punto informativo pubblico.

L'idea di fondo è semplice: il totem è un computer (in produzione, Rocky Linux) che fa tutto da solo. Il database, l'API, il pannello di amministrazione, il lettore pubblico e i file multimediali vivono tutti sulla stessa macchina e parlano tra loro in locale. Il pubblico tocca lo schermo, naviga contenuti e non deve mai accorgersi di cosa c'è "dietro".

## Perché è fatto così

I requisiti che hanno guidato il progetto:

- **Local-first.** Il contenuto informativo deve funzionare anche quando la connessione Internet salta. Se un comune perde la rete, il totem deve continuare a mostrare eventi, news e servizi senza scomporsi.
- **Amministrazione remota ma riservata.** Il pannello di gestione non si raggiunge da Internet: passa da una rete overlay privata (es. Tailscale o ZeroTier). La superficie touchscreen pubblica non deve mai esporre l'amministrazione.
- **Touchscreen-first.** Bottoni grandi, contrasto alto, nessuna funzione che richieda l'hover del mouse, ritorno automatico alla HOME dopo l'inattività, niente browser visibile.

In breve: una macchina sola, un totem affidabile, zero dipendenze dal cloud.

## Componenti

| Componente | Cosa fa |
| --- | --- |
| `crates/api` | API REST in Rust (Axum), contiene la logica di business, autenticazione, pubblicazione contenuti e upload media |
| `apps/cms-web` | Pannello di amministrazione in React/Vite: gestione contenuti, HOME, media, pubblicazioni |
| `apps/player-web` | Lettore pubblico in React/Vite: l'interfaccia touchscreen che vede l'utente |
| PostgreSQL | Database, unica fonte di verità per contenuti e configurazione |

## Stato attuale

Il progetto è in sviluppo. Al momento è solida la parte di backend:

- Migrazioni SQLx complete e reversibili (utenti admin, contenuti, media, bottoni HOME)
- API con autenticazione admin (password hashate con Argon2id, sessioni JWT)
- CRUD per contenuti, HOME e media
- Endpoint pubblici per il player con logica di "pubblicazione effettiva" (visibilità schedulata nel tempo)
- Upload media validati sul contenuto reale del file, non sulla dichiarazione del client (solo JPEG/PNG/WebP)

I frontend CMS e player sono impostati ma i loro flussi principali (editor HOME, gestionale contenuti, navigazione kiosk, timer di inattività) sono ancora da costruire. I dettagli sono nello stato dei lavori in `TASKS.md`.

## Come si avvia in sviluppo

Prima di tutto il database:

```bash
make dev-db
```

poi l'API:

```bash
make dev-api
```

e i due frontend, in terminali separati:

```bash
make dev-cms
make dev-player
```

Le variabili d'ambiente di sviluppo sono documentate in `.env.example`. In locale `make dev-api` legge `DATABASE_URL`, che di default punta al PostgreSQL avviato con Podman.

### Comandi utili

| Comando | Effetto |
| --- | --- |
| `make build` | Compila backend e builda i due frontend |
| `make test` | Esegue i test del workspace Rust |
| `make check` | Formattazione + lint + test, tutto in un colpo |
| `make fmt` | Formatta il codice (Rust e frontend) |

## Struttura del repository

```text
crates/api/        backend Rust (Axum, SQLx)
crates/api/migrations/   migrazioni SQL versionate e reversibili
apps/cms-web/      pannello di amministrazione
apps/player-web/   lettore pubblico touchscreen
compose.yaml       PostgreSQL di sviluppo (Podman)
```

## Documentazione

Per approfondire c'è qualcosa di più di un README:

- [`ARCHITECTURE.md`](ARCHITECTURE.md) — architettura, modello di rete, schema del database, API in dettaglio
- [`REQUIREMENTS.md`](REQUIREMENTS.md) — requisiti funzionali del sistema
- [`AGENTS.md`](AGENTS.md) — principi di progetto e vincoli architetturali (utile anche se collabori con un agente)
- [`TASKS.md`](TASKS.md) — lo stato dei lavori, fase per fase

## Una regola da ricordare

Internet non deve mai essere un prerequisito per mostrare un contenuto del kiosk. Se un vincolo o una modifica andasse in direzione opposta, è una violazione dei requisiti, non un'ottimizzazione.