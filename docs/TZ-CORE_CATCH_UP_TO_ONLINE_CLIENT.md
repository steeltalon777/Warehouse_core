# TZ: Warehouse_client_core — Догон онлайн-клиента по функционалу и стабилизация

## Execution Strategy

- [x] 🟢 Parallel execution recommended (Stage 1, Stage 3-B)
- [ ] 🟡 Sequential execution required (Stage 0, Stage 2, Stage 3-A)
- **Reason:** Stage 0 (stand gate) — последовательный, ошибки каскадны. Stage 1 (функциональный паритет) — 5 независимых юнитов. Stage 2 (Levels 7–12) — последователен по природе (outbox зависит от draft, engine от outbox). Stage 3 (архитектурные) — параллелен для 3-B, но 3-A требует Stage 1.

## Execution Checklist

- [ ] 0. Context verified — прочитаны все исходные ТЗ и smoke-report
- [ ] 1. Stage 0-A: Stand gate — verify fixes A–K, close residual bugs
- [ ] 2. Stage 0-B: Stand gate — full green smoke on clean profile
- [ ] 3. Stage 1-A: Issued Repository — domain DTOs + HTTP client + facade
- [ ] 4. Stage 1-B: DELETE operation endpoint
- [ ] 5. Stage 1-C: Catalog audit fields (created_by / updated_by)
- [ ] 6. Stage 1-D: Bulk catalog endpoints + document endpoint gaps
- [ ] 7. Stage 1-E: Document PDF rendering contract verification
- [ ] 8. Stage 2-A: OperationDraftService — verify + fix on real stand
- [ ] 9. Stage 2-B: Outbox + push — integration test on real stand
- [ ] 10. Stage 2-C: SyncEngine — correct failure reporting + hardening
- [ ] 11. Stage 2-D: Full FFI for write operations
- [ ] 12. Stage 2-E: Contract tests, performance, regression hardening (Level 11)
- [ ] 13. Stage 2-F: Documentation, packaging, handoff (Level 12)
- [ ] 14. Stage 3-A: API_MAP.md regeneration
- [ ] 15. Stage 3-B: CI schema-diff + ID type audit (parallel unit)
- [ ] 16. Unit/component tests: ≥50 tests, all pass
- [ ] 17. Stand smoke tests: full green on clean profile
- [ ] 18. Final acceptance review complete

## Check Rules

- Architect creates this checklist and acceptance criteria.
- Executor agents may check boxes only after implementation AND verification evidence.
- QA verifier may check final acceptance only after evidence review.
- Failed or unavailable checks stay unchecked with a blocker note.

---

## 0. Purpose

Закрыть разрыв между `warehouse_client_core` и онлайн-клиентом (SyncServer + Django BFF + Angular SPA) по функционалу и качеству интеграции. Состояние на 2026-06-04:

- **Unit/component**: 40 тестов проходят.
- **Реальный стенд**: 11 багов контракта/персистенции, gate НЕ пройден.
- **Пропущенный функционал**: Issued Repository (выданные объекты), DELETE operation, catalog audit fields, bulk catalog, document PDF pipeline.
- **Уровни TZ_CORE_CLIENT_READY_COMPLETION**: 0–3 сделаны, 4–5 заблокированы стендом, 7–12 не завершены.

**Источники данных для этого ТЗ**:

- `TZ_CORE_CLIENT_READY_COMPLETION.md` — roadmap с уровнями 0–12
- `CORE_STAND_SMOKE_REPORT.md` — инвентаризация 11 багов
- `TZ_CORE_STAND_FIXES_ORCHESTRATOR.md` — план исправлений A–K (закодированы, не верифицированы)
- `TZ-ISSUED_REPOSITORY_BACKEND_CONTRACT.md` — новый домен выданных объектов
- `TZ-B_OPERATIONS_DELETE_CONTRACT.md` — DELETE для отменённых операций
- `TZ-CATALOG_CREATED_BY_UPDATED_BY.md` — аудит-поля справочников
- `TZ-DOCUMENT_PDF_RENDERING_AND_UI.md` — рендеринг документов
- `TZ_NOMENCLATURE_BATCH_CATALOG_CRUD.md` — массовые операции каталога
- `CLIENT_READY_API_MATRIX.md` — классификация эндпоинтов
- `CORE_FACADE_V1.md` — 90+ методов фасада

---

## Stage 0: Stand Gate — Закрыть 11 багов и получить зелёный smoke

### 🟡 Sequential — ошибки каскадны, порядок важен

### 0-A. Verify fixes A–K, close residual bugs

**Исходное состояние**: исправления A–K из `TZ_CORE_STAND_FIXES_ORCHESTRATOR.md` закодированы в коде, но стенд-прогон 2026-05-20 показал остаточные провалы (`catalog_items` FK, `sync-pull` serialization/page-size, `operations list`, `bootstrap`).

**Файлы в зоне изменений (read-only аудит → правки)**:

| Файл | Что проверять |
|---|---|
| `crates/warehouse_core/src/storage/mod.rs` | Fix A: `Database::open()` создаёт файл БД при отсутствии |
| `crates/warehouse_core/src/operations/draft_service.rs` | Fix B: `SqliteDraftRepo::save()` — 11 плейсхолдеров, 4 bind |
| `crates/warehouse_core/src/storage/snapshot_writer.rs` | Fix C: `write_sites()` — `updated_at` NOT NULL |
| `crates/warehouse_core/src/sync/bootstrap.rs` | Fix D: порядок bootstrap: categories → units → items |
| `crates/warehouse_core/src/domain/operation.rs` | Fix E: operation/asset ID: `i64` → `String`/`Uuid` |
| `crates/warehouse_core/src/domain/sync_types.rs` | Fix F: `device_id`: `Uuid` → `i32` |
| `crates/warehouse_core/src/domain/balance.rs` | Fix G: `site_code` → `site_name` (API реально возвращает `site_name`) |
| `crates/warehouse_core/src/syncserver/client.rs` | Fix H: page_size capped to server limits |
| `crates/warehouse_core/src/syncserver/documents.rs` | Fix I: `documents_list` response shape |
| `crates/warehouse_core/src/sync/pull.rs` | Fix J: family names preserved (не `"unknown"`) |
| `crates/warehouse_core/src/sync/engine.rs` | Fix K: engine returns failure on required-family errors |

**Acceptance criteria**:
- Каждый Fix A–K подтверждён либо чтением кода (уже исправлен), либо минимальной правкой.
- `cargo fmt --all -- --check` и `cargo clippy --workspace --all-targets -- -D warnings` — чисто.
- `cargo test --workspace` — все тесты проходят.

### 0-B. Full green stand smoke on clean profile

**Сценарий (выполнить на чистом профиле)**:

```bash
# 1. Удалить старый профиль
rm -f /tmp/warehouse_smoke_test.db

# 2. Статические проверки
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# 3. Инициализация
cargo run -p warehouse_cli -- db init --db /tmp/warehouse_smoke_test.db

# 4. Health
cargo run -p warehouse_cli -- health-remote
cargo run -p warehouse_cli -- health-local --db /tmp/warehouse_smoke_test.db

# 5. Auth + site
cargo run -p warehouse_cli -- auth-context --db /tmp/warehouse_smoke_test.db
cargo run -p warehouse_cli -- site set 1 --db /tmp/warehouse_smoke_test.db

# 6. Bootstrap — БЕЗ ОШИБОК family
cargo run -p warehouse_cli -- bootstrap --db /tmp/warehouse_smoke_test.db

# 7. Sync pull — БЕЗ ОШИБОК на required families
cargo run -p warehouse_cli -- sync-pull --db /tmp/warehouse_smoke_test.db

# 8. Read smoke (локально после pull)
cargo run -p warehouse_cli -- catalog search test --db /tmp/warehouse_smoke_test.db
cargo run -p warehouse_cli -- balances list 1 --db /tmp/warehouse_smoke_test.db
cargo run -p warehouse_cli -- operations list 1 --db /tmp/warehouse_smoke_test.db
cargo run -p warehouse_cli -- temp-items list --db /tmp/warehouse_smoke_test.db
cargo run -p warehouse_cli -- assets pending --db /tmp/warehouse_smoke_test.db
cargo run -p warehouse_cli -- assets lost --db /tmp/warehouse_smoke_test.db
cargo run -p warehouse_cli -- assets issued --db /tmp/warehouse_smoke_test.db

# 9. Draft + outbox smoke
cargo run -p warehouse_cli -- draft create RECEIVE --site-id 1 --db /tmp/warehouse_smoke_test.db
cargo run -p warehouse_cli -- draft list --db /tmp/warehouse_smoke_test.db
cargo run -p warehouse_cli -- outbox list --db /tmp/warehouse_smoke_test.db
```

**Acceptance criteria**:
- Все команды выше выполняются без ошибок.
- `error_log` таблица пуста после полного цикла.
- `bootstrap` и `sync-pull` не содержат семейств с ошибками.
- Engine возвращает SUCCESS только при реальном успехе всех required families.

**Verification**:
- Real stand: `http://localhost:8000` (SyncServer) + `http://localhost:8001` (Django).
- Переменные окружения: `WAREHOUSE_USER_TOKEN`, `SYNC_USER_TOKEN`, `WAREHOUSE_DEVICE_TOKEN`, `SYNC_DEVICE_TOKEN`.

---

## Stage 1: Functional Parity — Догнать онлайн-клиент по функционалу

### 🟢 Parallel execution recommended — 5 независимых юнитов

### Stage 1-A: Issued Repository (Выданные объекты) 🆕

**Владелец**: `crates/warehouse_core/src/domain/` + `crates/warehouse_core/src/syncserver/` + `crates/warehouse_core/src/facade/`

**Контекст**: SyncServer реализовал полноценный каталог объектов выдачи (`/api/v1/issue-objects`, `/api/v1/issue-object-categories`) с CRUD, merge, деревом категорий и связью с `issued_assets`. В ядре — **ноль**.

**Новые файлы**:
- `crates/warehouse_core/src/domain/issue_objects.rs` — DTO: `IssueObjectResponse`, `IssueObjectCreate`, `IssueObjectUpdate`, `IssueObjectMerge`, `IssueObjectListResponse`, `IssueObjectCategoryResponse`, `IssueObjectCategoryCreate`, `IssueObjectCategoryUpdate`, `IssueObjectCategoryListResponse`, `TreeResponse`
- `crates/warehouse_core/src/syncserver/issue_objects.rs` — HTTP-клиент: `create`, `get`, `update`, `delete`, `list`, `merge`, `list_assets`, `get_tree`, `create_category`, `list_categories`, `get_category`, `update_category`, `delete_category`

**Изменяемые файлы**:
- `crates/warehouse_core/src/domain/mod.rs` — добавить `pub mod issue_objects;`
- `crates/warehouse_core/src/syncserver/mod.rs` — добавить `mod issue_objects;`
- `crates/warehouse_core/src/facade/mod.rs` — добавить методы: `list_issue_objects`, `search_issue_objects`, `get_issue_object`, `create_issue_object`, `update_issue_object`, `merge_issue_objects`, `list_issue_object_categories`, `create_issue_object_category`, `update_issue_object_category`, `get_issue_object_tree`
- `crates/warehouse_cli/src/main.rs` — добавить CLI-команды: `issue-objects list|search|get|create|update|merge`, `issue-object-categories list|tree`
- `docs/contracts/CLIENT_READY_API_MATRIX.md` — добавить группу «Issue Objects API (12 endpoints) — REQUIRED»

**DTO mapping (SyncServer → Rust)**:

| Pydantic schema | Rust DTO | Поля |
|---|---|---|
| `IssueObjectResponse` | `IssueObjectDto` | id, display_name, object_type, code?, normalized_key, comment?, category_id?, is_active, merged_into_id?, created_at, updated_at, deleted_at?, deleted_by_user_id? |
| `IssueObjectCreate` | `IssueObjectCreate` | display_name, object_type, code?, comment?, category_id |
| `IssueObjectUpdate` | `IssueObjectUpdate` | Все поля Optional |
| `IssueObjectMerge` | `IssueObjectMerge` | source_id, target_id |
| `IssueObjectCategoryResponse` | `IssueObjectCategoryDto` | id, name, normalized_key, parent_id?, sort_order, is_active, created_at, updated_at, deleted_at?, deleted_by_user_id? |
| `IssueObjectCategoryCreate` | `IssueObjectCategoryCreate` | name, parent_id?, sort_order, is_active |
| `IssueObjectCategoryUpdate` | `IssueObjectCategoryUpdate` | Все поля Optional |
| `TreeResponse` | `IssueObjectTreeDto` | id, type, name, comment?, category_id?, children |

**Required tests**:
- DTO serde roundtrip: `IssueObjectResponse` из JSON (unit)
- HTTP-клиент: mock server test для каждого метода (component)
- Facade: интеграционный тест с real stand (stand smoke)

**Acceptance criteria**:
- CLI `issue-objects list` возвращает список объектов выдачи с реального стенда.
- CLI `issue-objects merge --source 1 --target 2` выполняет merge.
- CLI `issue-object-categories tree` показывает дерево категорий.
- API-матрица обновлена: группа «Issue Objects» — REQUIRED.

### Stage 1-B: DELETE operation endpoint

**Владелец**: `crates/warehouse_core/src/syncserver/operations.rs` + `crates/warehouse_core/src/facade/mod.rs`

**Контекст**: SyncServer `DELETE /api/v1/operations/{id}` (TZ-B_OPERATIONS_DELETE_CONTRACT.md) позволяет удалять отменённые операции. В ядре HTTP-клиент и facade не имеют этого метода.

**Изменяемые файлы**:
- `crates/warehouse_core/src/syncserver/operations.rs` — добавить метод `operations_delete(&self, id: &str) -> CoreResult<()>`
- `crates/warehouse_core/src/facade/mod.rs` — добавить `delete_operation(id: &str) -> CoreResult<()>`
- `crates/warehouse_cli/src/main.rs` — команда `operations delete <id>`
- `docs/contracts/CLIENT_READY_API_MATRIX.md` — добавить `DELETE /operations/{id}` в группу Operations → REQUIRED

**Acceptance criteria**:
- Создать операцию, отменить, удалить через CLI — без ошибок.
- `cargo test --workspace` — тесты проходят.

### Stage 1-C: Catalog audit fields (created_by / updated_by)

**Владелец**: `crates/warehouse_core/src/domain/catalog.rs` + `crates/warehouse_core/src/storage/snapshot_writer.rs` + `crates/warehouse_core/src/storage/repos.rs`

**Контекст**: SyncServer добавил `created_by_user_id`, `updated_by_user_id` в Item, Category, Unit (TZ-CATALOG_CREATED_BY_UPDATED_BY). Ядро имеет эти поля только в operation DTO, но не в catalog DTO.

**Изменяемые файлы**:
- `crates/warehouse_core/src/domain/catalog.rs` — добавить поля в `ItemDto`, `CategoryDto`, `UnitDto`:
  - `created_by_user_id: Option<String>` (UUID from server)
  - `updated_by_user_id: Option<String>`
  - `created_by_user_name: Option<String>`
  - `updated_by_user_name: Option<String>`
- `crates/warehouse_core/src/storage/repos.rs` — обновить SQL-запросы `insert_item`, `insert_category`, `insert_unit`
- `crates/warehouse_core/src/storage/snapshot_writer.rs` — обновить `write_items`, `write_categories`, `write_units`
- `crates/warehouse_core/src/storage/migrations.rs` — миграция для добавления колонок в SQLite, если отсутствуют

**Acceptance criteria**:
- После `sync-pull` поля `created_by_user_name` / `updated_by_user_name` доступны в локальном SQLite.
- `catalog search` CLI показывает audit-информацию (или facade DTO содержит эти поля).
- Старые тесты не сломаны.

### Stage 1-D: Bulk catalog + document endpoint gaps

**Владелец**: `crates/warehouse_core/src/syncserver/catalog.rs` + `crates/warehouse_core/src/syncserver/documents.rs` + `docs/contracts/CLIENT_READY_API_MATRIX.md`

**Контекст**:
- SyncServer имеет `POST /catalog/admin/categories/bulk` и `POST /catalog/admin/units/bulk` — не в API-матрице и не в HTTP-клиенте.
- SyncServer имеет `POST /documents/operations/{operation_id}/documents` — не в матрице.
- `docs/contracts/CLIENT_READY_API_MATRIX.md` не отражает эти эндпоинты и устарел (76 vs 100+ путей).

**Изменяемые файлы**:
- `crates/warehouse_core/src/syncserver/catalog.rs` — добавить (за admin feature-gate): `categories_create_bulk`, `units_create_bulk`
- `crates/warehouse_core/src/syncserver/documents.rs` — добавить: `documents_create_for_operation`
- `docs/contracts/CLIENT_READY_API_MATRIX.md` — добавить:
  - `POST /catalog/admin/categories/bulk` → REQUIRED-ADMIN
  - `POST /catalog/admin/units/bulk` → REQUIRED-ADMIN
  - `POST /documents/operations/{op_id}/documents` → REQUIRED

**Acceptance criteria**:
- HTTP-клиент содержит методы для всех трёх эндпоинтов.
- API-матрица обновлена.

### Stage 1-E: Document PDF rendering contract verification

**Владелец**: `crates/warehouse_core/src/syncserver/documents.rs` + `crates/warehouse_core/src/facade/mod.rs`

**Контекст**: SyncServer реализовал `document_renderer.py` с рендерингом HTML/PDF. Facade имеет `render_document(id) -> Vec<u8>`, но контракт байтового ответа не проверен на реальном стенде.

**Изменяемые файлы** (только при обнаружении расхождений):
- `crates/warehouse_core/src/syncserver/documents.rs` — проверить, что `render` метод правильно обрабатывает Content-Type и бинарный ответ
- `crates/warehouse_core/src/facade/mod.rs` — при необходимости обновить сигнатуру `render_document`

**Acceptance criteria**:
- `cargo run -p warehouse_cli -- document render <doc_id>` возвращает корректные байты PDF.
- Content-Type ответа обрабатывается правильно (не пытается десериализовать как JSON).

### Stage 1 Integration Checkpoint

После завершения Stage 1-A, 1-B, 1-C, 1-D, 1-E:
- `cargo fmt --all -- --check` + `cargo clippy --workspace --all-targets -- -D warnings` — чисто
- `cargo test --workspace` — все тесты (unit + component + mocked HTTP) проходят
- `cargo build --workspace` — собирается без ошибок
- Stand smoke для новых команд: `issue-objects list`, `operations delete`, `document render`

---

## Stage 2: Levels 7–12 — Закрыть оставшиеся уровни TZ_CORE_CLIENT_READY_COMPLETION

### 🟡 Sequential — outbox зависит от draft, engine от outbox

### Stage 2-A: OperationDraftService — verify + fix on real stand

**Контекст**: Draft create падал с `NOT NULL constraint failed: operation_drafts.operation_type` (Fix B). После Stage 0 должен работать. Необходимо верифицировать полный цикл draft на реальном стенде.

**Файлы**:
- `crates/warehouse_core/src/operations/draft_service.rs`
- `crates/warehouse_core/src/operations/validation.rs`
- `crates/warehouse_core/src/facade/mod.rs`

**Сценарий верификации**:
```bash
# Создать draft
cargo run -p warehouse_cli -- draft create RECEIVE --site-id 1

# Добавить строку с catalog item
cargo run -p warehouse_cli -- draft add-line <draft_id> --item-id 1 --qty 5

# Валидировать
cargo run -p warehouse_cli -- draft validate <draft_id>

# Обновить заголовок
cargo run -p warehouse_cli -- draft update-header <draft_id> --comment "test"

# Клонировать
cargo run -p warehouse_cli -- draft clone <draft_id>

# Удалить клон
cargo run -p warehouse_cli -- draft delete <clone_id>
```

**Acceptance criteria**:
- Все команды выше выполняются без ошибок на реальном стенде.
- Drafts переживают перезапуск CLI (persistence).
- Валидация возвращает осмысленные ошибки для невалидных draft.

### Stage 2-B: Outbox + push — integration test on real stand

**Файлы**:
- `crates/warehouse_core/src/operations/outbox_service.rs`
- `crates/warehouse_core/src/syncserver/device_sync.rs`
- `crates/warehouse_core/src/facade/mod.rs`

**Сценарий**:
1. Создать draft (offline — без сети или с отключённым клиентом).
2. `queue_draft_submit` — enqueue в outbox.
3. Подключить сеть / включить клиент.
4. `send_outbox` — отправка pending events.
5. Проверить, что операция создана на сервере (через `operations list`).
6. `pull_once` — обновить локальный кэш.

**Acceptance criteria**:
- Полный цикл offline draft → queue → push → server accept → pull refresh проходит.
- Duplicate-same-payload идемпотентен (не создаёт дубликат).
- Server rejection (422) сохраняет draft для редактирования.

### Stage 2-C: SyncEngine — correct failure reporting + hardening

**Файлы**:
- `crates/warehouse_core/src/sync/engine.rs`
- `crates/warehouse_core/src/sync/pull.rs`
- `crates/warehouse_core/src/sync/bootstrap.rs`
- `crates/warehouse_core/src/facade/mod.rs`

**Контекст**: Engine возвращал SUCCESS даже при ошибках семейств. После Fix K должно быть исправлено. Нужна верификация + hardening:
- Concurrent sync lock работает.
- Cancellation оставляет профиль в resumable состоянии.
- Progress events доходят до facade.

**Acceptance criteria**:
- `sync full` возвращает FAILURE при ошибке любого required family.
- Два одновременных `sync full` не корраптят состояние (lock).
- `sync_status` показывает актуальные cursors + outbox count.

### Stage 2-D: Full FFI for write operations

**Файлы**:
- `crates/warehouse_ffi/src/lib.rs`
- `crates/warehouse_ffi/src/error.rs`

**Контекст**: FFI имеет 48 `extern "C"` экспортов для read-only операций. После Stage 2-A/B/C нужно выставить write-методы:
- `core_draft_create`, `core_draft_add_line`, `core_draft_validate`, `core_draft_queue_submit`
- `core_outbox_list`, `core_outbox_send`, `core_outbox_retry`, `core_outbox_cancel`
- `core_sync_full`, `core_sync_pull`, `core_sync_status`
- `core_issue_objects_list`, `core_issue_objects_merge`
- `core_operation_delete`

**Acceptance criteria**:
- `cargo build -p warehouse_ffi --release` собирает cdylib.
- Все новые FFI-методы имеют корректные `CoreErrorDto` — error codes 0–14.
- Handle lifecycle документирован.

### Stage 2-E: Contract tests, performance, regression hardening (Level 11)

**Файлы**: существующие тесты + новый `tests/contract/`

- Contract test pack: для каждого REQUIRED эндпоинта — сравнение ответа сервера с Rust DTO.
- Migration compatibility: обновление со старой схемы на новую без потери данных.
- Performance: 10k items, 1k balances, 1k operations — замеры времени pull.
- Secret safety: grep логов на токены.

**Acceptance criteria**:
- `cargo test --workspace` ≥ 80 тестов (с 40).
- Contract-тесты падают громко при несовместимых изменениях схемы SyncServer.

### Stage 2-F: Documentation, packaging, handoff (Level 12)

**Файлы**:
- `docs/CORE_FACADE_V1.md` — обновить (добавить Issue Objects, DELETE operation, write FFI)
- `docs/ANDROID_BINDINGS.md` — обновить под новые FFI-методы
- `docs/WPF_BINDINGS.md` — обновить под новые FFI-методы
- `docs/contracts/CLIENT_READY_API_MATRIX.md` — финальная версия со всеми группами
- `README.md` — обновить статус, команды сборки/тестов

**Acceptance criteria**:
- Мобильный разработчик может понять, как использовать `core_issue_objects_list` без чтения исходников Rust.
- WPF разработчик может заменить суррогатные реализации одним DI-свитчом.

---

## Stage 3: Architectural Improvements

### 🟡 Stage 3-A: Sequential (требует Stage 1)

### Stage 3-A: Regenerate API_MAP.md from OpenAPI

**Контекст**: `SyncServer/docs/API_MAP.md` устарел (76 путей, stale header references). Актуальных эндпоинтов 100+. Нужен скрипт регенерации из `/api/openapi.json`.

**Файлы**:
- `SyncServer/docs/API_MAP.md` — регенерировать
- `Warehouse_client_core/docs/contracts/CLIENT_READY_API_MATRIX.md` — сверить с новым API_MAP

**Acceptance criteria**:
- API_MAP.md отражает 100% текущих эндпоинтов (включая issue-objects, DELETE operations, bulk catalog).
- Все новые эндпоинты классифицированы в CLIENT_READY_API_MATRIX.md.

### 🟢 Stage 3-B: Parallel — CI schema-diff + ID type audit

**Два независимых юнита**:

#### 3-B.1: CI schema-diff check

**Файлы**: новый `scripts/check_dto_drift.sh` или CI workflow

По ADR-0007: «если 5+ инцидентов дрифта за релиз, добавить CI schema-diff». Уже 11+ багов контракта — пора.

Скрипт:
1. Выгружает OpenAPI JSON из SyncServer.
2. Сравнивает с Rust DTO (по именам полей, типам, required/optional).
3. Fail CI при несоответствии required полей.

#### 3-B.2: ID type audit

**Файлы**: все файлы `crates/warehouse_core/src/domain/*.rs`

Аудит: где `i64`/`i32` используется для ID, которые сервер возвращает как UUID-строки. Уже известно:
- `operation.id` → UUID string ✅ (Fix E)
- `asset.operation_id` → UUID string ✅
- `device_id` → i32 ✅ (Fix F)
- Нужно проверить: `document.id`, `report.*`, `temporary_item.*`, `recipient.*`

**Acceptance criteria**:
- CI скрипт готов к запуску.
- Аудит ID-типов завершён, все несоответствия задокументированы или исправлены.

---

## Required Test Strategy

### Static checks (каждый stage)

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace
```

### Unit tests

- DTO serde roundtrip для каждого нового/изменённого DTO
- Validation rules (draft, issue object, operation delete permissions)
- Error mapping (CoreError codes)
- FFI error envelope

### Component tests

- HTTP mock server для каждого нового subclient
- SQLite repo tests с реальной временной БД
- Facade методы против fake clients

### Integration tests (real stand)

- Real SyncServer stand (`http://localhost:8000`)
- Real SQLite migrated schema
- Полный цикл: bootstrap → pull → draft → queue → push → pull

### Stand smoke tests

- Все CLI-команды из Stage 0-B **плюс** новые из Stage 1/2
- На чистом профиле

### UI automation

- Не применимо к самому core (ядро не имеет UI)
- WPF FlaUI / Android instrumentation — в отдельных TZ для WarehouseDesktop/WarehouseMobile

### User scenarios

1. First launch: `db init` → `auth-context` → `bootstrap` → `sync-pull` (без ошибок)
2. Offline browse: pull → disconnect → search catalog/balances/issue-objects локально
3. Operation draft: create → add line → validate → queue → reconnect → push → accept
4. DELETE operation: create → cancel → delete
5. Issue objects: list → create → merge → list tree
6. Documents: generate → render PDF → fetch bytes

### Regression pack

- Auth/access scope
- Catalog tree/search
- Balances subject-first
- Operations workflow (create/submit/cancel/accept)
- Temporary items
- Assets pending/lost/issued (плюс issued repository)
- Documents/reports
- Outbox idempotency
- Sync locking/cancellation
- FFI handle lifecycle
- Secret redaction

---

## Real Test Stand

| Service | Address | Health Check |
|---|---|---|
| SyncServer API | `http://localhost:8000` | `GET /api/v1/health` |
| Django | `http://localhost:8001` | `GET /healthz/` |
| PostgreSQL | `localhost:5432` | `pg_isready -h localhost -p 5432 -t 3` |

**Переменные окружения (имена, не значения)**:
- `WAREHOUSE_USER_TOKEN` / `SYNC_USER_TOKEN`
- `WAREHOUSE_DEVICE_TOKEN` / `SYNC_DEVICE_TOKEN`

**Seed data requirements** (для полного сценария):
- root user token
- device token + device_id
- ≥2 sites
- catalog: ≥5 items, ≥2 categories, ≥2 units
- ≥1 recipient
- ≥1 operation (для теста delete)
- ≥1 issue_object + ≥1 issue_object_category (для Stage 1-A)
- ≥1 document (для Stage 1-E)

---

## Evidence Table Template

| Check | Command / Tool | Result | Evidence |
|---|---|---|---|
| Static checks | `cargo fmt`, `cargo clippy`, `cargo build` | pass/fail | output |
| Unit tests | `cargo test --workspace` | pass/fail (count) | test names |
| Component tests | mock HTTP + SQLite | pass/fail | log |
| Stand smoke — Stage 0 | CLI smoke commands (clean profile) | pass/fail | commands + output |
| Stand smoke — Stage 1 | `issue-objects list`, `operations delete`, `document render` | pass/fail | commands + output |
| Stand smoke — Stage 2 | draft → queue → push → pull | pass/fail | scenario log |
| Contract tests | contract test pack | pass/fail | report |
| FFI build | `cargo build -p warehouse_ffi --release` | pass/fail | artifact size |
| Regression | regression pack | pass/fail | report |
| Secret safety | grep logs for token patterns | pass/fail | log audit |

---

## Final Acceptance Criteria

Это ТЗ считается выполненным, когда:

1. **Stage 0**: Все 11 багов закрыты, stand smoke на чистом профиле — зелёный.
2. **Stage 1**: Issue objects, DELETE operation, catalog audit, bulk endpoints, документы — реализованы и проверены на стенде.
3. **Stage 2**: Draft/outbox/push цикл работает, engine корректно рапортует ошибки, FFI расширен, ≥80 тестов.
4. **Stage 3**: API_MAP.md регенерирован, CI schema-diff готов, ID-аудит завершён.
5. `CLIENT_READY_API_MATRIX.md` отражает 100% эндпоинтов SyncServer.
6. `CORE_FACADE_V1.md` актуален.
7. WPF/Android могут начать интеграцию read-only facade (уровень 6 TZ_CORE_CLIENT_READY_COMPLETION) и offline draft (уровень 10).
