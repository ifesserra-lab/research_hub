# IPC Commands Contract: Tabelas, CRUD e Sincronização de `egressos-dados`

**Feature**: `037-egressos-dados-tables`  
**Date**: 2026-11-05  

> **Regra de Segurança (`FR-001`)**: Todos os comandos abaixo invocam obrigatoriamente `state.require_egressos_dados_session()?` na primeira linha, recusando chamadas sem o login especial ativo da organização `ifesserra-lab` com `AppError::OrgUnauthorized` (`kind: "org_unauthorized"`).

## 1. Catálogo, CRUD e Metadados (`src-tauri/src/commands.rs`)

### `egressos_dados_get_catalog`
- **Arguments**: `{}`
- **Returns**: `EgressosDadosCatalogView`
  - `loaded: bool`
  - `imported_at: Option<String>`
  - `total_files: usize` (39 quando carregado)
  - `total_rows: usize`
  - `default_local_dir: String` (`"/home/thiagodeps/egressos-dados"` se existir na máquina, ou `""`)
  - `default_local_dir_exists: bool`
  - `files: Vec<EgressosDadosFileSummary>` (com `file_key`, `file_name`, `relative_path`, `title`, `kind`, `publication`, `role`, `primary_section`, `sections`, `columns`, `entity_group`, `total_rows`)

### `egressos_dados_list_rows`
- **Arguments**: `{ file_key: String, section_key: Option<String>, limit: Option<i64>, offset: Option<i64>, search: Option<String>, sort: Option<String>, order: Option<String> }`
- **Returns**: `EgressosDadosRowPage` (`items` com `_row_id`, `total`, `limit`, `offset`, `file_key`, `section_key`, `columns`, `entity_group`, `is_map_section`)

### `egressos_dados_get_row`
- **Arguments**: `{ id: i64 }`
- **Returns**: `Record` (contendo `_row_id` e todos os atributos do JSON original, preservando `"id"` textual quando existente)

### `egressos_dados_create_row`
- **Arguments**: `{ file_key: String, section_key: Option<String>, payload: Record }`
- **Returns**: `Record`

### `egressos_dados_update_row`
- **Arguments**: `{ id: i64, payload: Record, propagate: Option<bool> }`
- **Returns**: `EgressosDadosMutationResult` (`{ record: Record, propagated_count: usize }`)

### `egressos_dados_delete_row`
- **Arguments**: `{ id: i64, propagate: Option<bool> }`
- **Returns**: `EgressosDadosDeleteResult` (`{ deleted: usize, propagated_deleted: usize }`)

### `egressos_dados_get_envelope` & `egressos_dados_update_envelope`
- **Arguments (`get`)**: `{ file_key: String }` → `Record`
- **Arguments (`update`)**: `{ file_key: String, payload: Record }` → `Record`

## 2. Importação, Exportação e Snapshots Locais

### `egressos_dados_import_dir`
- **Arguments**: `{ path: String }` (aceita raiz `/home/thiagodeps/egressos-dados` ou subpasta `/home/thiagodeps/egressos-dados/data`)
- **Returns**: `EgressosDadosImportSummary` (`{ path, total_files: 39, total_rows, replaced, snapshot, imported_at }`)

### `egressos_dados_export_dir`
- **Arguments**: `{ path: String }` (grava `alunos.json` na raiz e 38 arquivos em `data/*.json`)
- **Returns**: `EgressosDadosExportSummary` (`{ path, total_files: 39, total_rows }`)

### `egressos_dados_list_snapshots`
- **Arguments**: `{}`
- **Returns**: `Vec<SnapshotInfo>` (filtrando prefixo `"egressos-dados-"`)

## 3. Sincronização GitHub (`ifesserra-lab/egressos-dados`)

### `egressos_dados_sync_get_config` / `egressos_dados_sync_set_config`
- **Default Config**: `{ repo: "ifesserra-lab/egressos-dados", branch: "main", path: "." }`
- **Credential**: Reutiliza automaticamente o token em memória de `EgressosDadosAuth` (`FR-010`, Clarificação Q4).

### `egressos_dados_sync_download`
- **Arguments**: `{ source_url: Option<String>, repo: Option<String>, branch: Option<String>, path: Option<String> }`
- **Returns**: `EgressosDadosImportSummary` (baixa e valida os 39 arquivos JSON `alunos.json` + `data/*.json` em diretório temporário, cria snapshot `egressos-dados-pre-sync` se necessário e importa na base).

### `egressos_dados_sync_upload`
- **Arguments**: `{ commit_message: Option<String> }`
- **Returns**: `UploadSummary` (publica os 39 arquivos JSON `alunos.json` + `data/*.json` em um único commit atômico via Git Data API usando o token da sessão `EgressosDadosAuth`).
