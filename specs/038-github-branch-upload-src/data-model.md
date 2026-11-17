# Data Model: Upload GitHub em Nova Branch, Seleção de Branches e Liberação de Upload para o Projeto SRC

**Feature**: `038-github-branch-upload-src`  
**Date**: 2026-11-05  

## 1. Estruturas de Domínio (`src-tauri/src/sync_domain.rs`)

### 1.1 `RemoteBranchInfo`
Representa uma branch existente em um repositório GitHub consultado via API (`GET /repos/{owner}/{repo}/branches`).

| Campo | Tipo | Descrição |
|---|---|---|
| `name` | `String` | Nome da branch remota (ex.: `"main"`, `"develop"`, `"update/horizon-1700000000"`). |
| `commit_sha` | `String` | SHA do commit `HEAD` atual da branch. |

### 1.2 `UploadBranchOptions`
Parâmetros opcionais de controle de branch enviados no momento do upload para qualquer um dos 4 projetos (`horizon`, `src`, `egressos`, `egressos_dados`).

| Campo | Tipo | Padrão | Descrição |
|---|---|---|---|
| `create_new_branch` | `bool` | `false` (API) / `true` (UI default) | Quando `true`, cria uma nova referência `refs/heads/{new_branch}` no GitHub a partir de `base_branch` (ou da branch configurada no projeto). Quando `false`, publica na branch existente `target_branch` (ou na branch configurada no projeto). |
| `base_branch` | `Option<String>` | `None` (usa `cfg.branch`) | Branch base a partir da qual a nova branch será ramificada quando `create_new_branch == true`. |
| `new_branch` | `Option<String>` | `None` (gera `update/<project>-<stamp>`) | Nome da nova branch a ser criada no GitHub quando `create_new_branch == true`. |
| `target_branch` | `Option<String>` | `None` (usa `cfg.branch`) | Branch existente onde o commit será gravado quando `create_new_branch == false`. |

### 1.3 Regras de Validação (`sync_domain.rs`)
- **`validate_branch(branch: &str)`**:
  - Não pode ser vazia, conter espaços em branco, conter `..`, iniciar com `-` ou `/`, nem terminar com `/` ou `.lock`.
- **`generate_branch_name(project: &str) -> String`**:
  - Gera um nome de branch válido e determinístico com prefixo `update/{project}-{timestamp}`.
- **`check_upload_allowed(project: &str)`**:
  - Autoriza todos os 4 projetos: `"horizon" | "src" | "egressos" | "egressos_dados" => Ok(())`.
- **`validate_path(project: &str, path: &str)`**:
  - `"horizon"`: exige sufixo `.zip`.
  - `"src"`: exige sufixo `.json`.
  - `"egressos"` e `"egressos_dados"`: caminho de diretório válido (sem `..` e não vazio).

---

## 2. Resultado de Upload (`src-tauri/src/sync_github.rs`)

### 2.1 `UploadResult`

| Campo | Tipo | Descrição |
|---|---|---|
| `commit_sha` | `String` | SHA do commit recém-criado no GitHub. |
| `html_url` | `String` | URL web do commit no GitHub. |
| `replaced` | `bool` | `true` quando atualizou arquivo/pasta preexistente em uma branch existente; `false` quando criou arquivo novo ou publicou em uma nova branch. |
| `branch` | `String` | Nome da branch onde o commit foi efetivamente publicado (seja a nova branch criada ou a branch existente selecionada). |
| `created_branch` | `bool` | `true` se uma nova branch (`refs/heads/{branch}`) foi criada nesta operação; `false` se atualizou uma branch existente. |
