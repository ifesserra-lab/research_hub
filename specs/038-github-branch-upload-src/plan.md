# Implementation Plan: Upload GitHub em Nova Branch, Seleção de Branches e Liberação de Upload para o Projeto SRC

**Branch**: `038-github-branch-upload-src` | **Date**: 2026-11-05 | **Spec**: [spec.md](file:///home/thiagodeps/research_hub/specs/038-github-branch-upload-src/spec.md)

**Input**: Feature specification from `/specs/038-github-branch-upload-src/spec.md`

## Summary

Habilitar em todos os **4 programas suportados** pelo ResearchHub (`horizon`, `src`, `egressos` e `egressos_dados`):
1. **Dois modos de branch no upload para o GitHub** (`FR-004`..`FR-007`):
   - **Criar nova branch com os novos dados** (padrão na UI): ramifica a partir de uma branch base selecionada (ex.: `main`), cria os blobs/árvore/commit e registra a nova branch remota via `POST /repos/{owner}/{repo}/git/refs` sem modificar a branch base;
   - **Enviar na branch selecionada**: permite escolher `main` ou qualquer outra branch existente do repositório e atualizar sua referência.
2. **Listagem e seleção de branches existentes além da `main`** (`FR-006`) via `GET /repos/{owner}/{repo}/branches` em `SyncPanel.jsx` (`horizon`, `src`, `egressos`) e `EgressosDadosDataControl.jsx` (`egressos_dados`).
3. **Liberação completa do upload do projeto SRC (`src`)** (`FR-001`..`FR-003`), removendo o bloqueio `sync_policy`, validando caminhos `.json`, habilitando `allowUpload={true}` em `/src` e publicando no GitHub o JSON consolidado gerado por `src_domain::export`.

## Technical Context

**Language/Version**: Rust 2021 (1.77+) no núcleo Tauri 2.0; JavaScript/JSX (ES2022) com Astro 5 e React 19 no front-end  
**Primary Dependencies**: `tauri` 2, `rusqlite` 0.32 (`bundled`), `serde` / `serde_json`, `reqwest` 0.12 (`json`, `rustls-tls`), `base64`, `url`; testes HTTP com `wiremock` 0.6 e `tokio`  
**Storage**: SQLite local + `sync_config.json` no diretório de dados do SO (`app_data_dir`)  
**Testing**: `cargo test` (unitários + integração `wiremock` em `src-tauri/tests/sync_github.rs`) e `npm test` (`Vitest` + `@testing-library/react` em `frontend/tests/unit/`)  
**Target Platform**: Linux e Windows (aplicação desktop Tauri 2.0)  
**Project Type**: Desktop application (`src-tauri` + `frontend`)  
**Performance Goals**: Listagem de branches em <3s; upload de arquivo único ou diretório (21/39 JSONs) com criação de nova branch em um único commit atômico  
**Constraints**: Limite de 100 MB por transferência; zero vazamento de tokens em erros/logs; toda lógica HTTP/Git Data API restrita ao núcleo Rust (`src-tauri/src/`)  
**Scale/Scope**: 4 programas (`horizon`, `src`, `egressos`, `egressos_dados`), 2 novos comandos IPC de listagem de branches, evolução compatível dos 2 comandos IPC de upload e atualização de `SyncPanel.jsx`, `EgressosDadosDataControl.jsx` e `src/index.astro`

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. TDD Obrigatório**: **PASS** — Testes unitários em `sync_domain.rs`, testes `wiremock` em `tests/sync_github.rs` e testes `Vitest` em `api.test.js`, `SyncPanel.test.jsx` e `EgressosDadosDataControl.test.jsx` serão escritos antes da implementação.
- **II. Paridade Funcional Antes de Melhoria**: **PASS** — O upload do `src` reutiliza `src_domain::export(&conn)`, produzindo saída byte a byte idêntica à exportação manual local (`SC-003`).
- **III. Testes Rigorosos (Rust e Front-end)**: **PASS** — Cobertura automatizada completa em Rust (`cargo test`) e Front-end (`npm test` + `npm run build`).
- **IV. CRUD Completo e Operações Diretas**: **PASS** — Mantém todas as operações diretas existentes e estende a publicação GitHub aos 4 projetos.
- **V. Aplicação Desktop Local e Autocontida**: **PASS** — Sem servidor externo próprio; comunicação com GitHub restrita às ações explícitas de sincronização.
- **VI. Lógica de Negócio em Rust**: **PASS** — Toda validação de branches, geração de nomes de branch e chamadas à Git Data API (`GET /branches`, `POST /git/refs`, `PATCH /git/refs/heads/*`) residem em `src-tauri/src/sync_domain.rs` e `src-tauri/src/sync_github.rs`, trafegando exclusivamente por `frontend/src/services/api.js`.

## Project Structure

### Documentation (this feature)

```text
specs/038-github-branch-upload-src/
├── plan.md              # Este arquivo
├── research.md          # Decisões técnicas (R1..R4)
├── data-model.md        # Entidades RemoteBranchInfo, UploadBranchOptions e UploadResult
├── quickstart.md        # Guia de validação automatizada
├── contracts/
│   ├── ipc-commands.md  # Contratos dos comandos Tauri atualizados e novos
│   └── frontend-routes.md # Rotas de api.js e componentes React/Astro
└── tasks.md             # Lista de tarefas TDD (/speckit-tasks)
```

### Source Code (repository root)

```text
src-tauri/
├── src/
│   ├── sync_domain.rs           # Liberação de upload para 'src', validação '.json', generate_branch_name e UploadBranchOptions
│   ├── sync_github.rs           # list_branches, create_branch_ref e suporte a nova branch em upload_file / upload_directory
│   ├── commands.rs              # github_list_branches, egressos_dados_sync_list_branches e evolução de github_upload / egressos_dados_sync_upload
│   └── lib.rs                   # Registro de github_list_branches e egressos_dados_sync_list_branches
└── tests/
    └── sync_github.rs           # Testes wiremock para list_branches, upload em nova branch nos 4 projetos e upload do SRC

frontend/
├── src/
│   ├── services/
│   │   └── api.js               # Rotas /github/branches, /egressos-dados/sync/branches e parâmetros de branch no upload
│   ├── components/
│   │   ├── SyncPanel.jsx        # Seletor de branches, modo "Criar nova branch" vs "Enviar na branch selecionada"
│   │   └── EgressosDadosDataControl.jsx # Seletor de branches e modo "Criar nova branch" vs "Enviar na branch selecionada"
│   └── pages/
│       └── src/
│           └── index.astro      # Habilita allowUpload={true} no SyncPanel do SRC
└── tests/
    └── unit/
        ├── api.test.js
        ├── SyncPanel.test.jsx
        └── EgressosDadosDataControl.test.jsx
```

**Structure Decision**: Arquitetura desktop Tauri 2.0 existente, estendendo `sync_domain.rs`, `sync_github.rs`, `commands.rs`, `api.js`, `SyncPanel.jsx`, `EgressosDadosDataControl.jsx` e `pages/src/index.astro` sem introduzir novas dependências.

## Complexity Tracking

Nenhuma violação da Constituição.
