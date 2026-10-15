# tuna

Start the foreground worker with the `TUNA_TISS_*`
credentials in its environment:

```sh
set -a; source .env; set +a
cargo run -- serve
```

Commands in other terminals reuse the worker's browser without needing credentials:

```sh
cargo run -- tiss login
cargo run -- tuwel login
cargo run -- tuwel notifications list
cargo run -- tuwel timeline
cargo run -- tuwel quizzes get --id 3061354
cargo run -- tuwel quizzes get --id 2984776
cargo run -- tiss messages
cargo run -- tiss programmes list
cargo run -- tiss courses get --semester 2026W --course 104340
cargo run -- tiss courses exams --semester 2026W --course 194024
cargo run -- tiss groups list --semester 2026W --course 104340
cargo run -- stop
```

The worker starts its own ChromeDriver on an automatically assigned local port.
ChromeDriver and Chromium must be on `PATH`; the Nix development shell and
Linux package provide both. The browser opens lazily and stays alive between
commands. The worker checks both TISS and TUWEL authentication before each
operation, reusing the shared TU Wien IdP session and logging in only when
necessary. `tiss login` and `tuwel login` both ensure that both sessions are ready.
The existing `TUNA_TISS_*` credentials are used for both services. Prefer
`TUNA_TISS_TOTP_URL` for automatic reauthentication; a fixed TOTP code expires.

TUWEL notifications include `read` and `read_at`, alongside their title, body,
context link, and creation time. Listing fetches the full list through Moodle's
read-only API without selecting notifications or marking them as read. Timestamps
are returned in UTC.

`tuwel timeline` returns the dashboard's All view, including all result pages,
using language-independent Moodle identifiers. It includes event/activity types,
course information, UTC event times, overdue status, and action availability.
The dashboard's local midnight and All filter's date range are respected;
saved filters and sorting preferences are not changed. Activity links and
action buttons are never followed.

`tuwel quizzes get --id ID` uses the course-module `id` from a quiz's `view.php`
URL. It returns the overview's title, course, description, dates, access
conditions, notices, result, and visible attempt summaries. Labels, dates, and
grades retain their displayed text; selectors do not depend on German/English
labels. Start, resume, review, and submission controls are never clicked, and
form tokens are not returned.

Opening a Moodle quiz overview is not strictly side-effect-free: Moodle logs
the view, may update completion tracking, and can finalise an already-expired
attempt. This command only reads the resulting overview, not quiz questions.

Group registration submits and confirms a real registration:

```sh
cargo run -- tiss groups register --semester 2026W --course 104340 --group "Group name"
```

Group names are matched case-insensitively with whitespace normalized. The
command returns the TISS confirmation message as JSON. Registration failures
are not retried automatically; check TISS before retrying an uncertain outcome.

Credentials, `--webdriver`, and `--headed` belong to `serve`. Use `--socket` or
`TUNA_SOCKET` to select a different worker. The socket directory must be owned
by you with permissions `0700`; the socket itself has permissions `0600`.
The default is `$XDG_RUNTIME_DIR/tuna/worker.sock`, falling back to the user
cache directory. `stop`, Ctrl-C, and SIGTERM close the browser and remove the socket.
They also stop the worker-owned ChromeDriver. Use `serve --webdriver URL` to
connect to an externally managed driver instead; that process is never stopped
by tuna.

