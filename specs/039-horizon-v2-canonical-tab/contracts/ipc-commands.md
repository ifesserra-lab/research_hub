# IPC & Frontend API Bridge Contracts: Aba Horizon (`horizon_v2`) e Horizon Legacy

**Feature**: `039-horizon-v2-canonical-tab`  
**Date**: 2026-11-12

## 1. Preservação Integral do Contrato do `Horizon Legacy`

Todos os comandos Tauri e rotas em `frontend/src/services/api.js` do **Horizon Legacy** permanecem inalterados:
- `GET /{entity}` → `invoke('list_entities', { entity, limit, offset, search, sort, order })`
- `GET /{entity}/{id}` → `invoke('get_entity', { entity, id })`
- `POST /{entity}` → `invoke('create_entity', { entity, payload })`
- `PUT /{entity}/{id}` → `invoke('update_entity', { entity, id, payload })`
- `DELETE /{entity}/{id}` → `invoke('delete_entity', { entity, id })`
- `POST /merge/{entity}` → `invoke('merge_entities', { entity, sourceIds, resolvedData })`
- `POST /link` → `invoke('link_entities', { parentType, parentId, childType, childId })`
- `invoke('import_canonical_zip', { path })` (evento `import://progress`)
- `invoke('export_canonical_zip')` (evento `export://progress`)
- Sincronização GitHub com `project: "horizon"`

---

## 2. Novos Comandos Tauri (`src-tauri/src/commands.rs`) para a Nova Aba `Horizon`

### 2.1 CRUD das 15 Entidades (`HORIZON_V2_ENTITIES`)

```rust
#[tauri::command(async)]
pub fn hv2_list_entities(
    state: State<'_, AppState>,
    entity: String,
    limit: Option<i64>,
    offset: Option<i64>,
    search: Option<String>,
    sort: Option<String>,
    order: Option<String>,
) -> Result<Page, AppError>;

#[tauri::command(async)]
pub fn hv2_get_entity(
    state: State<'_, AppState>,
    entity: String,
    id: i64,
) -> Result<Record, AppError>;

#[tauri::command(async)]
pub fn hv2_create_entity(
    state: State<'_, AppState>,
    entity: String,
    payload: Record,
) -> Result<Record, AppError>;

#[tauri::command(async)]
pub fn hv2_update_entity(
    state: State<'_, AppState>,
    entity: String,
    id: i64,
    payload: Record,
) -> Result<Record, AppError>;

#[tauri::command(async)]
pub fn hv2_delete_entity(
    state: State<'_, AppState>,
    entity: String,
    id: i64,
) -> Result<(), AppError>;
```

### 2.2 Fusão e Vínculo (`hv2_merge_entities`, `hv2_link_entities`)

```rust
#[tauri::command(async)]
pub fn hv2_merge_entities(
    state: State<'_, AppState>,
    entity: String,
    source_ids: Vec<i64>,
    resolved_data: Record,
) -> Result<Record, AppError>;

#[tauri::command(async)]
pub fn hv2_link_entities(
    state: State<'_, AppState>,
    parent_type: String,
    parent_id: i64,
    child_type: String,
    child_id: i64,
) -> Result<Record, AppError>;
```

### 2.3 Importação e Exportação Canônica Isolada (`import_horizon_v2_zip`, `export_horizon_v2_zip`)

```rust
#[tauri::command(async)]
pub fn import_horizon_v2_zip(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    path: Option<String>,
) -> Result<Option<ImportSummary>, AppError>;

#[tauri::command(async)]
pub fn export_horizon_v2_zip(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<ExportSummary>, AppError>;
```

- **Eventos Tauri**:
  - `horizon-v2-import://progress` (payload: `{ table: string, rows: number }`)
  - `horizon-v2-export://progress` (payload: `{ table: string, rows: number }`)
- **Artefatos no `app_data_dir`**:
  - Arquivo de referência: `horizon_v2_original.zip`
  - Snapshots: `snapshots/horizon-v2-<timestamp>.db`

---

## 3. Rotas Mapeadas em `frontend/src/services/api.js`

Registradas antes do curinga genérico `/{entity}`:

| Método e Caminho em `apiFetch` | Comando Tauri Invocado | Parâmetros |
| :--- | :--- | :--- |
| `POST /horizon-v2/import` | `import_horizon_v2_zip` | `{ path: b?.path ?? null }` |
| `POST /horizon-v2/export` | `export_horizon_v2_zip` | `{}` |
| `POST /horizon-v2/merge/:entity` | `hv2_merge_entities` | `{ entity, sourceIds: b.source_ids, resolvedData: b.resolved_data }` |
| `POST /horizon-v2/link` | `hv2_link_entities` | `{ parentType: b.parent_type, parentId: b.parent_id, childType: b.child_type, childId: b.child_id }` |
| `GET /horizon-v2/:entity/:id` | `hv2_get_entity` | `{ entity, id: Number(id) }` |
| `PUT /horizon-v2/:entity/:id` | `hv2_update_entity` | `{ entity, id: Number(id), payload: b }` |
| `DELETE /horizon-v2/:entity/:id` | `hv2_delete_entity` | `{ entity, id: Number(id) }` |
| `POST /horizon-v2/:entity` | `hv2_create_entity` | `{ entity, payload: b ?? {} }` |
| `GET /horizon-v2/:entity` | `hv2_list_entities` | `{ entity, limit, offset, search, sort, order }` |

---

## 4. Contrato de Sincronização GitHub (`project = "horizon_v2"`)

Os comandos `github_get_config`, `github_set_config`, `github_download`, `github_check_destination`, `github_list_branches` e `github_upload` aceitam `project: "horizon_v2"`:
- `validate_path("horizon_v2", path)` exige extensão `.zip` (mesma regra de `"horizon"`).
- `check_upload_allowed("horizon_v2")` retorna `Ok(())`.
- `download_and_import(..., "horizon_v2", ...)` invoca `import::import_horizon_v2_archive`.
- `github_upload(..., "horizon_v2", ...)` reconstrói o pacote via `export::build_horizon_v2_archive` usando `horizon_v2_original.zip`.
