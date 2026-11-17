# Frontend Routes & API Coupling Contract

**Feature**: `038-github-branch-upload-src`  
**Date**: 2026-11-05  

## 1. Rotas em `frontend/src/services/api.js`

| Método | Endpoint | Comando Tauri Invocado |
|---|---|---|
| `GET` | `/github/branches?project=<p>&repo=<r>` | `github_list_branches` (`{ project, repo }`) |
| `POST` | `/github/upload` | `github_upload` (`{ project, confirmOverwrite, createNewBranch, baseBranch, newBranch, targetBranch }`) |
| `GET` | `/egressos-dados/sync/branches?repo=<r>` | `egressos_dados_sync_list_branches` (`{ repo }`) |
| `POST` | `/egressos-dados/sync/upload` | `egressos_dados_sync_upload` (`{ confirmOverwrite, commitMessage, createNewBranch, baseBranch, newBranch, targetBranch }`) |

## 2. Componentes e Páginas Atualizados

- **`frontend/src/pages/src/index.astro`**:
  - Altera `<SyncPanel client:load project="src" allowUpload={false} />` para `<SyncPanel client:load project="src" allowUpload={true} />`.
- **`frontend/src/components/SyncPanel.jsx`** (usado por `horizon`, `src` e `egressos`):
  - Adiciona seletor e botão **"Carregar branches"** (`GET /github/branches`) para listar e escolher qualquer branch além da `main`.
  - Na seção de envio (upload), adiciona seletor de modo:
    - **"Criar nova branch com os novos dados"** (selecionado por padrão, com seletor de branch base e campo editável para o nome da nova branch sugerida `update/<project>-...`);
    - **"Enviar na branch selecionada"** (permite escolher `main` ou qualquer outra branch existente do repositório).
  - Exibe na mensagem de sucesso o nome da branch criada/atualizada e o link do commit.
- **`frontend/src/components/EgressosDadosDataControl.jsx`** (usado por `egressos_dados`):
  - Adiciona botão **"Carregar branches"** (`GET /egressos-dados/sync/branches`) e o mesmo seletor de modo (**"Criar nova branch"** vs. **"Enviar na branch selecionada"**) no painel de sincronização com `ifesserra-lab/egressos-dados`.
