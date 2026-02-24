# tuna

Fast, script-friendly TISS automation for TU Wien.

`tuna` handles the boring browser choreography for you: login, list groups, register for a course, and register for a specific group.

## What it does

- Logs in to TISS (username/password, optional TOTP).
- Lists course groups as JSON.
- Registers in a course (including confirmation step).
- Registers in a specific group by group name (including confirmation step).
- Supports scheduled start (`--postpone`) and automatic retries (`--retries`).

## Build

### Nix (recommended)

Build as flake package:

```bash
nix build .#tuna
```

Run it:

```bash
./result/bin/tuna --help
```

The flake package wraps `tuna` with `chromium` in `PATH`, so runtime browser dependency is included.

### Plain Go

```bash
go build -o tuna ./cmd/tuna
./tuna --help
```

## Usage

Top-level help:

```bash
tuna --help
```

### 1) List groups

```bash
tuna list-groups \
  --username "$TISS_USER" \
  --password "$TISS_PASS" \
  --totp "$TISS_TOTP_URL" \
  --semester 2026S \
  --course 064013
```

Output: JSON array on stdout.

### 2) Register in a course

```bash
tuna register-course \
  --username "$TISS_USER" \
  --password "$TISS_PASS" \
  --totp "$TISS_TOTP_URL" \
  --semester 2026S \
  --course 192026
```

### 3) Register in a group

```bash
tuna register-group \
  --username "$TISS_USER" \
  --password "$TISS_PASS" \
  --totp "$TISS_TOTP_URL" \
  --semester 2026S \
  --course 064013 \
  --group "Observers"
```

Output: selected group as JSON on stdout.

## Time-critical flags

### `--postpone`

Schedule command start for a timestamp. Format:

```text
YYYY-MM-DD HH:MM:SS+TZ
```

Example:

```bash
tuna --postpone "2026-02-24 02:16:00+01:00" register-group ...
```

Execution starts **5 seconds after** the given timestamp, never before.

### `--retries`

Retry the command on failure:

```bash
tuna --retries 3 register-group ...
```

This means up to 4 total attempts (1 initial + 3 retries).

## Logging and output contract

- Logs go to stderr (`slog` text handler).
- Machine-readable command results go to stdout.
- Non-zero exit code on failure (including login failures such as incorrect credentials).

## Practical note

Use `--headless=false` when debugging browser behavior.
