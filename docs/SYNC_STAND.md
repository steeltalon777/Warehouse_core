# SyncServer Real Test Stand

## Prerequisites

- PostgreSQL 16+
- Python 3.11+ with `SyncServer/` dependencies
- Rust stable 1.85+

## Environment Variables

| Variable | Purpose |
|---|---|
| `SYNC_TEST_DATABASE_URL` | PostgreSQL DSN |
| `SYNC_TEST_BASE_URL` | SyncServer base URL |
| `SYNC_TEST_ROOT_USER_TOKEN` | Root user UUID token |
| `SYNC_TEST_CHIEF_USER_TOKEN` | Chief storekeeper token |
| `SYNC_TEST_STOREKEEPER_USER_TOKEN` | Storekeeper token |
| `SYNC_TEST_OBSERVER_USER_TOKEN` | Observer token |
| `SYNC_TEST_DEVICE_TOKEN` | Registered device token |
| `SYNC_TEST_DEVICE_ID` | Device UUID |
| `SYNC_TEST_SITE_ID_MAIN` | Primary site ID |
| `SYNC_TEST_SITE_ID_SECONDARY` | Secondary site ID |
| `WAREHOUSE_CORE_TEST_PROFILE_DIR` | Temp profile directory |
| `WAREHOUSE_CORE_CONTRACT_TESTS` | Set to enable real stand tests |

## Seed Data Requirements

| Entity | Count | Notes |
|---|---|---|
| Users | 4 | root, chief, storekeeper, observer |
| Sites | 2 | main + secondary |
| Devices | 1 | registered with device token |
| Units | 5+ | active units |
| Categories | 10+ | tree structure |
| Items | 20+ | with SKU and hashtags |
| Recipients | 3+ | |
| Balances | via operations | not direct DB |
| Operations | 5+ | all types |
| Temporary items | 1 | from receive flow |
| Document templates | 1+ | |

## Health Checks

```bash
curl http://localhost:8000/api/v1/health
curl http://localhost:8000/api/v1/ready
curl http://localhost:8000/api/openapi.json
cargo run -p warehouse_cli -- health-remote --server http://localhost:8000
```

## Stand Lifecycle

```bash
# Start
docker run -d --name sync-pg -e POSTGRES_PASSWORD=test -p 5432:5432 postgres:16
cd SyncServer && alembic upgrade head && uvicorn app.main:app --port 8000

# Smoke
cargo run -p warehouse_cli -- health-remote --server http://localhost:8000
cargo run -p warehouse_cli -- auth-context
cargo run -p warehouse_cli -- bootstrap --server http://localhost:8000
cargo run -p warehouse_cli -- sync-pull
cargo run -p warehouse_cli -- catalog search --query bolt
cargo run -p warehouse_cli -- balances list --site-id 1
cargo run -p warehouse_cli -- operations list --site-id 1

# Cleanup
docker stop sync-pg && docker rm sync-pg
rm -rf $WAREHOUSE_CORE_TEST_PROFILE_DIR
```

## Reset

```bash
docker exec sync-pg psql -U postgres -c "DROP DATABASE IF EXISTS warehouse_test"
docker exec sync-pg psql -U postgres -c "CREATE DATABASE warehouse_test"
cd SyncServer && alembic upgrade head
```
