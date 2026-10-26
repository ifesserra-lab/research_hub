# Implementation Plan: Espaço de Dados do Projeto Egressos com CRUD e Sincronização GitHub

**Branch**: `035-egressos-data-workspace` | **Date**: 2026-10-22 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/035-egressos-data-workspace/spec.md`

## Summary

Adicionar suporte completo ao terceiro projeto institucional no Research Hub: **Egressos** (`/home/thiagodeps/egressos/dados`). Após o login, a tela `/projects` passa a oferecer **Horizon**, **SRC** e **Egressos** em espaços de dados totalmente isolados (`FR-001`..`FR-003`). Como o projeto Egressos armazena seus dados em uma **pasta de 21 arquivos JSON independentes** (sem um JSON unificado), o núcleo Rust ganha o módulo `egressos_domain.rs` e a migração aditiva `003_egressos_tables.sql` (`egressos_files` e `egressos_rows`), permitindo importar e exportar diretamente a pasta `dados/` com validação atômica, criação de snapshot `egressos-pre-import-*` e fidelidade *round-trip* de 100% nos 21 arquivos (`FR-004`..`FR-006`, `FR-010`, `SC-003`). Na área dedicada (`/egressos` e `/egressos/tabelas`), o curador dispõe de navegação 1:1 para as **21 abas principais** (com sub-abas internas para arquivos com múltiplas listas e editor de metadados de arquivo), busca, ordenação, CRUD completo e opção de escolha na interface para aplicar edições/exclusões apenas na tabela atual ou propagá-las por chave compartilhada (`rotulo`/`label` de egresso ou nome de empresa) para as tabelas relacionadas (`FR-007`..`FR-009`, `FR-008a`). Por fim, a sincronização GitHub por token (`sync_domain.rs`, `sync_github.rs` e `SyncPanel.jsx`) é estendida para o projeto `"egressos"` com download e upload habilitados diretamente sobre a pasta `dados/` do repositório remoto em um único commit atômico via Git Data API (`FR-011`..`FR-013`).

## Technical Context

**Language/Version**: Rust (edition 2021, Tauri 2.x) no núcleo + JavaScript/React sobre Astro (build estático embarcado) no front-end.

**Primary Dependencies**: Existentes no `Cargo.toml` — `rusqlite 0.32` (bundled), `serde` / `serde_json 1.0` (com `preserve_order`), `tauri_plugin_dialog` (seleção nativa de pasta `blocking_pick_folder`), `reqwest` (rustls) e `wiremock` (em `dev-dependencies` para testes HTTP sem rede real). Nenhuma dependência externa nova é necessária.

**Storage**: SQLite em arquivo único (`hub.db` no diretório `app_data_dir`), acrescido da migração `003_egressos_tables.sql` (`egressos_files` + `egressos_rows`), mais `sync_config.json` (já existente, estendido com a chave de projeto `"egressos"`).

**Testing**: `cargo test` (testes unitários e de integração em SQLite em memória em `egressos_domain.rs` e `db.rs` + testes de sincronização GitHub de diretório contra `wiremock` em `tests/sync_github.rs` + teste de resolução de rotas `ROUTES` em `lib.rs`) e `npm test` (Vitest para `api.js`, `ProjectSelector`, `SyncPanel`, `EgressosDataControl` e `EgressosTablesPage`).

**Target Platform**: Desktop Linux e Windows (Tauri 2.0).

**Project Type**: Aplicação desktop autocontida de processo único (`src-tauri/` + `frontend/`).

**Performance Goals**: Importação local dos 21 arquivos JSON de `/home/thiagodeps/egressos/dados` em < 10 s (`SC-002`); troca de projeto em < 5 s (`SC-001`); download/upload dos 21 arquivos JSON no GitHub em < 30 s em conexão padrão (`SC-005`).

**Constraints**: Isolamento absoluto entre Horizon, SRC e Egressos (`FR-002`, `FR-003`); validação atômica dos 21 arquivos antes de alterar a base (`FR-005`); fidelidade *round-trip* de 100% nos 21 arquivos JSON (`SC-003`); token GitHub jamais exposto em mensagens, logs ou erros (`FR-013`).

**Scale/Scope**: 2 tabelas SQLite novas (`egressos_files`, `egressos_rows`), 21 definições de arquivos/abas no catálogo `EGRESSOS_FILES` (~1.140 linhas somadas nas seções de `/home/thiagodeps/egressos/dados`), 10 comandos IPC novos (`import_egressos_dir`, `export_egressos_dir`, `egressos_get_catalog`, `egressos_list_rows`, `egressos_get_row`, `egressos_create_row`, `egressos_update_row`, `egressos_delete_row`, `egressos_get_envelope`, `egressos_update_envelope`) + suporte a `"egressos"` nos comandos `github_*`, 2 páginas Astro novas (`/egressos` e `/egressos/tabelas`), 1 layout novo (`EgressosDashboard.astro`) e 2 componentes React novos (`EgressosDataControl.jsx` e `EgressosTablesPage.jsx`).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Princípio | Status (pré-Phase 0) | Evidência / Justificativa | Re-check pós-Phase 1 |
|-----------|----------------------|---------------------------|----------------------|
| I. TDD Obrigatório | ✅ PASS | Cada funcionalidade (migração `003`, import/export da pasta `dados/`, *round-trip* dos 21 JSONs, CRUD + propagação opcional, sync GitHub de pasta e componentes UI) terá testes escritos antes da implementação. | ✅ Mantido — ver `quickstart.md` §2. |
| II. Paridade Funcional Antes de Melhoria | ✅ PASS | Os fluxos do Horizon e do SRC permanecem 100% intocados (`FR-003`), e o oráculo do Egressos são os próprios 21 arquivos JSON reais de `/home/thiagodeps/egressos/dados` verificados por teste automatizado de *round-trip* (`SC-003`). | ✅ Mantido — contrato `C5` em `contracts/egressos-dados-json.md`. |
| III. Testes Rigorosos (Rust e Front-end) | ✅ PASS | Domínio Egressos testado com `cargo test` em SQLite em memória (sem depender de janela Tauri); sincronização GitHub de diretório testada contra `wiremock` local; front-end testado com Vitest (`npm test`) e resolução de rotas verificada em `lib.rs`. | ✅ Mantido. |
| IV. CRUD Completo e Operações Diretas | ✅ PASS | Todas as 21 abas/arquivos JSON possuem CRUD completo (criar, visualizar, editar, excluir registros + editar metadados/sub-seções), além de opção direta de propagação entre tabelas relacionadas (`FR-008`, `FR-008a`). | ✅ Mantido. |
| V. Aplicação Desktop Local e Autocontida | ✅ PASS | Persistência no mesmo arquivo SQLite local (`hub.db`); operação 100% funcional offline com a pasta local `/home/thiagodeps/egressos/dados`; sincronização HTTPS com `api.github.com` parte diretamente do processo Rust sob ação explícita do curador. | ✅ Mantido. |
| VI. Lógica de Negócio em Rust | ✅ PASS | Catálogo dos 21 arquivos, validação atômica, decomposição/reconstrução JSON, CRUD, propagação cruzada e cliente GitHub vivem inteiramente em Rust (`egressos_domain.rs`, `sync_domain.rs`, `sync_github.rs`); o front-end comunica-se exclusivamente via `frontend/src/services/api.js`. | ✅ Mantido. |

Nenhuma violação da Constituição — tabela de Complexity Tracking vazia.

## Project Structure

### Documentation (this feature)

```text
specs/035-egressos-data-workspace/
├── plan.md                         # Este arquivo (/speckit-plan)
├── research.md                     # Phase 0 output
├── data-model.md                   # Phase 1 output
├── quickstart.md                   # Phase 1 output
├── contracts/                      # Phase 1 output
│   ├── egressos-dados-json.md      # Contrato dos 21 arquivos JSON da pasta dados/
│   ├── ipc-commands.md             # Comandos IPC Tauri novos e extensão github_*
│   └── frontend-routes.md          # Páginas Astro e rotas em api.js
└── tasks.md                        # Phase 2 output (/speckit-tasks — gerado na próxima etapa)
```

### Source Code (repository root)

```text
src-tauri/
├── migrations/
│   └── 003_egressos_tables.sql     # NOVO: tabelas egressos_files e egressos_rows + índices
├── src/
│   ├── egressos_domain.rs          # NOVO: catálogo dos 21 arquivos JSON, validação atômica,
│   │                               #   import_dir/export_dir, round-trip sem perdas, CRUD por
│   │                               #   aba/seção e propagação opcional (Alumni / Company)
│   ├── sync_domain.rs              # MODIFICAR: incluir "egressos" em PROJECTS, check_upload_allowed,
│   │                               #   validate_path para diretório e parsing de URL de pasta GitHub
│   ├── sync_github.rs              # MODIFICAR: download_egressos_dir (21 JSONs via Contents API) e
│   │                               #   upload_directory (21 blobs + 1 tree + 1 commit via Git Data API)
│   ├── db.rs                       # MODIFICAR: registrar 003_egressos_tables.sql em MIGRATIONS
│   ├── commands.rs                 # MODIFICAR: registrar comandos egressos_* e ramificação "egressos" em github_*
│   └── lib.rs                      # MODIFICAR: pub mod egressos_domain; invoke_handler; ROUTES (21 → 23)
└── tests/
    └── sync_github.rs              # MODIFICAR: testes wiremock para download/upload da pasta dados/ do Egressos

frontend/
├── src/
│   ├── pages/
│   │   ├── projects.astro          # MODIFICAR: incluir cartão Egressos (/egressos) e listener egressos:dirty
│   │   └── egressos/
│   │       ├── index.astro         # NOVO: página de resumo + import/export pasta local + SyncPanel GitHub
│   │       └── tabelas.astro       # NOVO: página de curadoria das 21 abas de tabelas do Egressos
│   ├── layouts/
│   │   └── EgressosDashboard.astro # NOVO: layout dedicado da área Egressos com menu lateral (Resumo + 21 abas)
│   ├── components/
│   │   ├── EgressosDataControl.jsx # NOVO: import/export da pasta dados/ com atalho para /home/thiagodeps/egressos/dados
│   │   ├── EgressosTablesPage.jsx  # NOVO: 21 abas + sub-abas de seção + busca/ordenação + CRUD + opção de propagação + metadados
│   │   ├── ProjectSelector.jsx     # MODIFICAR: ajustar grid para acomodar 3 projetos
│   │   └── SyncPanel.jsx           # MODIFICAR: suporte a modo diretório e mensagens/defaults do projeto "egressos"
│   └── services/
│       └── api.js                  # MODIFICAR: rotas /egressos/* (único ponto de acoplamento JS→Rust)
└── tests/
    └── unit/                       # MODIFICAR/NOVO: testes Vitest para api.js, ProjectSelector, SyncPanel e componentes Egressos
```

**Structure Decision**: Segue estritamente a arquitetura estabelecida em `033-src-data-tab` e `034-github-sync`: lógica de domínio pura e testável sem Tauri em `src-tauri/src/egressos_domain.rs`, migração incremental em `src-tauri/migrations/003_egressos_tables.sql`, extensão limpa de `sync_domain.rs`/`sync_github.rs` e fronteira IPC única em `frontend/src/services/api.js`.

## Complexity Tracking

> Nenhuma violação da Constituição — tabela vazia.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| — | — | — |
