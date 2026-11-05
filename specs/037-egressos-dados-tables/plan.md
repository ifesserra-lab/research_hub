# Implementation Plan: Tabelas, CRUD e Sincronização do Repositório `egressos-dados` (`alunos.json` e `data/*.json`)

**Branch**: `037-egressos-dados-tables` | **Date**: 2026-11-05 | **Spec**: [spec.md](file:///home/thiagodeps/research_hub/specs/037-egressos-dados-tables/spec.md)  
**Input**: Feature specification from `/specs/037-egressos-dados-tables/spec.md`

## Summary

Implementar na área restrita **Egressos — Dados** (`/egressos-dados` e `/egressos-dados/controle`, protegida pela verificação de membro ativo da organização GitHub `ifesserra-lab`) o suporte completo às **39 tabelas JSON** do repositório `/home/thiagodeps/egressos-dados` (`alunos.json` na raiz + 38 arquivos `.json` na pasta `data/`). A solução adiciona:
1. Migração SQLite `004_egressos_dados_tables.sql` criando `egressos_dados_files` e `egressos_dados_rows` isoladas dos demais projetos.
2. Módulo de domínio `src-tauri/src/egressos_dados_domain.rs` com o catálogo `EGRESSOS_DADOS_FILES` (39 datasets com `relative_path`, `publication` `pii`/`interno`/`publicavel` e `role` `entrada`/`saida`), suporte aos 4 formatos estruturais (`object_sections`, `root_array`, `root_map`, `nested_map`), preservação do campo `"id"` textual do domínio (usando `"_row_id"` para o ID interno do SQLite), CRUD completo, propagação opcional (`Alumni` e `Company`) e importação/exportação *round-trip* de `/home/thiagodeps/egressos-dados`.
3. Sincronização GitHub de diretório raiz + `data/` (`egressos_dados_sync_*`) reutilizando automaticamente o token validado em memória de `EgressosDadosAuth` contra `ifesserra-lab/egressos-dados`.
4. Interface em duas páginas no menu lateral (`/egressos-dados` com `EgressosDadosTablesPage.jsx` e filtros por categoria `Todos`/`PII`/`Internos`/`Publicáveis`, e `/egressos-dados/controle` com `EgressosDadosDataControl.jsx`), ambas envoltas pelo gate de autenticação `EgressosDadosWorkspace.jsx`.

## Technical Context

**Language/Version**: Rust 2021 (stable) no backend Tauri 2.0; JavaScript (ES2022 / Astro 5 / React 18) no front-end  
**Primary Dependencies**: `tauri` 2, `rusqlite` 0.31 (`bundled`), `serde` / `serde_json` (com `preserve_order`), `reqwest` 0.12 (`blocking`, `json`, `rustls-tls`), `sha1`, `tempfile`; React 18, Astro 5, Tailwind CSS 3  
**Storage**: SQLite embarcado (`hub.db`) com 2 novas tabelas dedicadas (`egressos_dados_files` e `egressos_dados_rows`); token do GitHub mantido exclusivamente em memória (`AppState.egressos_dados_auth`)  
**Testing**: `cargo test` (testes unitários em `db.rs` e `egressos_dados_domain.rs` contra `/home/thiagodeps/egressos-dados` e fixtures sintéticas + testes de integração HTTP com `wiremock` em `src-tauri/tests/sync_github.rs`) e `npm test` (`vitest` + `@testing-library/react`)  
**Target Platform**: Desktop Linux e Windows (Tauri 2.0)  
**Project Type**: `desktop-app` (Tauri 2.0 Rust Core + Astro/React Static Frontend)  
**Performance Goals**: Importação e exportação local dos 39 arquivos JSON (`alunos.json` + 38 em `data/*.json`) em < 5s (meta da spec: < 10s); listagem, filtragem e CRUD de linhas em < 50ms  
**Constraints**: Sem servidor HTTP; toda comunicação JS→Rust via `frontend/src/services/api.js`; todas as rotas de `egressos-dados` exigem `require_egressos_dados_session()`  
**Scale/Scope**: 39 arquivos JSON (~3.500 registros somados em mais de 95 seções), 13 novos comandos Tauri IPC, 2 páginas Astro (`/egressos-dados` e `/egressos-dados/controle`)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. TDD Obrigatório**: **PASS** — As tasks exigirão escrever primeiro os testes (vermelho) para a migração `004`, o parser/round-trip dos 39 arquivos, o CRUD, a sincronização GitHub e os componentes React antes da implementação (verde).
- **II. Paridade Funcional Antes de Melhoria**: **PASS** — O teste de *round-trip* verifica que importar os 39 arquivos de `/home/thiagodeps/egressos-dados` e exportá-los produz JSONs 100% equivalentes semanticamente aos originais do repositório Python `egressos-dados`.
- **III. Testes Rigorosos (Rust e Front-end)**: **PASS** — Cobertura automatizada em `cargo test` e `npm test` (Vitest) sem depender de arquivos externos fora do repositório quando rodar em CI (usando gerador sintético dos 39 arquivos nos testes e validando também contra `/home/thiagodeps/egressos-dados` quando presente localmente).
- **IV. CRUD Completo e Operações Diretas**: **PASS** — Todas as 39 tabelas (`entrada` e `saida`) possuem CRUD completo (criar, visualizar, editar, excluir) e edição de metadados de arquivo.
- **V. Aplicação Desktop Local e Autocontida**: **PASS** — Armazenamento local em SQLite (`hub.db`) via migração `004_egressos_dados_tables.sql`; importação/exportação local e CRUD funcionam 100% offline após o desbloqueio da sessão.
- **VI. Lógica de Negócio em Rust**: **PASS** — Catálogo dos 39 datasets, decomposição/reconstrução JSON, validação, busca, ordenação, propagação e sincronização Git Data API residem integralmente em `src-tauri/src/`; o front-end comunica-se exclusivamente via `frontend/src/services/api.js`.

## Project Structure

### Documentation (this feature)

```text
specs/037-egressos-dados-tables/
├── plan.md              # Este arquivo (/speckit-plan)
├── research.md          # Phase 0 output (/speckit-plan)
├── data-model.md        # Phase 1 output (/speckit-plan)
├── quickstart.md        # Phase 1 output (/speckit-plan)
├── contracts/           # Phase 1 output (/speckit-plan)
│   ├── ipc-commands.md
│   └── frontend-routes.md
└── tasks.md             # Phase 2 output (/speckit-tasks)
```

### Source Code (repository root)

```text
src-tauri/
├── migrations/
│   └── 004_egressos_dados_tables.sql # Novas tabelas egressos_dados_files e egressos_dados_rows
├── src/
│   ├── db.rs                         # Registro da migração 004 + testes de schema (23 tabelas)
│   ├── egressos_dados_domain.rs      # Novo módulo: catálogo de 39 datasets, round-trip 4 kinds, CRUD e propagação
│   ├── sync_domain.rs                # ProjectId::EgressosDados + defaults (ifesserra-lab/egressos-dados, main, .)
│   ├── sync_github.rs                # download_egressos_dados_repo e upload_egressos_dados_repo (alunos.json + data/*.json)
│   ├── commands.rs                   # Comandos IPC egressos_dados_* protegidos por require_egressos_dados_session
│   └── lib.rs                        # Registro dos novos comandos IPC e da rota /egressos-dados/controle (25 rotas)
└── tests/
    └── sync_github.rs                # Testes wiremock para download/upload dos 39 arquivos de egressos-dados

frontend/
├── src/
│   ├── services/
│   │   └── api.js                    # Rotas /egressos-dados/catalog, /files/*, /rows/*, /import, /export, /snapshots, /sync/*
│   ├── components/
│   │   ├── EgressosDadosWorkspace.jsx   # Gate de autenticação ifesserra-lab envolvendo children (Tabelas ou Controle)
│   │   ├── EgressosDadosTablesPage.jsx  # Navegação nas 39 tabelas com filtro de categoria (PII/Interno/Publicável) e CRUD
│   │   └── EgressosDadosDataControl.jsx # Import/export local (/home/thiagodeps/egressos-dados), snapshots e sync GitHub
│   ├── layouts/
│   │   └── EgressosDadosDashboard.astro # Menu lateral com Tabelas (/egressos-dados) e Controle de Dados (/egressos-dados/controle)
│   └── pages/
│       └── egressos-dados/
│           ├── index.astro              # Página principal das 39 tabelas protegida pelo gate
│           └── controle.astro           # Página de controle de dados e sync protegida pelo gate
└── tests/
    └── unit/
        ├── api.test.js
        ├── EgressosDadosWorkspace.test.jsx
        ├── EgressosDadosTablesPage.test.jsx
        └── EgressosDadosDataControl.test.jsx
```

**Structure Decision**: Segue a arquitetura desktop Tauri + Astro/React do repositório, adicionando `egressos_dados_domain.rs` e `004_egressos_dados_tables.sql` no núcleo Rust e os componentes `EgressosDadosTablesPage.jsx` e `EgressosDadosDataControl.jsx` no front-end, acoplados exclusivamente por `frontend/src/services/api.js`.

## Complexity Tracking

Nenhuma violação da Constituição — tabela não aplicável.
