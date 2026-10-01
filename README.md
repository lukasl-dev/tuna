# tuna

Rust browser automation using [thirtyfour](https://github.com/stevepryde/thirtyfour).
The example searches Wikipedia for Selenium and checks the resulting page title.

```sh
nix develop
chromedriver --port=9515
```

In another terminal:

```sh
nix develop
cargo run
```

The Linux development shell includes Chromium and ChromeDriver. On macOS,
install Chrome and ChromeDriver separately. The example connects to the local
ChromeDriver server on port 9515 instead of downloading a driver automatically.

Build the package with `nix build`, or run checks with `nix flake check`.
