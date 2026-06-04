#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
MIGRATIONS_DIR="$PROJECT_DIR/migrations/sqlite"
EXPECTED_SCHEMA="$SCRIPT_DIR/expected-schema.sql"

# Preflight check: Python with sqlite3 stdlib module
if ! python3 -c "import sqlite3" 2>/dev/null; then
    echo "ERROR: Python 3 with sqlite3 module is required."
    echo "  Install:  apt install python3 (Debian/Ubuntu)"
    echo "           brew install python3 (macOS)"
    exit 1
fi

# Collect migrations in sorted order
shopt -s nullglob
MIG_FILES=($(ls "$MIGRATIONS_DIR"/*.sql 2>/dev/null | sort))
if [ ${#MIG_FILES[@]} -eq 0 ]; then
    echo "ERROR: No migration files found in $MIGRATIONS_DIR"
    exit 1
fi

TEMP_DB=$(mktemp /tmp/warehouse-schema-XXXXXX.db)
TEMP_DUMP=$(mktemp /tmp/warehouse-schema-dump-XXXXXX.sql)
trap 'rm -f "$TEMP_DB" "$TEMP_DUMP"' EXIT

echo "=== Applying migrations to temp DB ==="
for f in "${MIG_FILES[@]}"; do
    echo "  Applying: $(basename "$f")"
done

# Build a Python-compatible list of migration paths
PY_MIG_LIST=""
for f in "${MIG_FILES[@]}"; do
    PY_MIG_LIST="${PY_MIG_LIST}${PY_MIG_LIST:+, }\"$f\""
done

python3 << PYEOF > /dev/null
import sqlite3

db_path = "$TEMP_DB"
migrations = [$PY_MIG_LIST]

conn = sqlite3.connect(db_path)
conn.execute("PRAGMA foreign_keys = OFF;")
try:
    for m in migrations:
        with open(m) as f:
            conn.executescript(f.read())
finally:
    conn.execute("PRAGMA foreign_keys = ON;")
conn.close()
PYEOF

echo "=== Dumping schema ==="
# Dump current schema from the temp DB
python3 << PYEOF > "$TEMP_DUMP"
import sqlite3

db_path = "$TEMP_DB"

conn = sqlite3.connect(db_path)

cur = conn.execute(
    "SELECT sql FROM sqlite_master WHERE sql IS NOT NULL AND name != 'sqlite_sequence' ORDER BY type DESC, name"
)
for row in cur.fetchall():
    stmt = row[0].strip()
    if not stmt.endswith(';'):
        stmt += ';'
    print(stmt)

conn.close()
PYEOF

echo "=== Schema dumped to $TEMP_DUMP ==="

if [ ! -f "$EXPECTED_SCHEMA" ]; then
    echo "No expected schema file found. Creating: $EXPECTED_SCHEMA"
    cp "$TEMP_DUMP" "$EXPECTED_SCHEMA"
    echo "Schema snapshot saved. Commit this file to version control."
    exit 0
fi

echo "=== Comparing schemas ==="
if diff -u "$EXPECTED_SCHEMA" "$TEMP_DUMP"; then
    echo "Schema is consistent with migrations."
    exit 0
else
    echo "SCHEMA DRIFT DETECTED!"
    echo "If the change is intentional, update the expected schema:"
    echo "  cp $TEMP_DUMP $EXPECTED_SCHEMA"
    exit 1
fi
