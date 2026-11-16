---
description: "Task list for feature 040-horizon-analytics-dashboard"
---

# Tasks: Adaptação Final do Pacote Canônico Horizon, Indicador de Status de Projetos e Dashboards Analíticos em Todas as Abas

**Input**: Design documents from `/specs/040-horizon-analytics-dashboard/`

**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md), [data-model.md](./data-model.md), [contracts/ipc-commands.md](./contracts/ipc-commands.md), [quickstart.md](./quickstart.md)

**Tests**: Obrigatórios (Constituição Princípio I — TDD Obrigatório: escrever o teste primeiro, ver falhar, implementar até passar e refatorar).

**Organization**: Tarefas agrupadas por história de usuário (US1, US2, US3) em estrita ordem de TDD para permitir implementação e validação independentes.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Pode ser executada em paralelo (arquivos distintos, sem dependências de tarefas incompletas)
- **[Story]**: História de usuário associada (`[US1]`, `[US2]`, `[US3]`)
- Caminhos exatos de arquivos incluídos em todas as descrições

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Verificação do pacote de referência final e criação do esqueleto do módulo analítico

- [X] T001 Verificar a presença e integridade do pacote de referência `exports_canonical.zip` (698 entradas e 70.545 registros nas 15 tabelas canônicas) na raiz `/home/thiagodeps/research_hub/exports_canonical.zip`
- [X] T002 Criar o módulo base `src-tauri/src/analytics.rs` e declará-lo (`pub mod analytics;`) em `src-tauri/src/lib.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Tipos de domínio analíticos compartilhados e regras de classificação de status de projetos (`Active` / `In Progress` vs. `Concluded` / `Cancelled` / `Unknown`)

**⚠️ CRITICAL**: Pré-requisito para as fases de histórias de usuário

- [X] T003 Escrever testes unitários em `src-tauri/src/analytics.rs` para a regra de classificação de status de projeto (`classify_project_status`: `"Active"` → `is_active: true, "Ativo (Em Execução)"`, `"In Progress"` → `is_active: true, "Ativo (Em Andamento)"`, `"Concluded"` → `is_active: false, "Inativo (Concluído)"`, `"Cancelled"` → `is_active: false, "Inativo (Cancelado)"`, `"Unknown"`/`"N/A"`/`""`/`None` → `is_active: false, "Inativo (Não Informado)"`) e higienização de instituições de formação acadêmica (filtrando rótulos `"(Carga Horária..."`)
- [X] T004 Implementar as estruturas serializáveis (`ProjectWorkspaceStatus`, `KpiCard`, `ChartItem`, `ChartSection`, `DashboardPayload`) e funções puras de classificação/higienização em `src-tauri/src/analytics.rs` até os testes de `T003` passarem

**Checkpoint**: Estruturas base e regras puras de classificação prontas e testadas

---

## Phase 3: User Story 1 - Adaptação Completa à Versão Final do `exports_canonical.zip` (Priority: P1) 🎯 MVP

**Goal**: Suportar as 22 colunas de `researchers` e `students` (incluindo `linkedin_url`), atualizar as contagens de referência para 70.545 registros nas 15 tabelas canônicas e garantir 100% de fidelidade no round-trip de importação e exportação.

**Independent Test**: Executar `cargo test` validando a migração `006`, o acordo entre `registry.rs` e o schema SQLite para as 22 colunas de pessoas, a importação dos 70.545 registros do `exports_canonical.zip` e o round-trip de exportação das 698 entradas.

### Tests for User Story 1 (TDD — Red First) ⚠️

- [X] T005 [P] [US1] Atualizar os testes de schema e registry em `src-tauri/src/db.rs` e `src-tauri/src/registry.rs` para exigir a coluna `linkedin_url` (22 colunas em `researchers`, `students`, `hv2_researchers` e `hv2_students`)
- [X] T006 [P] [US1] Atualizar o teste `reference_archive_matches_row_counts` em `src-tauri/src/import.rs` com as contagens finais do novo `exports_canonical.zip` (`researchers`: 18334, `students`: 11717, `articles`: 8079, `initiatives`: 15546, `knowledge_areas`: 1563, `professional_activities`: 9560, `research_productions`: 3732, `research_groups`: 353, `proficiencies`: 794, `advisorships`: 190, `organizations`: 440, `awards`: 183, `campuses`: 23, `fellowships`: 20, `languages`: 11; total: 70.545) e validar a preservação de `linkedin_url` em `src-tauri/src/export.rs`

### Implementation for User Story 1

- [X] T007 [US1] Criar a migração `src-tauri/migrations/006_person_linkedin_url.sql` (`ALTER TABLE researchers ADD COLUMN linkedin_url TEXT; ALTER TABLE students ADD COLUMN linkedin_url TEXT; ALTER TABLE hv2_researchers ADD COLUMN linkedin_url TEXT; ALTER TABLE hv2_students ADD COLUMN linkedin_url TEXT;`) e registrá-la em `MIGRATIONS` em `src-tauri/src/db.rs`
- [X] T008 [US1] Adicionar `"linkedin_url"` em `PERSON_COLUMNS` em `src-tauri/src/registry.rs` (totalizando 22 colunas para `researchers` e `students` no Horizon e Horizon Legacy)
- [X] T009 [P] [US1] Adicionar o campo `{ name: 'linkedin_url', label: 'LinkedIn URL' }` nas páginas `frontend/src/pages/horizon/researchers.astro` e `frontend/src/pages/horizon/students.astro` (mantendo o teste `horizon_v2_pages_only_declare_columns_that_exist` verde)
- [X] T010 [P] [US1] Adicionar o campo `{ name: 'linkedin_url', label: 'LinkedIn URL' }` nas páginas `frontend/src/pages/dashboard/researchers.astro` e `frontend/src/pages/dashboard/students.astro`

**Checkpoint**: User Story 1 concluída — `cargo test` passa integralmente no round-trip do `exports_canonical.zip` final com 22 colunas em pesquisadores/alunos e 70.545 registros.

---

## Phase 4: User Story 2 - Indicativo Visual de Projeto Ativo ou Inativo (Priority: P1)

**Goal**: Exibir o selo semântico **Ativo / Inativo** tanto em cada registro de projeto nas listagens e formulários (`initiatives` e `advisorships` no Horizon e Horizon Legacy) quanto nos 5 cartões de seleção de projetos (`/projects`).

**Independent Test**: Verificar via `cargo test` e `npm test` que `projects_status` retorna o estado ativo/inativo e contagens dos 5 projetos, que `ProjectSelector` exibe o selo correspondente nos cartões e que `EntityTable` e `EntityForm` renderizam o `ProjectStatusBadge` para registros de projetos.

### Tests for User Story 2 (TDD — Red First) ⚠️

- [X] T011 [P] [US2] Escrever testes de integração em `src-tauri/src/analytics.rs` para `compute_projects_status(conn)` validando o status ativo/inativo, `total_rows` e `active_projects` para os 5 projetos (`horizon-v2`, `horizon`, `src`, `egressos`, `egressos-dados`) em banco vazio e banco populado
- [X] T012 [P] [US2] Criar `frontend/tests/unit/ProjectStatusBadge.test.jsx` testando a renderização do selo para `"Active"`, `"In Progress"`, `"Concluded"`, `"Cancelled"`, `"Unknown"` e nulo
- [X] T013 [P] [US2] Adicionar testes em `frontend/tests/unit/api.test.js` (rota `GET /projects/status` → `invoke('projects_status')`) e em `frontend/tests/unit/ProjectSelector.test.jsx` (exibição do indicativo Ativo / Sem dados nos cartões de projetos)

### Implementation for User Story 2

- [X] T014 [US2] Implementar `compute_projects_status(conn: &Connection) -> Result<Vec<ProjectWorkspaceStatus>, AppError>` em `src-tauri/src/analytics.rs` e expor o comando Tauri `projects_status` em `src-tauri/src/commands.rs` e `src-tauri/src/lib.rs`
- [X] T015 [US2] Mapear a rota `GET /projects/status` → `invoke('projects_status')` em `frontend/src/services/api.js` (antes das rotas genéricas `/{entity}`)
- [X] T016 [P] [US2] Criar o componente `frontend/src/components/ProjectStatusBadge.jsx` com a função auxiliar `classifyProjectStatus(status)` e renderização visual do badge Ativo / Inativo
- [X] T017 [P] [US2] Atualizar `frontend/src/components/EntityTable.jsx` para renderizar `ProjectStatusBadge` quando `col === 'status'` (aplicado em `initiatives` e `advisorships` conforme `FR-004`) e `frontend/src/components/EntityForm.jsx` para exibir `ProjectStatusBadge` ao lado do campo quando `field.name === 'status'` (atualizando dinamicamente ao editar conforme `FR-005`)
- [X] T018 [US2] Atualizar `frontend/src/components/ProjectSelector.jsx` e `frontend/src/pages/projects.astro` para buscar `GET /projects/status` e exibir em cada um dos 5 cartões o selo indicando se o projeto está **Ativo** (com contagem de registros e projetos ativos) ou **Inativo (Sem dados)**

**Checkpoint**: User Story 2 concluída — tanto os registros de projetos quanto os cartões da tela `/projects` exibem claramente o indicativo de projeto Ativo ou Inativo.

---

## Phase 5: User Story 3 - Dashboards Analíticos Contextualizados para Todas as Abas (Priority: P1)

**Goal**: Disponibilizar tanto o **Dashboard Geral do Projeto** nos 5 ambientes (**Horizon**, **Horizon Legacy**, **SRC**, **Egressos** e **Egressos — Dados**) quanto o modo/painel **"Ver Gráficos da Tabela"** dentro de cada página de tabela individual, exibindo gráficos e KPIs calculados em Rust sobre o SQLite local (projetos ativos vs. inativos, volume das tabelas, quanto recebem/supervisionam os pesquisadores e bolsistas por programa de bolsa, instituições de origem/atuação, titulação e métricas específicas de cada aba).

**Independent Test**: Executar `cargo test` e `npm test` verificando as agregações em Rust e a renderização visual dos componentes `AnalyticsDashboard` e `TableAnalyticsPanel` para todos os 5 projetos e suas tabelas.

### Tests for User Story 3 (TDD — Red First) ⚠️

- [X] T019 [P] [US3] Escrever testes de integração em `src-tauri/src/analytics.rs` para `horizon_analytics(conn, v2, entity)` cobrindo: (a) Dashboard Geral (`entity: None`) com status de projetos ativos (`Active` + `In Progress`) vs. concluídos/cancelados, contagens das 15 tabelas, bolsas por programa (`fellowships`), top pesquisadores orientadores e top bolsistas (`advisorships`), instituições de atuação (`professional_activities`) e de formação (`academic_education` sem ruído de carga horária), titulação e produção por ano; e (b) Gráficos por Tabela (`entity: Some(route)`) para as 15 entidades canônicas
- [X] T020 [P] [US3] Escrever testes de integração em `src-tauri/src/analytics.rs` para `src_analytics(conn)`, `egressos_analytics(conn, file_key)` e `egressos_dados_analytics(conn, file_key)` cobrindo visão geral e visão por tabela individual em banco vazio e populado
- [X] T021 [P] [US3] Adicionar testes em `frontend/tests/unit/api.test.js` para as rotas `GET /horizon-v2/analytics`, `GET /analytics`, `GET /src/analytics`, `GET /egressos/analytics` e `GET /egressos-dados/analytics` (com e sem query string `entity` / `file`)
- [X] T022 [P] [US3] Criar `frontend/tests/unit/AnalyticsDashboard.test.jsx` testando a renderização de `AnalyticsDashboard` (KPIs, gráficos de status de projetos, bolsas/pesquisadores, instituições, contagens por tabela e estado vazio) e `TableAnalyticsPanel` (alternância do botão "Ver Gráficos da Tabela" e renderização dos gráficos específicos da tabela)

### Implementation for User Story 3

- [X] T023 [US3] Implementar `horizon_analytics(conn: &Connection, v2: bool, entity: Option<&str>) -> Result<DashboardPayload, AppError>` em `src-tauri/src/analytics.rs` (Dashboard Geral + gráficos contextualizados para cada uma das 15 rotas canônicas)
- [X] T024 [US3] Implementar `src_analytics(conn: &Connection) -> Result<DashboardPayload, AppError>`, `egressos_analytics(conn: &Connection, file_key: Option<&str>) -> Result<DashboardPayload, AppError>` e `egressos_dados_analytics(conn: &Connection, file_key: Option<&str>) -> Result<DashboardPayload, AppError>` em `src-tauri/src/analytics.rs`
- [X] T025 [US3] Expor os comandos Tauri `hv2_get_analytics`, `get_horizon_analytics`, `src_get_analytics`, `egressos_get_analytics` e `egressos_dados_get_analytics` em `src-tauri/src/commands.rs` e registrá-los em `src-tauri/src/lib.rs`
- [X] T026 [US3] Mapear as rotas `GET /horizon-v2/analytics`, `GET /analytics`, `GET /src/analytics`, `GET /egressos/analytics` e `GET /egressos-dados/analytics` em `frontend/src/services/api.js`
- [X] T027 [P] [US3] Criar o componente `frontend/src/components/AnalyticsDashboard.jsx` com renderização de cartões de KPI e gráficos SVG/Tailwind (`donut`, `bar`, `columns`, `financial_table`), botão de atualização e reação automática a eventos de importação
- [X] T028 [P] [US3] Criar o componente `frontend/src/components/TableAnalyticsPanel.jsx` com botão alternador **"📊 Ver Gráficos da Tabela"** que busca e exibe sob demanda os KPIs e gráficos contextualizados da tabela ativa
- [X] T029 [P] [US3] Integrar `AnalyticsDashboard` nas páginas principais dos projetos: `frontend/src/pages/horizon/index.astro`, `frontend/src/pages/dashboard/index.astro`, `frontend/src/pages/src/index.astro` e `frontend/src/pages/egressos/index.astro`
- [X] T030 [P] [US3] Integrar `TableAnalyticsPanel` em `frontend/src/components/EntityPage.jsx` (habilitando "Ver Gráficos da Tabela" nas 15 abas de entidades tanto do Horizon quanto do Horizon Legacy)
- [X] T031 [P] [US3] Integrar `TableAnalyticsPanel` / `AnalyticsDashboard` nos workspaces `frontend/src/components/SrcAcoesPage.jsx`, `frontend/src/components/EgressosTablesPage.jsx`, `frontend/src/components/EgressosDadosTablesPage.jsx` e `frontend/src/components/EgressosDadosWorkspace.jsx` (habilitando o Dashboard Geral e "Ver Gráficos da Tabela" no SRC, nas 21 tabelas de Egressos e nas 39 tabelas de Egressos — Dados)

**Checkpoint**: User Story 3 concluída — todas as abas de projeto e todas as páginas de tabelas individuais possuem Dashboards e Gráficos contextualizados funcionando localmente.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Validação ponta a ponta de todas as suítes de testes e build estático

- [X] T032 Executar `cargo test` em `src-tauri/` garantindo que 100% dos testes unitários, de schema, round-trip do `exports_canonical.zip` e agregações analíticas passem sem avisos ou regressões
- [X] T033 [P] Executar `npm test` e `npm run build` em `frontend/` garantindo que 100% dos testes unitários de componentes/rotas e o build estático Astro passem sem erros
