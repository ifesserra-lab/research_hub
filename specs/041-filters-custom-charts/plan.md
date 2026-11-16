# Implementation Plan: Filtros Avançados nas Tabelas e Dashboards e Criação de Novos Gráficos Personalizados nas 5 Abas

**Branch**: `041-filters-custom-charts` | **Date**: 2026-11-16 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/041-filters-custom-charts/spec.md`

## Summary

Implementar nas **5 abas do ResearchHub** (**Horizon**, **Horizon Legacy**, **SRC**, **Egressos** e **Egressos — Dados**) um sistema unificado de **filtros estruturados por coluna** (com operadores `contains`, `equals`, `not_equals`, `is_empty`, `not_empty` e combinação lógica `AND` padrão ou `OR` selecionável) aplicável tanto às listagens paginadas das tabelas quanto ao recálculo dinâmico dos Dashboards Analíticos, além de permitir **criar, persistir e excluir novos gráficos personalizados** no Dashboard a partir de qualquer tabela, coluna de agrupamento, métrica (`count` Contagem de Registros ou `sum` Soma Numérica / Quantidade de Vínculos) e conjunto de filtros (visíveis tanto no painel "Ver Gráficos da Tabela" quanto no Dashboard Geral da aba).

## Technical Context

**Language/Version**: Rust 1.77+ (Edition 2021) no núcleo desktop; JavaScript/JSX (ES2022) com Astro 4.16 e React 18.3 no front-end  
**Primary Dependencies**: `tauri` 2.0, `rusqlite` (bundled), `serde` / `serde_json`, `astro`, `react`, `tailwindcss`  
**Storage**: SQLite local em arquivo único (`hub.db` no `app_data_dir` do SO) com nova migração `007_custom_dashboard_charts.sql` (`exported: false`)  
**Testing**: `cargo test` (unitários e integração em memória cobrindo filtros em tabelas, dashboards filtrados e CRUD/cálculo de gráficos personalizados nas 5 abas) e `Vitest` + Testing Library (`npm test` no front-end)  
**Target Platform**: Desktop Linux e Windows (Tauri 2.0)  
**Project Type**: `desktop-app` (Tauri 2.0 com front-end estático Astro/React embarcado)  
**Performance Goals**: Listagem filtrada de tabelas em <500ms (SC-001); recálculo de Dashboards filtrados e gráficos personalizados em <2s sobre 70.545 registros (SC-002)  
**Constraints**: Operação 100% local/offline; toda avaliação de filtros, agrupamento e agregação em Rust (`src-tauri/src/`); comunicação JS→Rust restrita exclusivamente a `frontend/src/services/api.js`; 0% de alteração nos arquivos de exportação das 5 abas (SC-004)  
**Scale/Scope**: 5 ambientes de projeto (Horizon, Horizon Legacy, SRC, Egressos com 21 tabelas, Egressos — Dados com 39 datasets), 39 tabelas SQLite locais

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **I. TDD Obrigatório**: **PASS** — todas as tasks seguirão a ordem red → green → refactor em `cargo test` e `npm test`.
- **II. Paridade Funcional Antes de Melhoria**: **PASS** — a nova tabela `custom_dashboard_charts` é estritamente interna (`exported: false`), mantendo intacta a paridade de round-trip de `exports_canonical.zip`, do JSON consolidado do SRC e das pastas de Egressos.
- **III. Testes Rigorosos (Rust e Front-end)**: **PASS** — cobertura automatizada em `cargo test` (migração `007`, motor de filtros `AND`/`OR` em colunas escalares e JSON, listagens filtradas nas 5 abas, dashboards filtrados e criação/listagem/exclusão de gráficos personalizados) e `Vitest` (`FilterBuilderBar`, `CustomChartModal`, `AnalyticsDashboard`, `TableAnalyticsPanel` e rotas em `api.js`).
- **IV. CRUD Completo e Operações Diretas**: **PASS** — todas as operações de filtro e gestão de gráficos personalizados são diretas pelo perfil Admin.
- **V. Aplicação Desktop Local e Autocontida**: **PASS** — persistência dos gráficos personalizados no mesmo arquivo SQLite `hub.db` via migração `007_custom_dashboard_charts.sql`.
- **VI. Lógica de Negócio em Rust**: **PASS** — o motor de avaliação de filtros (`FilterSpec` / `ColumnFilter`) e o agregador de gráficos personalizados (`CustomDashboardChart`) residem integralmente em `src-tauri/src/`, expostos via IPC em `frontend/src/services/api.js`.

## Project Structure

### Documentation (this feature)

```text
specs/041-filters-custom-charts/
├── plan.md              # Este arquivo (/speckit-plan)
├── research.md          # Decisões técnicas da Fase 0
├── data-model.md        # Esquema SQLite (007), gramática de filtros e estrutura de gráficos personalizados
├── quickstart.md        # Guia de validação e execução de testes
├── contracts/
│   └── ipc-commands.md  # Contratos Tauri IPC e rotas em api.js
└── tasks.md             # Tarefas ordenadas por TDD (/speckit-tasks)
```

### Source Code (repository root)

```text
src-tauri/
├── migrations/
│   └── 007_custom_dashboard_charts.sql  # NOVO: tabela interna custom_dashboard_charts
├── src/
│   ├── db.rs                            # Registra migração 007 em MIGRATIONS e valida schema
│   ├── analytics.rs                     # Motor FilterSpec/ColumnFilter, suporte a filtros nos 5 dashboards e CRUD/cálculo de CustomDashboardChart
│   ├── crud.rs                          # Suporte a filters: Option<&str> em list / hv2_list
│   ├── src_domain.rs                    # Suporte a filters: Option<&str> em list_acoes
│   ├── egressos_domain.rs               # Suporte a filters: Option<&str> em list_rows
│   ├── egressos_dados_domain.rs         # Suporte a filters: Option<&str> em list_rows
│   ├── commands.rs                      # Parâmetro filters nas listagens/analytics + comandos create_custom_chart, delete_custom_chart, list_custom_charts
│   └── lib.rs                           # Registro dos novos comandos no invoke_handler

frontend/
├── src/
│   ├── components/
│   │   ├── FilterBuilderBar.jsx         # NOVO: barra reutilizável de filtros por coluna (AND/OR, chips, limpar e botão "Criar Gráfico com estes Filtros")
│   │   ├── CustomChartModal.jsx         # NOVO: modal/construtor de novos gráficos personalizados (título, tabela, coluna group_by, métrica count/sum, tipo e filtros)
│   │   ├── AnalyticsDashboard.jsx       # Filtros interativos no Dashboard (barra + clique nas categorias), botão "+ Novo Gráfico" e exclusão de gráficos personalizados
│   │   ├── TableAnalyticsPanel.jsx      # Recebe activeFilters e repassa para o Dashboard da tabela e criador de gráficos
│   │   ├── EntityPage.jsx               # Integra FilterBuilderBar + criação de gráfico a partir dos filtros nas 15 tabelas do Horizon e Horizon Legacy
│   │   ├── SrcAcoesPage.jsx             # Integra FilterBuilderBar + criação de gráfico no SRC
│   │   ├── EgressosTablesPage.jsx       # Integra FilterBuilderBar + criação de gráfico nas 21 tabelas de Egressos
│   │   └── EgressosDadosTablesPage.jsx  # Integra FilterBuilderBar + criação de gráfico nas 39 tabelas de Egressos — Dados
│   └── services/
│       └── api.js                       # Query param filters nas listagens/analytics + rotas POST/GET/DELETE /custom-charts
└── tests/
    └── unit/
        ├── api.test.js                  # Testes das rotas com filters e /custom-charts
        ├── FilterBuilderBar.test.jsx    # NOVO: testes unitários da barra de filtros e atalho de criação de gráfico
        └── AnalyticsDashboard.test.jsx  # Testes de filtros no dashboard e criação/exclusão de gráficos personalizados
```

**Structure Decision**: Extensão limpa sobre `src-tauri/src/analytics.rs` e os módulos de listagem (`crud.rs`, `src_domain.rs`, `egressos_domain.rs`, `egressos_dados_domain.rs`), compartilhando um único avaliador de filtros em Rust e dois componentes React modulares (`FilterBuilderBar.jsx` e `CustomChartModal.jsx`) reutilizados nas 5 abas.

## Complexity Tracking

Nenhuma violação da Constituição. Tabela não aplicável.
