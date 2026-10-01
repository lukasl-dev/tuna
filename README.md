# tuna

Rust browser automation using [thirtyfour](https://github.com/stevepryde/thirtyfour).
The CLI searches Wikipedia for the supplied terms and prints the resulting page title.

```sh
nix develop
chromedriver --port=9515
```

In another terminal:

```sh
nix develop
cargo run -- selenium
cargo run -- "Rust programming language"
cargo run -- --help
```

The Linux development shell includes Chromium and ChromeDriver. On macOS,
install Chrome and ChromeDriver separately. The example connects to the local
ChromeDriver server on port 9515 instead of downloading a driver automatically.
Use `--webdriver <URL>` to connect to a different WebDriver server:

```sh
cargo run -- selenium --webdriver http://localhost:4444
```

Build the package with `nix build`, or run checks with `nix flake check`.
