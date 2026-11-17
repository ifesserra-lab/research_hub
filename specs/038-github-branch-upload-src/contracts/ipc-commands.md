# IPC Commands Contract: Upload GitHub em Nova Branch, Seleção de Branches e Upload SRC

**Feature**: `038-github-branch-upload-src`  
**Date**: 2026-11-05  

## 1. Sincronização Geral (`horizon`, `src`, `egressos`) — `src-tauri/src/commands.rs`

### `github_list_branches` (Novo comando)
- **Guard**: `state.require_session()?`
- **Arguments**:
  - `project: String` (`"horizon"` | `"src"` | `"egressos"`)
  - `repo: Option<String>` (se omitido, usa o repositório configurado para `project` em `sync_config.json`)
- **Returns**: `Vec<RemoteBranchInfo>` (`[{ name: "main", commit_sha: "..." }, ...]`)

### `github_upload` (Atualizado com suporte a `src`, criação de nova branch e seleção de branch)
- **Guard**: `state.require_session()?`
- **Arguments**:
  - `project: String` (`"horizon"` | `"src"` | `"egressos"`)
  - `confirm_overwrite: Option<bool>`
  - `create_new_branch: Option<bool>`
  - `base_branch: Option<String>`
  - `new_branch: Option<String>`
  - `target_branch: Option<String>`
- **Behavior**:
  - Para `"src"`: gera o JSON consolidado via `src_domain::export(&conn)?.into_bytes()` e publica no caminho `.json` configurado.
  - Para `"horizon"`: gera o arquivo `.zip` canônico via `export::build_archive` e publica no caminho `.zip` configurado.
  - Para `"egressos"`: gera os 21 arquivos `.json` em memória via `egressos_domain::export_files_in_memory(&conn)?` e publica no diretório configurado.
  - Se `create_new_branch == Some(true)`: ramifica a partir de `base_branch` (ou `cfg.branch`), cria o commit e registra `refs/heads/{new_branch}` via `POST /repos/{owner}/{repo}/git/refs`.
  - Se `create_new_branch == Some(false)` ou `None`: publica na branch `target_branch` (ou `cfg.branch`).
- **Returns**: `UploadResult` (`{ commit_sha, html_url, replaced, branch, created_branch }`)

---

## 2. Sincronização Especial `egressos-dados` (`ifesserra-lab`) — `src-tauri/src/commands.rs`

### `egressos_dados_sync_list_branches` (Novo comando)
- **Guard**: `let auth = state.require_egressos_dados_session()?` (reutiliza `auth.token` em memória)
- **Arguments**:
  - `repo: Option<String>` (padrão `"ifesserra-lab/egressos-dados"`)
- **Returns**: `Vec<RemoteBranchInfo>`

### `egressos_dados_sync_upload` (Atualizado com suporte a criação de nova branch e seleção de branch)
- **Guard**: `let auth = state.require_egressos_dados_session()?`
- **Arguments**:
  - `confirm_overwrite: Option<bool>`
  - `commit_message: Option<String>`
  - `create_new_branch: Option<bool>`
  - `base_branch: Option<String>`
  - `new_branch: Option<String>`
  - `target_branch: Option<String>`
- **Returns**: `UploadResult` (`{ commit_sha, html_url, replaced, branch, created_branch }`)
