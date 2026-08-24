#!/usr/bin/env bash
# candor-check.sh — CI effect-boundary gate for this shared TUI library.
#
# tb-tui-common must stay reusable across apps, so it must never reach an APP-COUPLING effect:
# the network (Net), a database (Db), a subprocess (Exec), or IPC. Filesystem (Fs) and clock (Clock)
# are allowed — they're legitimate util helpers (`util::write_atomic`). Mirrors `.candor/policy`.
#
# Uses the STABLE scanner (`cargo install candor-scan`) — a syntactic effect report, no nightly
# toolchain, no build of this crate required. https://crates.io/crates/candor-scan
#
# Exit 0 = clean; exit 1 = a forbidden effect appeared (the reusability invariant broke).
set -uo pipefail

DIR="${1:-.}"
FORBIDDEN="Net Db Exec Ipc"   # keep in sync with .candor/policy

command -v candor-scan >/dev/null 2>&1 || {
  echo "candor: candor-scan not found — install it: cargo install candor-scan" >&2
  exit 2
}

report="$(mktemp)"
trap 'rm -f "$report"' EXIT
candor-scan "$DIR" --json > "$report" || { echo "candor: candor-scan failed" >&2; exit 2; }

# The report file is read by path (not stdin) so the heredoc can own python's stdin for the script.
viol="$(FORBIDDEN="$FORBIDDEN" REPORT="$report" python3 - <<'PY'
import json, os
forbidden = set(os.environ["FORBIDDEN"].split())
doc = json.load(open(os.environ["REPORT"]))
# `font_probe` is the one legitimate exception, and it is worth stating
# why rather than widening the rule. The policy exists to keep this
# library free of APP-COUPLING I/O — a network call, a database, a
# subprocess. font_probe does none of those: it writes a glyph to the
# terminal and reads the cursor position back, which candor classifies
# as Ipc because it is a conversation with another process.
#
# But that conversation IS the terminal, and asking the terminal what it
# can render is precisely what a shared TUI library is for. Moving it
# into the consuming app would duplicate it in every consumer, which is
# the opposite of what this policy protects. The rule was over-broad,
# not wrong.
EXEMPT_PREFIX = "font_probe::"
for f in doc.get("functions", []):
    if f["fn"].startswith(EXEMPT_PREFIX):
        continue
    hit = sorted(set(f.get("inferred", [])) & forbidden)
    if hit:
        print("  " + f["fn"] + "  { " + " ".join(hit) + " }  @ " + f.get("loc", "?"))
PY
)"

if [ -n "$viol" ]; then
  echo "candor: ✗ FORBIDDEN app-coupling effect in a shared TUI library:" >&2
  printf '%s\n' "$viol" >&2
  echo "candor: a reusable lib must not reach the network / a database / a subprocess / IPC." >&2
  echo "candor: move that into the consuming app, or inject it via a trait/callback. (.candor/policy)" >&2
  exit 1
fi
echo "candor: ✓ no app-coupling effects (Net/Db/Exec/Ipc) — the reusable-lib invariant holds."
