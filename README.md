# Lexicon

Entiendo que el repositorio del juego se llama **Lexicon**. Si finalmente mantienes `Lexinexo` como nombre público, reemplaza el nombre en los textos.

## Descripción corta para GitHub

> Daily semantic word game powered by LXDB.

Alternativa algo más descriptiva:

> A daily word game about semantic proximity, hidden concepts and unexpected connections.

Topics:

```text
word-game
daily-game
semantic-game
rust
vue
nuxt
webgpu
lxdb
spanish
```

---

## README completo para Lexicon

````md
# Lexicon

**Lexicon is a daily semantic word game powered by LXDB.**

Find hidden concepts, follow semantic connections and discover how words relate to each other.

Lexicon is the game.  
[LXDB](https://github.com/an0mal1a/lxdb) is the semantic subsystem behind it.

---

## The idea

Every day, Lexicon presents a new semantic challenge.

Players submit words and receive feedback based on their real semantic proximity to the hidden target.

A typical game looks like this:

```text
cold        12%
winter      41%
snow        68%
ice         84%
glacier     96%
````

The goal is not merely to guess a spelling or definition.

The goal is to navigate meaning.

---

## Core principles

Lexicon should be:

* easy to understand;
* difficult to master;
* deterministic;
* playable without generated or random scores;
* language-aware;
* visually distinctive;
* rewarding to revisit every day.

The frontend never decides whether a word is semantically close.

The authoritative result comes from the Rust game engine and its LXDB dataset.

---

## Architecture

Lexicon is built as an application on top of LXDB:

```text
linguistic sources
    ↓
LXDB dictionary generator
    ↓
Spanish semantic dataset
    ↓
LXDB storage and query engine
    ↓
Lexicon game engine
    ↓
Rust API
    ↓
web application
```

The dependency direction is intentional:

```text
Lexicon → LXDB
```

LXDB has no dependency on Lexicon.

---

## Repository structure

```text
lexicon/
├── crates/
│   ├── lexicon-core/
│   ├── lexicon-game/
│   └── lexicon-api/
├── apps/
│   └── web/
├── config/
│   └── challenges/
├── docs/
├── scripts/
├── Cargo.toml
└── README.md
```

### Components

| Component           | Responsibility                                               |
| ------------------- | ------------------------------------------------------------ |
| `lexicon-core`      | Game-domain models and shared types                          |
| `lexicon-game`      | Challenge selection, attempts, hints, scoring and game rules |
| `lexicon-api`       | HTTP API, persistence and LXDB integration                   |
| `apps/web`          | Player-facing web experience                                 |
| `config/challenges` | Curated or generated challenge configuration                 |

---

## LXDB integration

Lexicon uses LXDB for:

* word validation;
* lexical normalization;
* semantic proximity;
* direct relationships;
* bounded semantic paths;
* challenge candidate selection.

Lexicon does not expose LXDB internals to players.

Players never see:

* token IDs;
* relation IDs;
* binary offsets;
* dataset sections;
* compiler metadata;
* graph implementation details.

The game consumes application-facing results such as:

```json
{
  "word": "vehículo",
  "score": 0.82,
  "distance": 2,
  "is_exact": false
}
```

---

## Game modes

### Daily challenge

One shared challenge per language and calendar day.

Features:

* hidden target;
* unlimited or configurable attempts;
* semantic proximity feedback;
* hints;
* streaks;
* statistics;
* shareable result.

### Infinite mode

Play generated challenges without affecting the daily streak.

### Semantic path

Find a valid chain between two concepts.

```text
ocean → water → energy → electricity
```

### Discovery

Explore interesting semantic neighborhoods in a playful, non-technical interface.

---

## Daily challenge flow

```text
open Lexicon
    ↓
request today's public challenge
    ↓
start or restore anonymous session
    ↓
submit a word
    ↓
API normalizes and validates it
    ↓
game engine queries LXDB
    ↓
semantic score is calculated
    ↓
attempt is persisted
    ↓
frontend displays the result
    ↓
continue or win
```

The hidden target is never sent to the browser before the game is won or abandoned.

---

## Semantic scoring

The game engine calculates a deterministic score.

The first implementation may combine:

1. exact equality;
2. direct LXDB relation weight;
3. reverse relation with a configurable penalty;
4. best bounded semantic path;
5. path decay;
6. lexical and dataset metadata where available.

Conceptually:

```text
exact match       → 1.00
strong neighbor   → 0.80–0.99
short graph path  → 0.40–0.79
distant concept   → 0.01–0.39
unknown word      → no score
```

The final algorithm and parameters are documented in:

```text
docs/scoring.md
```

Scores must not be:

* random;
* generated in the browser;
* manually hardcoded per challenge;
* silently changed without versioning.

---

## Challenge selection

Daily challenges are selected by the backend.

A valid target should:

* exist in the active LXDB dataset;
* be marked playable;
* have sufficient semantic connectivity;
* not be excessively obscure;
* not be an invalid proper name or malformed entry;
* have enough nearby words to create a useful gradient;
* avoid recently repeated targets.

Selection must be deterministic for a date, language and challenge-set version.

The browser's local clock is not authoritative.

---

## API

Development base URL:

```text
http://localhost:3001/api
```

Expected endpoints:

```text
GET  /api/health
GET  /api/game/daily
POST /api/game/daily/start
POST /api/game/:game_id/attempt
POST /api/game/:game_id/hint
POST /api/game/:game_id/abandon
GET  /api/game/:game_id
GET  /api/stats
GET  /api/modes
```

### Submit an attempt

```http
POST /api/game/{game_id}/attempt
Content-Type: application/json
```

```json
{
  "word": "vehículo"
}
```

Example response:

```json
{
  "attempt": {
    "word": "vehículo",
    "score": 0.82,
    "distance": 2,
    "rank": 146,
    "is_exact": false
  },
  "status": "in_progress",
  "attempt_count": 7
}
```

The target word must not be present in an in-progress response.

---

## Development setup

### Requirements

* Rust stable
* Node.js
* the package manager declared by `apps/web`
* a compatible Spanish LXDB dataset

### Clone

```bash
git clone https://github.com/an0mal1a/lexicon.git
cd lexicon
```

### Configure the API

Create an environment file from the example:

```bash
cp .env.example .env
```

Example:

```env
LEXICON_DATASET=../lxdb/datasets/generated/es/dictionary.lxdb
LEXICON_DATABASE_URL=sqlite://data/lexicon.db
LEXICON_BIND_ADDRESS=127.0.0.1:3001
LEXICON_ALLOWED_ORIGIN=http://localhost:3000
RUST_LOG=info
```

### Start the API

```bash
cargo run -p lexicon-api
```

### Start the frontend

```bash
cd apps/web
npm install
npm run dev
```

Use the package manager lockfile present in the repository when it differs from npm.

---

## Dataset requirement

Lexicon does not generate production dictionaries itself.

It consumes datasets generated by LXDB.

Build the Spanish dataset from the LXDB repository:

```bash
cargo run --manifest-path ../lxdb/Cargo.toml -p lxdb-cli -- dictionary build es \
  --profile development \
  --source-fixture ../lxdb/crates/lxdb-dictionary/tests/fixtures \
  --output datasets/generated/es
```

Then configure:

```env
LEXICON_DATASET=/absolute/path/to/dictionary.lxdb
```

The exact Spanish generation process is documented in LXDB:

```text
LXDB/docs/dictionaries/spanish.md
```

Lexicon only documents the game-specific dataset requirements in:

```text
docs/dataset-requirements.md
```

---

## Game-specific dataset requirements

A dataset compatible with Lexicon must provide:

* normalized Spanish words;
* stable token lookup;
* semantic relations or proximity neighbors;
* bounded path traversal;
* sufficient graph connectivity;
* source and build metadata;
* a declared scoring profile;
* a way to distinguish valid words from playable targets.

Not every accepted word needs to be eligible as a daily target.

For example:

```text
accepted word:
    recognized as a valid player attempt

playable target:
    valid, common enough and sufficiently connected
```

This separation allows the game to recognize a broad vocabulary without choosing unusable or obscure targets.

---

## Anonymous sessions

The MVP can use anonymous sessions persisted with a secure cookie.

The backend stores:

* game session;
* challenge;
* attempts;
* hints used;
* status;
* score;
* streak;
* statistics.

The client must not be trusted to provide authoritative scores, targets or completion states.

---

## Frontend

The web interface is a game, not a technical dataset inspector.

Main navigation:

```text
Play
Modes
Statistics
```

The primary screen focuses on:

* today's challenge;
* word input;
* attempt history;
* semantic proximity;
* hints;
* streak;
* win state;
* sharing.

LXDB implementation details remain invisible.

---

## Visual direction

Lexicon uses a dark, immersive and modern visual language.

WebGPU or WebGL may be used for:

* background connections;
* semantic particles;
* subtle motion;
* transitions;
* celebration effects.

GPU rendering is decorative and interactive, but not authoritative game logic.

A visual fallback must exist when WebGPU is unavailable.

---

## Sharing

A result can be shared without revealing the answer:

```text
Lexicon #42
🟩🟩🟨🟧
7 attempts · 🔥 5
```

The target word must never appear in the shared text.

---

## Testing

### Rust

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
```

### Frontend

```bash
cd apps/web
npm run lint
npm run typecheck
npm run test
npm run build
```

### End-to-end

The core E2E flow is:

```text
load daily challenge
→ start session
→ submit valid word
→ receive real LXDB-backed score
→ submit more attempts
→ discover target
→ persist win
→ display result
```

---

## Documentation

```text
docs/
├── architecture.md
├── gameplay.md
├── scoring.md
├── daily-challenges.md
├── dataset-requirements.md
├── API.md
├── persistence.md
└── frontend.md
```

Dataset generation and binary-format documentation belong to the LXDB repository.

Game behavior and dataset consumption requirements belong here.

---

## Security

The API must:

* keep targets server-side;
* validate session ownership;
* limit input size;
* normalize Unicode;
* reject malformed requests;
* rate-limit attempts;
* avoid exposing internal errors;
* load and validate the LXDB dataset at startup.

Please disclose security issues privately.

---

## Relationship with LXDB

Lexicon is LXDB's first complete application.

The responsibilities are intentionally separated:

```text
LXDB
├── vocabulary
├── relationships
├── semantic data
├── binary storage
└── queries

Lexicon
├── daily challenge
├── attempts
├── scoring
├── hints
├── streaks
├── persistence
└── player experience
```

---

## Roadmap

### MVP

* Spanish daily challenge;
* anonymous sessions;
* real LXDB-backed attempts;
* hints;
* streaks;
* statistics;
* shareable results;
* responsive web experience.

### Next

* infinite mode;
* semantic path mode;
* richer statistics;
* accounts and sync;
* multiple languages;
* achievements;
* challenge archive.

---

## License

Add the chosen project license here.

LXDB is licensed separately.

The game does not redistribute raw linguistic sources unless explicitly stated. Production dataset attribution is distributed with the corresponding LXDB dataset.
