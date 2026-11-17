# Implementation Plan: Adaptação Final do Pacote Canônico Horizon, Indicador de Status de Projetos e Dashboards Analíticos em Todas as Abas

**Branch**: `040-horizon-analytics-dashboard` | **Date**: 2026-11-16 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/040-horizon-analytics-dashboard/spec.md`

## Summary

Adaptar o ResearchHub à versão final do pacote `exports_canonical.zip` (70.545 registros nas 15 tabelas canônicas, 698 entradas no ZIP e inclusão da 22ª coluna `linkedin_url` em `researchers` e `students`), adicionar indicativos visuais semânticos de **Ativo / Inativo** tanto nos registros de projetos (`initiatives` e `advisorships` nas listagens e formulários do Horizon e Horizon Legacy) quanto nos cartões dos 5 ambientes na tela de seleção de projetos (`/projects`), e disponibilizar **Dashboards Analíticos com Gráficos** em todas as abas do sistema (visão geral consolidada nos 5 projetos — **Horizon**, **Horizon Legacy**, **SRC**, **Egressos** e **Egressos — Dados** — mais painel contextual **"Ver Gráficos da Tabela"** em cada aba de tabela individual), com toda a agregação estatística processada em Rust sobre o SQLite local.

## Technical Context

**Language/Version**: Rust 1.77+ (Edition 2021) no núcleo desktop; JavaScript/JSX (ES2022) com Astro 4.16 e React 18.3 no front-end  
**Primary Dependencies**: `tauri` 2.0, `rusqlite` (bundled), `serde` / `serde_json`, `zip`, `astro`, `react`, `tailwindcss` (gráficos renderizados em SVG/Tailwind nativos sem dependências externas de canvas)  
**Storage**: SQLite local em arquivo único (`hub.db` no `app_data_dir` do SO) com migração `006_person_linkedin_url.sql` + arquivos de referência `original.zip` e `horizon_v2_original.zip`  
**Testing**: `cargo test` (unitários e integração em memória + validação contra o `exports_canonical.zip` final) e `Vitest` + Testing Library (`npm test` no front-end)  
**Target Platform**: Desktop Linux e Windows (Tauri 2.0)  
**Project Type**: `desktop-app` (Tauri 2.0 com front-end estático Astro/React embarcado)  
**Performance Goals**: Agregação analítica completa das 15 tabelas canônicas (70.545 linhas) e das tabelas SRC/Egressos em <500ms em Rust (entrega e renderização total em <2s conforme SC-003); round-trip de importação/exportação em <30s  
**Constraints**: Operação 100% local/offline; toda regra de negócio, SQL e agregação estatística em Rust (`src-tauri/src/`); comunicação JS→Rust restrita exclusivamente a `frontend/src/services/api.js`; metadados de entidades centralizados em `src-tauri/src/registry.rs`  
**Scale/Scope**: 5 ambientes de projeto (Horizon, Horizon Legacy, SRC, Egressos com 21 tabelas, Egressos — Dados com 39 tabelas), 38 tabelas SQLite existentes, 70.545 registros no pacote canônico de referência (698 arquivos no ZIP)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. TDD Obrigatório**: **PASS** — todas as tasks seguirão estritamente a ordem red → green → refactor em `cargo test` e `npm test`.
- **II. Paridade Funcional Antes de Melhoria**: **PASS** — a migração `006_person_linkedin_url.sql` e a atualização de `PERSON_COLUMNS` em `registry.rs` garantem fidelidade 100% no round-trip da versão final de `exports_canonical.zip` (22 colunas em `researchers` e `students`, 70.545 linhas e 683 entradas não gerenciadas preservadas byte a byte).
- **III. Testes Rigorosos (Rust e Front-end)**: **PASS** — cobertura automatizada em `cargo test` (migração `006`, registry, import/export com a versão final do ZIP, agregações analíticas para os 5 projetos e por tabela, status de projetos) e `Vitest` (`ProjectStatusBadge`, `ProjectSelector`, `EntityTable`, `EntityForm`, `AnalyticsDashboard`, `TableAnalyticsPanel` e rotas em `api.js`).
- **IV. CRUD Completo e Operações Diretas**: **PASS** — o novo campo `linkedin_url` é integrado ao CRUD completo de `researchers` e `students`; todas as operações diretas permanecem sob controle do perfil Admin.
- **V. Aplicação Desktop Local e Autocontida**: **PASS** — os gráficos utilizam componentes SVG/Tailwind leves embarcados no bundle estático e dados agregados localmente no SQLite, sem rede e sem serviços externos.
- **VI. Lógica de Negócio em Rust**: **PASS** — toda contagem, classificação de status, cálculo financeiro de bolsas (orientadores, bolsistas e programas), higienização de instituições e agregação por tabela residem em `src-tauri/src/analytics.rs`; o front-end apenas invoca comandos via `frontend/src/services/api.js` e renderiza os resultados.

## Project Structure

### Documentation (this feature)

```text
specs/040-horizon-analytics-dashboard/
├── plan.md              # Este arquivo (/speckit-plan)
├── research.md          # Decisões técnicas da Fase 0
├── data-model.md        # Esquema atualizado (006), estruturas analíticas e regras de status
├── quickstart.md        # Guia de validação e execução de testes
├── contracts/
│   └── ipc-commands.md  # Contratos Tauri IPC e rotas em api.js
└── tasks.md             # Tarefas ordenadas por TDD (/speckit-tasks)
```

### Source Code (repository root)

```text
src-tauri/
├── migrations/
│   ├── 005_horizon_v2_tables.sql        # Tabelas hv2_* existentes
│   └── 006_person_linkedin_url.sql      # NOVO: adiciona linkedin_url em researchers, students, hv2_researchers e hv2_students
├── src/
│   ├── registry.rs                      # Adiciona "linkedin_url" em PERSON_COLUMNS (22 colunas)
│   ├── db.rs                            # Registra migração 006 em MIGRATIONS e valida schema
│   ├── import.rs                        # Atualiza contagens esperadas da versão final do exports_canonical.zip (70.545 linhas)
│   ├── export.rs                        # Testes de round-trip validando as 22 colunas de researchers/students
│   ├── analytics.rs                     # NOVO: agregações de status de projetos, Dashboards Gerais dos 5 projetos e Gráficos por Tabela
│   ├── commands.rs                      # Novos comandos IPC: projects_status, horizon_dashboard, horizon_entity_analytics, src_analytics, egressos_analytics, egressos_dados_analytics
│   └── lib.rs                           # Registro do módulo analytics e dos novos comandos no invoke_handler

frontend/
├── src/
│   ├── components/
│   │   ├── ProjectStatusBadge.jsx       # NOVO: selo semântico Ativo / Inativo para status de projetos e cartões
│   │   ├── AnalyticsDashboard.jsx       # NOVO: Dashboard Analítico Geral parametrizado para os 5 projetos (KPIs + gráficos SVG/Tailwind)
│   │   ├── TableAnalyticsPanel.jsx      # NOVO: painel expansível "Ver Gráficos da Tabela" para qualquer tabela individual
│   │   ├── ProjectSelector.jsx          # Exibe selo Ativo/Sem dados e contagem de registros em cada um dos 5 cartões de /projects
│   │   ├── EntityTable.jsx              # Renderiza ProjectStatusBadge na coluna status
│   │   ├── EntityForm.jsx               # Renderiza ProjectStatusBadge ao lado do campo status
│   │   ├── EntityPage.jsx               # Inclui botão/painel "Ver Gráficos da Tabela" nas 15 abas do Horizon e Horizon Legacy
│   │   ├── SrcAcoesWorkspace.jsx        # Inclui botão/painel "Ver Gráficos da Tabela" na aba de Ações do SRC
│   │   ├── EgressosWorkspace.jsx        # Inclui botão/painel "Ver Gráficos da Tabela" nas 21 tabelas de Egressos
│   │   └── EgressosDadosWorkspace.jsx   # Inclui Dashboard Geral + "Ver Gráficos da Tabela" nas 39 tabelas de Egressos — Dados
│   ├── pages/
│   │   ├── projects.astro               # Carrega /projects/status e repassa para ProjectSelector
│   │   ├── horizon/
│   │   │   ├── index.astro              # Inclui AnalyticsDashboard (project="horizon-v2")
│   │   │   ├── researchers.astro        # Adiciona campo linkedin_url (22 campos)
│   │   │   └── students.astro           # Adiciona campo linkedin_url (22 campos)
│   │   ├── dashboard/
│   │   │   ├── index.astro              # Inclui AnalyticsDashboard (project="horizon")
│   │   │   ├── researchers.astro        # Adiciona campo linkedin_url
│   │   │   └── students.astro           # Adiciona campo linkedin_url
│   │   ├── src/
│   │   │   └── index.astro              # Inclui AnalyticsDashboard (project="src")
│   │   └── egressos/
│   │       └── index.astro              # Inclui AnalyticsDashboard (project="egressos")
│   └── services/
│       └── api.js                       # Novas rotas GET de analytics e /projects/status
└── tests/
    └── unit/
        ├── api.test.js                  # Testes das novas rotas de analytics e status
        ├── ProjectSelector.test.jsx     # Testes do selo de projeto ativo/inativo nos cartões
        ├── ProjectStatusBadge.test.jsx  # NOVO: testes unitários do badge de status de projeto
        └── AnalyticsDashboard.test.jsx  # NOVO: testes unitários dos dashboards e gráficos por tabela
```

**Structure Decision**: Arquitetura Tauri 2.0 mantida rigorosamente fiel aos princípios do repositório: um novo módulo `src-tauri/src/analytics.rs` concentra toda a inteligência estatística e consultas SQL sobre as 38 tabelas locais, exposta por comandos Tauri mapeados exclusivamente em `frontend/src/services/api.js`, enquanto componentes React reutilizáveis (`ProjectStatusBadge`, `AnalyticsDashboard` e `TableAnalyticsPanel`) atendem os 5 projetos e todas as abas de tabelas sem duplicação.

## Complexity Tracking

Nenhuma violação da Constituição. Tabela não aplicável.
