---
description: "Task list for feature 041-filters-custom-charts"
---

# Tasks: Filtros Avançados nas Tabelas e Dashboards e Criação de Novos Gráficos Personalizados nas 5 Abas

**Input**: Design documents from `/specs/041-filters-custom-charts/`

**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md), [data-model.md](./data-model.md), [contracts/ipc-commands.md](./contracts/ipc-commands.md), [quickstart.md](./quickstart.md)

**Tests**: Obrigatórios (Constituição Princípio I — TDD Obrigatório: escrever o teste primeiro, ver falhar, implementar até passar e refatorar).

**Organization**: Tarefas organizadas por história de usuário (US1, US2, US3) em estrita ordem de TDD para permitir implementação e validação independentes.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Pode ser executada em paralelo (arquivos distintos, sem dependências de tarefas incompletas)
- **[Story]**: História de usuário associada (`[US1]`, `[US2]`, `[US3]`)
- Caminhos exatos de arquivos incluídos em todas as descrições

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Migração SQLite para a tabela interna `custom_dashboard_charts` (`exported: false`)

- [X] T001 Atualizar o teste `migration_creates_nineteen_tables` e adicionar teste de schema para `custom_dashboard_charts` (`id`, `scope`, `entity`, `title`, `group_by`, `metric`, `metric_field`, `kind`, `filter_logic`, `filters_json`, `created_at`) em `src-tauri/src/db.rs`
- [X] T002 Criar a migração `src-tauri/migrations/007_custom_dashboard_charts.sql` e registrá-la em `MIGRATIONS` em `src-tauri/src/db.rs` até os testes de `T001` passarem

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Motor unificado de avaliação de filtros (`ColumnFilter`, `FilterSpec` com lógica `and`/`or` e extração de valores escalares e JSON estruturado)

**⚠️ CRITICAL**: Pré-requisito para todas as histórias de usuário

- [X] T003 Escrever testes unitários em `src-tauri/src/analytics.rs` para `parse_filter_spec`, `extract_field_tokens` e `record_matches_filter_spec` validando: (a) todos os 5 operadores (`contains`, `equals`, `not_equals`, `is_empty`, `not_empty`), (b) combinação lógica `"and"` (padrão) e `"or"`, e (c) extração de `name`/`title`/`institution`/`degree` dentro de colunas JSON estruturadas (`campus`, `knowledge_areas`, `academic_education`)
- [X] T004 Implementar `ColumnFilter`, `FilterSpec`, `parse_filter_spec`, `extract_field_tokens` e `record_matches_filter_spec` em `src-tauri/src/analytics.rs` até os testes de `T003` passarem

**Checkpoint**: Motor de filtros testado e pronto para uso nas listagens, dashboards e gráficos personalizados

---

## Phase 3: User Story 1 - Filtros Estruturados por Coluna nas Tabelas das 5 Abas (Priority: P1) 🎯 MVP

**Goal**: Permitir aplicar um ou mais filtros por coluna (combinados com `E / AND` por padrão ou `OU / OR`) em todas as tabelas das 5 abas (**Horizon**, **Horizon Legacy**, **SRC**, **Egressos** e **Egressos — Dados**), atualizando a listagem paginada e o total de registros.

**Independent Test**: Executar `cargo test` e `npm test` validando a filtragem estruturada em `crud.rs`, `src_domain.rs`, `egressos_domain.rs`, `egressos_dados_domain.rs` e no componente `FilterBuilderBar.jsx`.

### Tests for User Story 1 (TDD — Red First) ⚠️

- [X] T005 [P] [US1] Escrever testes de integração em `src-tauri/src/crud.rs` (Horizon e Horizon Legacy) e `src-tauri/src/src_domain.rs` (SRC) validando listagem e contagem total com `filters: Option<&str>` (`and` e `or`, operadores `contains`, `equals`, `not_equals`, `is_empty`, `not_empty`) combinados com `search` e `sort`
- [X] T006 [P] [US1] Escrever testes de integração em `src-tauri/src/egressos_domain.rs` e `src-tauri/src/egressos_dados_domain.rs` validando `list_rows_filtered` com `filters: Option<&str>` (`and` e `or`) sobre os registros JSON das seções
- [X] T007 [P] [US1] Adicionar testes em `frontend/tests/unit/api.test.js` verificando o repasse do query parameter `filters` nas rotas de listagem das 5 abas (`/{entity}`, `/horizon-v2/{entity}`, `/src/acoes`, `/egressos/files/{fk}/rows`, `/egressos-dados/files/{fk}/rows`)
- [X] T008 [P] [US1] Criar `frontend/tests/unit/FilterBuilderBar.test.jsx` testando adição de filtros por coluna, alternância `E (AND)` / `OU (OR)`, remoção individual por chip, limpeza geral e acionamento do callback `onCreateChartFromFilters`

### Implementation for User Story 1

- [X] T009 [US1] Adicionar suporte a `filters: Option<&str>` (`FilterSpec`) em `src-tauri/src/crud.rs` (`list_filtered`, `hv2_list_filtered`) e `src-tauri/src/src_domain.rs` (`list_acoes_filtered`), mantendo `list`, `hv2_list` e `list_acoes` compatíveis com os testes existentes
- [X] T010 [US1] Adicionar `list_rows_filtered` com `filters: Option<&str>` (`FilterSpec`) em `src-tauri/src/egressos_domain.rs` e `src-tauri/src/egressos_dados_domain.rs`, mantendo `list_rows` compatível com os testes existentes
- [X] T011 [US1] Atualizar os comandos `list_entities`, `hv2_list_entities`, `src_list_acoes`, `egressos_list_rows` e `egressos_dados_list_rows` em `src-tauri/src/commands.rs` e as rotas correspondentes em `frontend/src/services/api.js` para repassar `filters`
- [X] T012 [P] [US1] Criar o componente `frontend/src/components/FilterBuilderBar.jsx` com seletor de coluna, operador (`contém`, `igual a`, `diferente de`, `preenchido`, `vazio`), valor, alternador `E (AND)` / `OU (OR)`, chips removíveis, botão "Limpar filtros" e botão **"📊 Criar Gráfico com estes Filtros"**
- [X] T013 [P] [US1] Integrar `FilterBuilderBar` em `frontend/src/components/EntityPage.jsx` (15 abas do Horizon e 15 abas do Horizon Legacy) e `frontend/src/components/SrcAcoesPage.jsx` (aba SRC), repassando `filters` serializado na consulta paginada
- [X] T014 [P] [US1] Integrar `FilterBuilderBar` em `frontend/src/components/EgressosTablesPage.jsx` (21 tabelas de Egressos) e `frontend/src/components/EgressosDadosTablesPage.jsx` (39 tabelas de Egressos — Dados), repassando `filters` serializado na consulta paginada

**Checkpoint**: User Story 1 concluída — todas as tabelas das 5 abas possuem filtros estruturados por coluna funcionando de ponta a ponta.

---

## Phase 4: User Story 2 - Filtros Globais e de Recorte nos Dashboards das 5 Abas (Priority: P1)

**Goal**: Permitir aplicar filtros de recorte nos Dashboards Analíticos das 5 abas (tanto via `FilterBuilderBar` no Dashboard quanto clicando em categorias/barras de um gráfico), recalculando dinamicamente os KPIs e gráficos sobre o subconjunto filtrado.

**Independent Test**: Executar `cargo test` e `npm test` verificando que os 5 motores de analytics recalculam KPIs e gráficos quando `filters` é informado e que `AnalyticsDashboard.jsx` permite aplicar e limpar filtros.

### Tests for User Story 2 (TDD — Red First) ⚠️

- [X] T015 [P] [US2] Escrever testes de integração em `src-tauri/src/analytics.rs` verificando que `horizon_analytics`, `src_analytics`, `egressos_analytics` e `egressos_dados_analytics` recalculam KPIs e séries gráficas quando recebem `filters: Option<&str>` (ex.: filtrando por `campus = Serra` ou `status = Active` ou `curso = BSI`)
- [X] T016 [P] [US2] Adicionar testes em `frontend/tests/unit/api.test.js` e `frontend/tests/unit/AnalyticsDashboard.test.jsx` verificando o envio de `filters` nas 5 rotas de analytics e a filtragem interativa no componente `AnalyticsDashboard`

### Implementation for User Story 2

- [X] T017 [US2] Atualizar `horizon_analytics`, `src_analytics`, `egressos_analytics` e `egressos_dados_analytics` em `src-tauri/src/analytics.rs` (incluindo `available_entities` no `DashboardPayload`) e os comandos correspondentes em `src-tauri/src/commands.rs` e `frontend/src/services/api.js` para aceitar `filters: Option<String>`
- [X] T018 [P] [US2] Atualizar `frontend/src/components/AnalyticsDashboard.jsx` e `frontend/src/components/TableAnalyticsPanel.jsx` para exibir controles de filtro de recorte no Dashboard (e clique rápido em itens de gráficos para filtrar) e sincronizar os filtros ativos da tabela quando aberto dentro de uma página de tabela

**Checkpoint**: User Story 2 concluída — todos os Dashboards das 5 abas recalculam KPIs e gráficos dinamicamente com filtros.

---

## Phase 5: User Story 3 - Criação de Novos Gráficos Personalizados no Dashboard a partir dos Filtros (Priority: P1)

**Goal**: Permitir criar, persistir no SQLite local e excluir novos gráficos personalizados em qualquer uma das 5 abas (escolhendo título, tabela, coluna de agrupamento `group_by`, métrica `count` ou `sum` + `metric_field`, tipo visual e filtros `AND`/`OR`), exibindo os gráficos criados tanto no painel "Ver Gráficos da Tabela" da tabela de origem quanto no Dashboard Geral daquela aba.

**Independent Test**: Executar `cargo test` e `npm test` validando a criação, cálculo (`count` e `sum`), listagem no painel da tabela e no Dashboard Geral, e exclusão de `CustomDashboardChart` nas 5 abas.

### Tests for User Story 3 (TDD — Red First) ⚠️

- [X] T019 [P] [US3] Escrever testes de integração em `src-tauri/src/analytics.rs` para `create_custom_chart`, `list_custom_charts` e `delete_custom_chart` cobrindo: (a) cálculo com métrica `"count"` e métrica `"sum"` (numérica e contagem de vínculos JSON), (b) aplicação das regras de filtro (`and`/`or`) do gráfico, (c) inclusão automática do gráfico personalizado tanto no painel da tabela (`entity = Some(...)`) quanto no Dashboard Geral da aba (`entity = None`), e (d) isolamento entre as 5 abas
- [X] T020 [P] [US3] Adicionar testes em `frontend/tests/unit/api.test.js` (rotas `POST /custom-charts`, `GET /custom-charts` e `DELETE /custom-charts/:id`) e em `frontend/tests/unit/AnalyticsDashboard.test.jsx` (criação de novo gráfico via `CustomChartModal` e exclusão de gráfico personalizado)

### Implementation for User Story 3

- [X] T021 [US3] Implementar `create_custom_chart`, `list_custom_charts`, `delete_custom_chart` e `compute_custom_chart_section` em `src-tauri/src/analytics.rs`, anexando os gráficos personalizados correspondentes ao `DashboardPayload` retornado pelos 5 motores analíticos
- [X] T022 [US3] Expor os comandos Tauri `create_custom_chart`, `list_custom_charts` e `delete_custom_chart` em `src-tauri/src/commands.rs`, registrá-los em `src-tauri/src/lib.rs` e mapear as rotas `/custom-charts` em `frontend/src/services/api.js`
- [X] T023 [P] [US3] Criar o componente `frontend/src/components/CustomChartModal.jsx` permitindo configurar título, tabela/entidade, coluna de agrupamento (`group_by`), métrica (`count` Contagem de Registros ou `sum` Soma Numérica / Quantidade de Vínculos + `metric_field`), tipo visual (`bar`, `donut`, `columns`, `financial_table`) e filtros (`AND`/`OR`, pré-carregados quando acionado a partir da tabela ou do Dashboard)
- [X] T024 [P] [US3] Integrar `CustomChartModal` e o botão de exclusão de gráficos personalizados (`is_custom: true`) em `frontend/src/components/AnalyticsDashboard.jsx`, `frontend/src/components/TableAnalyticsPanel.jsx`, `frontend/src/components/EntityPage.jsx`, `frontend/src/components/SrcAcoesPage.jsx`, `frontend/src/components/EgressosTablesPage.jsx` e `frontend/src/components/EgressosDadosTablesPage.jsx`

**Checkpoint**: User Story 3 concluída — o usuário pode criar novos gráficos personalizados a partir de filtros em todas as 5 abas e visualizá-los tanto na tabela quanto no Dashboard Geral.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Validação integral de todas as suítes de testes e build de produção

- [X] T025 Executar `cargo test` em `src-tauri/` validando que 100% dos testes unitários, de integração, paridade de exportação (`SC-004`) e analytics passem sem erros
- [X] T026 [P] Executar `npm test` e `npm run build` em `frontend/` validando que 100% dos testes de componentes/rotas e o build estático Astro passem sem erros

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)** + **Phase 2 (Foundational)** → bloqueiam as fases de histórias de usuário (`US1`, `US2`, `US3`).
- **Phase 3 (US1)** → **Phase 4 (US2)** → **Phase 5 (US3)**: compartilham `FilterBuilderBar.jsx`, `analytics.rs` e `api.js`.
- **Phase 6 (Polish)**: depende da conclusão de `US1`, `US2` e `US3`.

### Parallel Opportunities

- Dentro da **US1**: `T005`, `T006`, `T007` e `T008` podem rodar em paralelo; `T012`, `T013` e `T014` podem rodar em paralelo.
- Dentro da **US2**: `T015` e `T016` podem rodar em paralelo.
- Dentro da **US3**: `T019` e `T020` podem rodar em paralelo; `T023` e `T024` podem rodar em paralelo.

---

## Parallel Example: User Story 1

```bash
# Testes de backend e frontend em paralelo (Red Phase):
Task: "T005 [P] [US1] Escrever testes de integração em src-tauri/src/crud.rs e src-tauri/src/src_domain.rs"
Task: "T006 [P] [US1] Escrever testes de integração em src-tauri/src/egressos_domain.rs e src-tauri/src/egressos_dados_domain.rs"
Task: "T007 [P] [US1] Adicionar testes em frontend/tests/unit/api.test.js"
Task: "T008 [P] [US1] Criar frontend/tests/unit/FilterBuilderBar.test.jsx"
```

---

## Implementation Strategy

### MVP First (User Story 1)

1. Concluir Phase 1 (`007_custom_dashboard_charts.sql`) e Phase 2 (`FilterSpec` / `ColumnFilter` engine)
2. Concluir Phase 3 (US1 — Filtros estruturados por coluna nas tabelas das 5 abas)
3. Validar testes em `cargo test` e `npm test`

### Incremental Delivery

1. Adicionar Phase 4 (US2 — Filtros interativos nos Dashboards das 5 abas)
2. Adicionar Phase 5 (US3 — Criação e persistência de novos gráficos personalizados a partir dos filtros)
3. Executar Phase 6 (`cargo test`, `npm test` e `npm run build`)
