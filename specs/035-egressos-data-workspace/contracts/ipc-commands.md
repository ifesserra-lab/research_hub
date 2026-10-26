# Contract: Comandos IPC Tauri do Domínio Egressos (`ipc-commands.md`)

**Date**: 2026-10-22 | **Feature**: [spec.md](../spec.md)

Todos os comandos abaixo exigem sessão autenticada (`state.require_session()?`) e mantêm os comandos do Horizon (`list_entities`, `get_entity`, etc.) e do SRC (`src_*`) 100% inalterados (`FR-002`, `FR-003`).

## 1. Importação, Exportação e Resumo da Base Egressos

### `import_egressos_dir`
- **Argumentos**: `{ path: Option<String> }`
  - Se `path` for `None`, abre o diálogo nativo de seleção de pasta (`blocking_pick_folder`), iniciando em `/home/thiagodeps/egressos/dados` quando o caminho existir no computador (`FR-004`). Se o curador cancelar o diálogo, retorna `Ok(None)`.
- **Evento emitido**: `"egressos-import://progress"` com payload `{ phase: "validating" | "importing" | "done", file_key: Option<String>, files_done: usize, files_total: usize }`.
- **Retorno**: `Result<Option<EgressosImportSummary>, AppError>`
  ```json
  {
    "path": "/home/thiagodeps/egressos/dados",
    "total_files": 21,
    "total_rows": 1142,
    "snapshot": "egressos-pre-import-1761156000000.db",
    "imported_at": "2026-10-22T18:00:00Z"
  }
  ```

### `export_egressos_dir`
- **Argumentos**: `{ path: Option<String> }`
  - Se `path` for `None`, abre o diálogo nativo de seleção de pasta (`blocking_pick_folder`) para gravar os 21 arquivos `.json` (`FR-010`).
- **Retorno**: `Result<Option<EgressosExportSummary>, AppError>`
  ```json
  {
    "path": "/home/thiagodeps/egressos/dados",
    "total_files": 21,
    "total_rows": 1142
  }
  ```

### `egressos_get_catalog`
- **Argumentos**: `{}`
- **Retorno**: `Result<EgressosCatalogView, AppError>`
  - Retorna o resumo da base carregada (`imported_at`, `total_files`, `total_rows`, `default_local_dir`) e a lista das **21 abas (`files`)**, cada uma com `file_key`, `file_name`, `title`, `kind`, `primary_section`, `sections` (com contagem de registros por seção), `columns` e `entity_group`.

---

## 2. CRUD de Registros e Envelope por Aba/Seção (`FR-007`..`FR-009`)

### `egressos_list_rows`
- **Argumentos**:
  - `fileKey: String` (um dos 21 `file_key`)
  - `sectionKey: Option<String>` (default: `primary_section` do arquivo)
  - `limit: Option<i64>` (default: 50)
  - `offset: Option<i64>` (default: 0)
  - `search: Option<String>`
  - `sort: Option<String>`
  - `order: Option<String>` (`"asc"` | `"desc"`)
- **Retorno**: `Result<EgressosRowPage, AppError>`
  ```json
  {
    "items": [ { "id": 1, "_record_key": "A", "label": "A", "nome": "...", ... } ],
    "total": 50,
    "limit": 50,
    "offset": 0,
    "file_key": "egressos_perfil",
    "section_key": "egressos",
    "columns": ["label", "nome", "curso", "cargo_atual", "empresa_atual", "local_atual", "area_atual"],
    "entity_group": "alumni"
  }
  ```

### `egressos_get_row`
- **Argumentos**: `{ id: i64 }`
- **Retorno**: `Result<Record, AppError>` (objeto JSON completo da linha + `"id"` e `"_record_key"`).

### `egressos_create_row`
- **Argumentos**: `{ fileKey: String, sectionKey: Option<String>, payload: Record }`
- **Retorno**: `Result<Record, AppError>` (cria nova linha com `ord = MAX(ord) + 1`, valida unicidade de `_key` quando seção é mapa, retorna registro criado).

### `egressos_update_row`
- **Argumentos**: `{ id: i64, payload: Record, propagate: Option<bool> }`
  - Quando `propagate == Some(true)` (`FR-008a`), propaga os campos compartilhados do grupo da entidade (`alumni` ou `company`) para os registros correspondentes nas demais tabelas relacionadas dentro da mesma transação.
- **Retorno**: `Result<EgressosMutationResult, AppError>`
  ```json
  {
    "record": { "id": 1, ... },
    "propagated_count": 4
  }
  ```

### `egressos_delete_row`
- **Argumentos**: `{ id: i64, propagate: Option<bool> }`
  - Quando `propagate == Some(true)`, remove também os registros correspondentes da mesma entidade (`alumni` ou `company`) nas tabelas relacionadas.
- **Retorno**: `Result<EgressosDeleteResult, AppError>` (`{ "deleted": 1, "propagated_deleted": 0 }`).

### `egressos_get_envelope` / `egressos_update_envelope`
- **Argumentos (`get`)**: `{ fileKey: String }` → retorna `Record` com os metadados/campos escalares e objetos auxiliares do topo do arquivo JSON (`FR-009`).
- **Argumentos (`update`)**: `{ fileKey: String, payload: Record }` → atualiza os metadados de nível de arquivo preservando as seções tabulares.

---

## 3. Extensão dos Comandos de Sincronização GitHub (`github_*`) para `project = "egressos"` (`FR-011`..`FR-013`)

Os comandos existentes `github_get_config`, `github_set_config`, `github_download`, `github_check_destination` e `github_upload` passam a aceitar `project: "egressos"`:
- **`github_set_config`**: aceita `path` de diretório (ex.: `"dados"`), com default sugerido `repo: "thiagodeps/egressos"`, `branch: "main"`, `path: "dados"`.
- **`github_download`**: aceita URL de pasta do GitHub (ex.: `https://github.com/thiagodeps/egressos/tree/main/dados` ou `https://github.com/thiagodeps/egressos`) ou usa `(repo, branch, path)` configurado, baixa os 21 arquivos `.json` da pasta, valida-os atomicamente, cria snapshot `egressos-pre-import-*` e importa na base Egressos.
- **`github_check_destination`**: verifica se o repositório, a branch e o diretório `path` (ex.: `dados`) existem na branch alvo, informando `file_exists: true` se os arquivos `.json` já existirem na pasta remota com conteúdo divergente.
- **`github_upload`**: gera os 21 arquivos `.json` em memória a partir da base Egressos atual e publica todos os 21 arquivos em um único commit na branch existente via Git Data API (`upload_directory`).
