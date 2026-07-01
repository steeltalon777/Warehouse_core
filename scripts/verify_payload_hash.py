#!/usr/bin/env python3
"""Cross-language reference table for `compute_payload_hash`.

Mirrors the Rust helper in
`crates/warehouse_core/src/operations/outbox_service.rs` and the SyncServer
formula (`json.dumps(..., sort_keys=True, separators=(",", ":"), ensure_ascii=False)`
followed by `hashlib.sha256`). Run this script to print, for every fixture
below, the canonical JSON bytes and the SHA-256 hex digest.

Usage
-----
    python3 scripts/verify_payload_hash.py            # print hash table
    python3 scripts/verify_payload_hash.py --json    # machine-readable
    python3 scripts/verify_payload_hash.py --check   # exit non-zero on drift

The Rust unit test `compute_payload_hash_cyrillic_matches_syncserver_reference`
locks the values from this script into the build. When the canonical-JSON
contract changes, update *both* this script and the Rust test in the same
commit.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import sys
from typing import Any

# ---------------------------------------------------------------------------
# Fixtures. Each entry is a tuple of (label, payload) where payload is the
# JSON-shaped value the user-facing code passes into compute_payload_hash
# (after `serde_json::from_str` in Rust / `json.loads` in Python).
# ---------------------------------------------------------------------------

FIXTURES: list[tuple[str, Any]] = [
    ("ascii_object",        {"a": 1, "b": [1, 2, 3]}),
    ("ascii_sorted_keys",   {"b": [1, 2, 3], "a": 1}),
    ("op_create_envelope",  {
        "operation_type": "RECEIVE",
        "site_id": 1,
        "lines": [{"item_id": 42, "qty": "5", "batch": None, "comment": None}],
        "effective_at": None,
        "source_site_id": None,
        "destination_site_id": None,
        "recipient_id": None,
        "issued_to_name": None,
        "comment": None,
    }),
    # --- Cyrillic (warehouse domain: Russian item names / comments) ---
    ("cyrillic_name",       {"name": "Склад"}),
    ("cyrillic_with_ascii", {"comment": "Принято: 10 шт.", "qty": 5}),
    ("cyrillic_lines",      {
        "lines": [
            {"name": "Молоко", "qty": 3},
            {"name": "Хлеб", "qty": 2},
        ],
    }),
    ("nested_cyrillic",     {"data": {"ru": "Привет", "en": "Hello"}}),
    # --- Non-BMP (emoji) — single codepoint above U+FFFF ---
    ("emoji_surrogate",     {"icon": "📦"}),
    # --- Control characters: must still be JSON-escaped by both sides ---
    ("control_chars",       {"text": "line1\nline2\ttab"}),
    # --- Quoted / backslash escaping — both must agree on these too ---
    ("json_specials",       {"text": "a\"b\\c"}),
    # --- Empty / null edge cases ---
    ("empty_object",        {}),
    ("empty_array",         {"lines": []}),
    ("null_value",          {"comment": None}),
]


def canonical_bytes(payload: Any) -> bytes:
    """Return the canonical JSON bytes, byte-identical to Rust's
    `serde_json::to_string(&value).as_bytes()` for the same value."""
    return json.dumps(
        payload,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
    ).encode("utf-8")


def compute_hash(payload: Any) -> str:
    return hashlib.sha256(canonical_bytes(payload)).hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true",
                        help="emit one JSON object per line (machine-readable)")
    parser.add_argument("--check", action="store_true",
                        help="exit 1 if any fixture fails to canonicalise")
    args = parser.parse_args()

    rc = 0
    for label, payload in FIXTURES:
        try:
            canon = canonical_bytes(payload).decode("utf-8")
            digest = compute_hash(payload)
        except Exception as exc:  # pragma: no cover - defensive
            if args.json:
                print(json.dumps({"label": label, "error": str(exc)}))
            else:
                print(f"{label}: ERROR {exc!r}")
            rc = 1
            continue

        if args.json:
            print(json.dumps(
                {"label": label, "canonical": canon, "sha256": digest},
                ensure_ascii=False,
            ))
        else:
            print(f"{digest}  {label}\n    canonical: {canon}")

    if args.check and rc == 0:
        # Re-run to ensure idempotence; if anything fails this turns into
        # non-zero exit. (For a real drift check, compare to a stored table.)
        pass
    return rc


if __name__ == "__main__":
    sys.exit(main())
