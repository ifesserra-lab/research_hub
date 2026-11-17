# Implementation Plan: Aba Horizon (Nova Versão Canônica) e Preservação do Horizon Legacy

**Branch**: `039-horizon-v2-canonical-tab` | **Date**: 2026-11-12 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/039-horizon-v2-canonical-tab/spec.md`

## Summary

Renomear o projeto existente do Horizon na seleção de projetos para **Horizon Legacy** (preservando intactos seus componentes, rotas `/dashboard/*`, tabelas SQLite originais, arquivo `original.zip`, snapshots `hub-*` e configuração de sincronização `"horizon"`) e criar uma nova aba de projeto **Horizon** (`/horizon` e `/horizon/*`) isolada em banco (`hv2_*`), arquivo de referência (`horizon_v2_original.zip`), snapshots (`horizon-v2-*`) e sincronização GitHub (`"horizon_v2"`), adaptada ao novo pacote `exports_canonical.zip` (698 entradas, 70.615 registros nas 15 tabelas canônicas, preservação byte a byte dos 683 arquivos complementares e exposição completa de todas as colunas canônicas nos formulários, incluindo o objeto estruturado `enrichment` em Iniciativas).

## Technical Context

**Language/Version**: Rust 1.77+ (Edition 2021) no núcleo desktop; JavaScript/JSX (ES2022) com Astro 5 e React 19 no front-end  
**Primary Dependencies**: `tauri` 2.0, `rusqlite` (bundled), `serde` / `serde_json`, `zip`, `reqwest`, `astro`, `react`, `tailwindcss`  
**Storage**: SQLite local em arquivo único (`hub.db` no `app_data_dir` do SO) + arquivos de referência `original.zip` (Horizon Legacy) e `horizon_v2_original.zip` (novo Horizon)  
**Testing**: `cargo test` (unitários e integração em memória + pacote real `exports_canonical.zip`) e `Vitest` + Testing Library (`npm test` no front-end)  
**Target Platform**: Desktop Linux e Windows (Tauri 2.0)  
**Project Type**: `desktop-app` (Tauri 2.0 com front-end estático Astro/React embarcado)  
**Performance Goals**: Importação e round-trip de exportação das 15 tabelas canônicas (70.615 linhas e 698 entradas no ZIP) em menos de 30 segundos em hardware padrão; listagem paginada e busca em <50ms  
**Constraints**: Operação 100% local/offline para curadoria e import/export; toda regra de negócio e SQL em Rust; comunicação JS→Rust restrita a `frontend/src/services/api.js`  
**Scale/Scope**: 15 tabelas canônicas em cada ambiente Horizon (Legacy e Novo), 70.615 registros no pacote de referência, 698 entradas no arquivo `.zip` (58,5 MB)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. TDD Obrigatório**: **PASS** — todas as tasks seguirão a ordem red → green → refactor em `cargo test` e `npm test`.
- **II. Paridade Funcional Antes de Melhoria**: **PASS** — o ambiente **Horizon Legacy** é mantido integralmente compatível e inalterado em suas regras, enquanto o novo **Horizon** preserva a fidelidade de round-trip do novo `exports_canonical.zip` (698 entradas, tipos JSON restaurados e 683 entradas preservadas byte a byte).
- **III. Testes Rigorosos (Rust e Front-end)**: **PASS** — cobertura em `cargo test` (migração `005`, registry, CRUD isolado, fusão/vínculo, import/export com `exports_canonical.zip` atualizado, sync GitHub e resolução de rotas Tauri) e `Vitest` (`ProjectSelector`, `api.js`, `EntityForm`/`EntityPage` e `DataControlCenter`).
- **IV. CRUD Completo e Operações Diretas**: **PASS** — todas as 15 entidades na nova aba **Horizon** possuem CRUD completo, busca textual/ID, ordenação, fusão e vínculo diretos pelo perfil Admin.
- **V. Aplicação Desktop Local e Autocontida**: **PASS** — todas as novas tabelas (`hv2_*`) residem no mesmo arquivo SQLite embarcado `hub.db` via migração `005_horizon_v2_tables.sql`, sem servidor externo.
- **VI. Lógica de Negócio em Rust**: **PASS** — SQL, leitura/escrita de ZIP e sincronização permanecem 100% em `src-tauri/src/`, e toda chamada JS→Rust passa exclusivamente por `frontend/src/services/api.js`.

## Project Structure

### Documentation (this feature)

```text
specs/039-horizon-v2-canonical-tab/
├── plan.md              # Este arquivo (/speckit-plan)
├── research.md          # Decisões técnicas da Fase 0
├── data-model.md        # Modelo de dados SQLite (hv2_*) e estrutura do novo ZIP
├── quickstart.md        # Guia de validação e execução de testes
├── contracts/
│   └── ipc-commands.md  # Contratos Tauri IPC e rotas em api.js
└── tasks.md             # Tarefas ordenadas por TDD (/speckit-tasks)
```

### Source Code (repository root)

```text
src-tauri/
├── migrations/
│   ├── 001_init.sql                  # Tabelas originais (Horizon Legacy + admins)
│   └── 005_horizon_v2_tables.sql     # NOVO: 15 tabelas isoladas (hv2_*) para a nova aba Horizon
├── src/
│   ├── registry.rs                   # HORIZON_V2_ENTITIES + by_horizon_v2_route + canonical_table()
│   ├── db.rs                         # Registro da migração 005 + testes de schema
│   ├── crud.rs                       # Funções CRUD parametrizadas por catálogo (ENTITIES vs HORIZON_V2_ENTITIES)
│   ├── special.rs                    # Merge e link parametrizados por catálogo (hv2_merge, hv2_link)
│   ├── import.rs                     # Importação parametrizada (horizon_v2_original.zip, prefixo horizon-v2) + testes atualizados
│   ├── export.rs                     # Exportação parametrizada + testes atualizados para o novo exports_canonical.zip (698 entradas)
│   ├── sync_domain.rs                # Inclusão do projeto "horizon_v2" em PROJECTS e validações .zip
│   ├── sync_github.rs                # Suporte a "horizon_v2" em download_and_import
│   ├── commands.rs                   # Comandos hv2_* (CRUD, merge, link, import_horizon_v2_zip, export_horizon_v2_zip, github_upload)
│   └── lib.rs                        # Registro dos novos comandos no invoke_handler e rotas /horizon/* em ROUTES
└── tests/
    └── sync_github.rs                # Testes de sincronização GitHub para "horizon_v2"

frontend/
├── src/
│   ├── components/
│   │   ├── ProjectSelector.jsx       # Novo card "Horizon" (/horizon) + renomeação do card antigo para "Horizon Legacy" (/dashboard)
│   │   ├── DataControlCenter.jsx     # Suporte a props opcionais para operar em /horizon-v2 (mantendo defaults do Legacy)
│   │   ├── EntityPage.jsx            # Suporte a apiPrefix="/horizon-v2" e basePath="/horizon" (mantendo defaults do Legacy)
│   │   └── EntityForm.jsx            # Renderização legível de objetos estruturados (enrichment) e links com basePath
│   ├── layouts/
│   │   ├── Dashboard.astro           # Layout do Horizon Legacy (/dashboard/*)
│   │   └── HorizonDashboard.astro    # NOVO: Layout da nova aba Horizon (/horizon/*)
│   ├── pages/
│   │   ├── projects.astro            # Lista de projetos atualizada com "Horizon" e "Horizon Legacy"
│   │   ├── dashboard/                # 16 páginas existentes do Horizon Legacy (intactas)
│   │   └── horizon/                  # NOVO: index.astro + 15 páginas das entidades canônicas com todos os campos expostos
│   └── services/
│       └── api.js                    # Mapeamento das rotas /horizon-v2/* e listener onHorizonV2ImportProgress
└── tests/
    └── unit/
        ├── api.test.js               # Testes das rotas /horizon-v2/*
        └── ProjectSelector.test.jsx  # Testes dos cards "Horizon" e "Horizon Legacy"
```

**Structure Decision**: Mantida a arquitetura desktop Tauri 2.0 (`src-tauri/` + `frontend/`), adicionando a migração `005_horizon_v2_tables.sql`, parametrizando os módulos de domínio existentes em `src-tauri/src/` para evitar duplicação de lógica e criando o conjunto de páginas `/horizon/*` no Astro reutilizando os componentes React existentes parametrizados por prefixo de rota/API.

## Complexity Tracking

Nenhuma violação da Constituição. Tabela não aplicável.
