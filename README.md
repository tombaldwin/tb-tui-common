# tb-tui-common

Shared TUI helpers reused across [`ebman`](https://github.com/tombaldwin/ebman) and [`pgman`](https://github.com/tombaldwin/pgman) — k9s-style terminal UIs for AWS Elastic Beanstalk and PostgreSQL respectively.

## What's in it

- **`font_probe`** — Powerline-glyph terminal probe (CSI device-status interrogation; returns whether the active font advances `U+E0B0` to a single cell).
- **`overlay`** — `OverlaySize` enum + `centered_overlay(category, frame)` helper so every modal overlay (text dump, picker, action confirm, palette) sizes from one table.
- **`theme`** — `IconStyle` enum + `contrast_text_for(bg)` WCAG-aware foreground picker so themed pills stay readable against arbitrary backgrounds.
- **`splash`** — pixel-art `render_frame(frame, palette)` loop for ASCII-grid splash animations driven by a per-character palette closure.
- **`util`** — `parse_bool(&str)` (loose form: `"yes"` / `"on"` / `"1"` etc.) and `write_atomic(path, content)` (writes via a sibling temp file + rename so a crash mid-write doesn't corrupt the existing file).

## Effect boundary (CI-enforced)

This is a *shared* library, so it must stay reusable: it never reaches an **app-coupling** effect — no
network, database, subprocess, or IPC; only presentation, plus filesystem/clock in the `util` helpers.
CI enforces that with [candor](https://github.com/tombaldwin/candor)'s stable scanner
(`cargo install candor-scan`, no nightly needed) via `ci/candor-check.sh` against
[`.candor/policy`](.candor/policy) — a PR that lands a `reqwest`/database/subprocess call here fails the
`candor (effect boundary)` job. (Terminal I/O via `crossterm` isn't modelled as an effect, so the gate
covers app-coupling I/O, not the TTY.)

## Stability

Pre-1.0. The API mostly stabilised when ebman 0.8.0 / pgman 0.0.x shipped against it; expect non-breaking additions and occasional renames if a consumer surfaces a friction point. Pin a specific minor version if you care about that.

## License

MIT OR Apache-2.0.
