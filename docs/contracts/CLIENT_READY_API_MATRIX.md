# Client-Ready API Endpoint Coverage Matrix

> Source: `SyncServer/docs/API_MAP.md` (97 paths, 19 groups)
> Base path: `/api/v1`

## Legend

| Classification | Meaning |
|---|---|
| **REQUIRED** | Client must call this through CoreFacade |
| **REQUIRED-ADMIN** | Admin UI only; client-core admin gate |
| **COMPAT-ONLY** | Legacy `/business/*` or device-POST wrappers; not used by core |
| **OUT-OF-SCOPE** | Not needed by any client |

---

## 1. Health API (5 endpoints) — REQUIRED

| Method | Path | Use in core |
|---|---|---|
| `GET` | `/health` | Connection readiness check before bootstrap |
| `GET` | `/ready` | Full readiness including DB |
| `GET` | `/health/detailed` | Diagnostics screen |
| `GET` | `/health/readiness` | Load-balancer semantics, same as `/ready` |
| `GET` | `/health/liveness` | Not used (k8s internal) — OUT-OF-SCOPE |

---

## 2. Auth API (4 endpoints) — REQUIRED

| Method | Path | Use in core |
|---|---|---|
| `POST` | `/auth/sync-user` | Bootstrap identity; root-only (admin gate) |
| `GET` | `/auth/me` | Profile refresh, identity cache |
| `GET` | `/auth/sites` | Available sites after token bind |
| `GET` | `/auth/context` | Full context: user+role+sites+permissions+device |

---

## 3. Catalog Read API (11 endpoints) — REQUIRED

| Method | Path | Use in core |
|---|---|---|
| `GET` | `/catalog/items?updated_after=` | Pull-sync items (cursor) |
| `GET` | `/catalog/categories?updated_after=` | Pull-sync categories (cursor) |
| `GET` | `/catalog/categories/tree` | Category tree for navigation |
| `GET` | `/catalog/units?updated_after=` | Pull-sync units (cursor) |
| `GET` | `/catalog/sites` | Catalog site list |
| `GET` | `/catalog/read/items?page=&page_size=&query=` | Search/browse items |
| `GET` | `/catalog/read/categories` | Browse/search categories |
| `GET` | `/catalog/read/categories/{id}/items` | Items in a category |
| `GET` | `/catalog/read/categories/{id}/children` | Child categories |
| `GET` | `/catalog/read/categories/{id}/parent-chain` | Category breadcrumb |

---

## 4. Catalog Admin API (12 endpoints) — REQUIRED-ADMIN

Feature-gated behind runtime `allow_catalog_admin`. Only `root` and `chief_storekeeper` may call.

| Method | Path |
|---|---|
| `GET/POST` | `/catalog/admin/units` |
| `GET/PATCH/DELETE` | `/catalog/admin/units/{id}` |
| `POST` | `/catalog/admin/units/bulk` |
| `GET/POST` | `/catalog/admin/categories` |
| `GET/PATCH/DELETE` | `/catalog/admin/categories/{id}` |
| `POST` | `/catalog/admin/categories/bulk` |
| `GET/POST` | `/catalog/admin/items` |
| `GET/PATCH/DELETE` | `/catalog/admin/items/{id}` |

---

## 5. Operations API (8 endpoints) — REQUIRED

| Method | Path | Use in core |
|---|---|---|
| `GET` | `/operations?page=&site_id=&status=` | List/filter operation history |
| `GET` | `/operations/{id}` | Single operation detail + lines |
| `POST` | `/operations` | Create operation online |
| `PATCH` | `/operations/{id}` | Update operation |
| `PATCH` | `/operations/{id}/effective-at` | Change effective date |
| `POST` | `/operations/{id}/submit` | Submit for processing |
| `POST` | `/operations/{id}/cancel` | Cancel operation |
| `POST` | `/operations/{id}/accept-lines` | Accept/reject lines |

---

## 6. Balances API (3 endpoints) — REQUIRED

| Method | Path | Use in core |
|---|---|---|
| `GET` | `/balances?site_id=&item_id=&category_id=` | Pull-sync balances |
| `GET` | `/balances/by-site?site_id=` | Per-site balance view |
| `GET` | `/balances/summary` | Stock summary |

---

## 7. Temporary Items API (6 endpoints) — REQUIRED

| Method | Path | Use in core |
|---|---|---|
| `GET` | `/temporary-items` | List temp items |
| `GET` | `/temporary-items/{id}` | Single temp item |
| `POST` | `/temporary-items/{id}/approve-as-item` | Temp→catalog promotion |
| `GET` | `/temporary-items/{id}/operations` | Operations referencing temp |
| `POST` | `/temporary-items/{id}/merge` | Merge into existing item |
| `DELETE` | `/temporary-items/{id}` | Soft delete |

---

## 8. Documents API (6 endpoints) — REQUIRED

| Method | Path | Use in core |
|---|---|---|
| `POST` | `/documents/generate` | Generate from operation |
| `GET` | `/documents/{id}` | Document metadata |
| `GET` | `/documents/{id}/render` | Binary render (HTML/PDF) |
| `GET` | `/documents` | List documents |
| `PATCH` | `/documents/{id}/status` | Finalize/void |
| `GET` | `/documents/operations/{op_id}/documents` | Link by operation |
| `POST` | `/documents/operations/{op_id}/documents` | Create document for operation (shortcut) |

---

## 9. Recipients API (6 endpoints) — REQUIRED

| Method | Path | Use in core |
|---|---|---|
| `GET` | `/recipients` | Pull-sync + search |
| `GET` | `/recipients/{id}` | Single recipient |
| `POST` | `/recipients` | Create |
| `PATCH` | `/recipients/{id}` | Update |
| `DELETE` | `/recipients/{id}` | Delete |
| `POST` | `/recipients/merge` | Merge duplicates |

---

## 10. Asset Registers API (5 endpoints) — REQUIRED

| Method | Path | Use in core |
|---|---|---|
| `GET` | `/pending-acceptance` | Pending acceptance list |
| `GET` | `/lost-assets` | Lost assets list |
| `GET` | `/lost-assets/{line_id}` | Single lost row |
| `POST` | `/lost-assets/{line_id}/resolve` | Resolve lost asset |
| `GET` | `/issued-assets` | Issued assets list |

---

## 11. Reports API (2 endpoints) — REQUIRED

| Method | Path | Use in core |
|---|---|---|
| `GET` | `/reports/item-movement` | Movement report |
| `GET` | `/reports/stock-summary` | Stock summary |

---

## 12. Device Sync API (4 endpoints) — REQUIRED

| Method | Path | Use in core |
|---|---|---|
| `POST` | `/ping` | Sync handshake before push/pull |
| `POST` | `/push` | Push outbox events |
| `POST` | `/pull?since_seq=&limit=` | Pull server events |
| `POST` | `/bootstrap/sync` | Root-only full bootstrap |

---

## 13. Admin API (16 endpoints) — REQUIRED-ADMIN

Feature-gated behind `allow_admin`. Only root. Covers users, sites, devices, access scopes, token rotation.

---

## 14. Compatibility API (7 endpoints) — COMPAT-ONLY

All `/business/*` and device-POST catalog wrappers. Core uses native GET/`X-User-Token` catalog endpoints, not these.

---

## Summary

| Classification | Count |
|---|---|
| REQUIRED | 68 |
| REQUIRED-ADMIN | 28 |
| COMPAT-ONLY | 7 |
| OUT-OF-SCOPE | 1 |

Core must implement ~39 endpoint calls (the REQUIRED ones excluding admin-gated) for full client-ready status. Admin endpoints are feature-gated.
