# tuna

Start ChromeDriver:

```sh
chromedriver --port=9515
```

In another terminal, start the foreground worker with the `TUNA_TISS_*`
credentials in its environment:

```sh
set -a; source .env; set +a
cargo run -- serve
```

Commands in other terminals reuse the worker's browser without needing credentials:

```sh
cargo run -- tiss login
cargo run -- tiss messages
cargo run -- tiss programmes list
cargo run -- tiss courses get --semester 2026W --course 104340
cargo run -- tiss groups list --semester 2026W --course 104340
cargo run -- stop
```

The browser opens lazily and stays alive between commands. The worker checks
authentication before each operation and logs in only when necessary. Prefer
`TUNA_TISS_TOTP_URL` for automatic reauthentication; a fixed TOTP code expires.

Credentials, `--webdriver`, and `--headed` belong to `serve`. Use `--socket` or
`TUNA_SOCKET` to select a different worker. The socket directory must be owned
by you with permissions `0700`; the socket itself has permissions `0600`.
The default is `$XDG_RUNTIME_DIR/tuna/worker.sock`, falling back to the user
cache directory. `stop`, Ctrl-C, and SIGTERM close the browser and remove the socket.

