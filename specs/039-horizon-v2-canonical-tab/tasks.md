---

description: "Task list for Aba Horizon (Nova Versão Canônica) e Preservação do Horizon Legacy"
---

# Tasks: Aba Horizon (Nova Versão Canônica) e Preservação do Horizon Legacy

**Input**: Design documents from `/specs/039-horizon-v2-canonical-tab/`

**Prerequisites**: [plan.md](./plan.md) (required), [spec.md](./spec.md) (required for user stories), [research.md](./research.md), [data-model.md](./data-model.md), [contracts/ipc-commands.md](./contracts/ipc-commands.md), [quickstart.md](./quickstart.md)

**Tests**: Obrigatórios pela Constituição do Projeto (Princípio I — TDD Obrigatório: escrever o teste primeiro, ver falhar, implementar até passar e refatorar).

**Organization**: Tarefas agrupadas por User Story para permitir implementação e validação independentes de cada história.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Pode ser executada em paralelo (arquivos distintos, sem dependências de tasks incompletas)
- **[Story]**: User story à qual a tarefa pertence (`[US1]`, `[US2]`, `[US3]`)
- Caminhos exatos dos arquivos incluídos em todas as descrições

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Preparação da estrutura de diretórios e migração SQLite para o novo domínio isolado `horizon_v2` (`hv2_*`).

- [X] T001 Criar a migração SQLite `src-tauri/migrations/005_horizon_v2_tables.sql` declarando as 15 tabelas isoladas (`hv2_researchers`, `hv2_students`, `hv2_articles`, `hv2_research_groups`, `hv2_initiatives`, `hv2_advisorships`, `hv2_awards`, `hv2_campuses`, `hv2_organizations`, `hv2_fellowships`, `hv2_proficiencies`, `hv2_professional_activities`, `hv2_knowledge_areas`, `hv2_languages`, `hv2_research_productions`) com `id INTEGER PRIMARY KEY` e demais colunas `TEXT` conforme `data-model.md`
- [X] T002 [P] Criar o diretório de páginas `frontend/src/pages/horizon/` e o layout base `frontend/src/layouts/HorizonDashboard.astro` apontando o menu lateral para `/horizon` e `/horizon/<entidade>`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Infraestrutura central no registry e no banco SQLite que bloqueia todas as histórias de usuário.

**⚠️ CRITICAL**: Nenhuma User Story pode iniciar antes da conclusão desta fase.

- [X] T003 [P] Escrever testes unitários em `src-tauri/src/registry.rs` validando `HORIZON_V2_ENTITIES` (15 entidades com tabelas `hv2_*`), `by_horizon_v2_route`, `EntityDef::canonical_table()` (removendo o prefixo `hv2_` para mapear ao arquivo `{tabela}_canonical.json`) e garantindo que rotas `hv2` e legadas resolvam para tabelas físicas distintas
- [X] T004 [P] Escrever testes de migração em `src-tauri/src/db.rs` verificando a criação das 15 novas tabelas `hv2_*` (totalizando 38 tabelas no banco) e a concordância bidirecional entre `HORIZON_V2_ENTITIES` e o schema SQLite (`horizon_v2_registry_and_schema_agree`)
- [X] T005 Implementar `HORIZON_V2_ENTITIES`, `by_horizon_v2_route`, `horizon_v2_exported` e `EntityDef::canonical_table(&self)` (além de incluir `"enrichment"`, `"members"`, `"leaders"` e `"academic_education"` em `SORT_BY_LENGTH`) em `src-tauri/src/registry.rs`
- [X] T006 Registrar `include_str!("../migrations/005_horizon_v2_tables.sql")` em `MIGRATIONS` dentro de `src-tauri/src/db.rs` e fazer passar todos os testes de migração e schema

**Checkpoint**: Fundação pronta — `registry.rs` e `db.rs` suportam tanto `ENTITIES` (Horizon Legacy) quanto `HORIZON_V2_ENTITIES` (Novo Horizon).

---

## Phase 3: User Story 1 - Renomear a Aba Atual para "Horizon Legacy" Mantendo Funcionamento Integral (Priority: P1) 🎯 MVP

**Goal**: Renomear o projeto original na tela de seleção de projetos para **Horizon Legacy** (ao lado do novo card **Horizon**), preservando 100% da funcionalidade, dados locais, arquivo `original.zip`, snapshots `hub-*` e sincronização `"horizon"`, e atualizando os testes de referência do pacote `exports_canonical.zip` na raiz do repositório.

**Independent Test**: Executar `npm test` para `ProjectSelector.test.jsx` e `cargo test` para `import::tests::reference_archive_matches_row_counts` e `export::tests::real_archive_round_trip_preserves_types_and_entries`, comprovando que o card "Horizon Legacy" aponta para `/dashboard` e que o fluxo legado importa e exporta o `exports_canonical.zip` sem regressões.

### Tests for User Story 1 (TDD - Write First) ⚠️

- [X] T007 [P] [US1] Atualizar e expandir os testes em `frontend/tests/unit/ProjectSelector.test.jsx` para verificar a exibição do card renomeado **Horizon Legacy** (`href: '/dashboard'`) e do novo card **Horizon** (`href: '/horizon'`)
- [X] T008 [P] [US1] Atualizar os testes de referência em `src-tauri/src/import.rs` (`reference_archive_matches_row_counts`) e `src-tauri/src/export.rs` (`real_archive_round_trip_preserves_types_and_entries`) para validar as contagens reais do novo `exports_canonical.zip` na raiz do repositório (70.615 linhas totais nas 15 tabelas, 698 entradas no ZIP e 683 entradas não gerenciadas preservadas byte a byte)

### Implementation for User Story 1

- [X] T009 [P] [US1] Atualizar `DEFAULT_PROJECTS` e o layout de grade em `frontend/src/components/ProjectSelector.jsx` para exibir o novo projeto **Horizon** (`id: 'horizon-v2'`, `title: 'Horizon'`, `href: '/horizon'`) e renomear o projeto existente (`id: 'horizon'`) para **Horizon Legacy** (`title: 'Horizon Legacy'`, `href: '/dashboard'`)
- [X] T010 [P] [US1] Atualizar a constante `PROJECTS` em `frontend/src/pages/projects.astro` e o cabeçalho do menu lateral em `frontend/src/layouts/Dashboard.astro` para identificar visualmente o ambiente **Horizon Legacy**

**Checkpoint**: User Story 1 concluída e validada independentemente.

---

## Phase 4: User Story 2 - Nova Aba "Horizon" Isolada e Adaptada ao Novo `exports_canonical.zip` (Priority: P1)

**Goal**: Entregar o ciclo completo e isolado da nova aba **Horizon** (CRUD nas 15 tabelas `hv2_*`, fusão, vínculo, importação/exportação com `horizon_v2_original.zip` e snapshots `horizon-v2-*`, sincronização GitHub com `project = "horizon_v2"` e ponte IPC `/horizon-v2/*` em `api.js`).

**Independent Test**: Importar o novo `exports_canonical.zip` no domínio `horizon_v2`, executar operações de CRUD, fusão e vínculo sobre `hv2_*`, exportar o pacote verificando as 698 entradas (incluindo `initiatives.enrichment` restaurado como objeto JSON e as 683 entradas preservadas) e comprovar que as tabelas do `Horizon Legacy` permanecem vazias/inalteradas.

### Tests for User Story 2 (TDD - Write First) ⚠️

- [X] T011 [P] [US2] Escrever testes unitários em `src-tauri/src/crud.rs` validando CRUD completo (`hv2_list`, `hv2_get`, `hv2_create`, `hv2_update`, `hv2_delete`) nas 15 tabelas de `HORIZON_V2_ENTITIES` e comprovando isolamento total em relação às tabelas de `ENTITIES` (Horizon Legacy)
- [X] T012 [P] [US2] Escrever testes unitários em `src-tauri/src/special.rs` validando `hv2_merge` e `hv2_link` sobre as tabelas `hv2_*` sem afetar as tabelas legadas
- [X] T013 [P] [US2] Escrever testes de integração em `src-tauri/src/import.rs` e `src-tauri/src/export.rs` validando `import_horizon_v2_archive` (`horizon_v2_original.zip`, snapshot `horizon-v2-*`, isolamento das tabelas legadas) e `build_horizon_v2_archive` (round-trip das 15 tabelas `hv2_*` mapeadas para `{canonical_table}_canonical.json` + preservação do objeto `enrichment` em `initiatives_canonical.json`)
- [X] T014 [P] [US2] Escrever testes unitários e de integração em `src-tauri/src/sync_domain.rs` e `src-tauri/tests/sync_github.rs` validando o projeto `"horizon_v2"` (validação de caminho `.zip`, configuração independente de `"horizon"`, download e upload do pacote canônico do novo Horizon)
- [X] T015 [P] [US2] Escrever testes unitários em `frontend/tests/unit/api.test.js` cobrindo todas as rotas `/horizon-v2/*` (`GET/POST /horizon-v2/:entity`, `GET/PUT/DELETE /horizon-v2/:entity/:id`, `POST /horizon-v2/merge/:entity`, `POST /horizon-v2/link`, `POST /horizon-v2/import`, `POST /horizon-v2/export`) e `onHorizonV2ImportProgress`

### Implementation for User Story 2

- [X] T016 [US2] Parametrizar as funções de `src-tauri/src/crud.rs` pelo catálogo de entidades e expor `hv2_list`, `hv2_get`, `hv2_create`, `hv2_update` e `hv2_delete` operando sobre `registry::HORIZON_V2_ENTITIES`
- [X] T017 [US2] Parametrizar `merge` e `link` em `src-tauri/src/special.rs` pelo catálogo de entidades e expor `hv2_merge` e `hv2_link` operando sobre `registry::HORIZON_V2_ENTITIES`
- [X] T018 [US2] Parametrizar `src-tauri/src/import.rs` e `src-tauri/src/export.rs` usando `def.canonical_table()` para o nome do arquivo `{tabela}_canonical.json` no ZIP e `def.table` para o SQLite, expondo `HORIZON_V2_ORIGINAL_ARCHIVE` (`"horizon_v2_original.zip"`), `import_horizon_v2_archive` (com prefixo de snapshot `"horizon-v2"`) e `build_horizon_v2_archive`
- [X] T019 [US2] Adicionar `"horizon_v2"` em `PROJECTS`, `validate_path` e `check_upload_allowed` em `src-tauri/src/sync_domain.rs`, e conectar `"horizon_v2"` em `download_and_import` dentro de `src-tauri/src/sync_github.rs`
- [X] T020 [US2] Implementar os comandos Tauri `hv2_list_entities`, `hv2_get_entity`, `hv2_create_entity`, `hv2_update_entity`, `hv2_delete_entity`, `hv2_merge_entities`, `hv2_link_entities`, `import_horizon_v2_zip`, `export_horizon_v2_zip` (e suporte a `project == "horizon_v2"` em `github_upload`) em `src-tauri/src/commands.rs`, registrando-os em `src-tauri/src/lib.rs`
- [X] T021 [US2] Mapear as rotas `/horizon-v2/*` e exportar `onHorizonV2ImportProgress` em `frontend/src/services/api.js`
- [X] T022 [US2] Parametrizar `frontend/src/components/DataControlCenter.jsx` com prop opcional `apiPrefix = ""` para utilizar `apiFetch('${apiPrefix}/import')`, `apiFetch('${apiPrefix}/export')` e `onHorizonV2ImportProgress` de `frontend/src/services/api.js` quando `apiPrefix === "/horizon-v2"` (mantendo o comportamento original para o Horizon Legacy), e criar a página principal da nova aba em `frontend/src/pages/horizon/index.astro` com `DataControlCenter` configurado para `apiPrefix="/horizon-v2"` e `SyncPanel` com `project="horizon_v2"`

**Checkpoint**: User Stories 1 e 2 funcionais e testadas independentemente.

---

## Phase 5: User Story 3 - Visibilidade Completa dos Campos Canônicos na Nova Aba "Horizon" (Priority: P2)

**Goal**: Expor nas 15 páginas de curadoria da nova aba **Horizon** (`/horizon/*`) todas as colunas canônicas de cada entidade (incluindo o objeto estruturado `enrichment` em Iniciativas em modo somente-leitura legível) e garantir que os links de vínculos (`?openId=`) naveguem dentro do próprio ambiente (`/horizon/<rota>` vs `/dashboard/<rota>`).

**Independent Test**: Renderizar `EntityForm` e `EntityPage` com dados enriquecidos de `initiatives` e `basePath="/horizon"`, verificando a exibição estruturada legível de `enrichment` (incluindo sub-objetos como `objetivos` e listas como `cronograma`/`palavras_chave`), a navegação de links para `/horizon/<rota>?openId=<id>` e a validação de colunas nas 15 páginas `frontend/src/pages/horizon/*.astro`.

### Tests for User Story 3 (TDD - Write First) ⚠️

- [X] T023 [P] [US3] Escrever testes unitários de componentes para `EntityForm` e `EntityPage` em `frontend/tests/unit/EntityForm.test.jsx` verificando a renderização legível de objetos aninhados em campos `json_readonly` (como `initiatives.enrichment`), a geração de links `${basePath}/${route}?openId=${item.id}` com `basePath="/horizon"` (incluindo colunas singulares/aliases como `organization`, `research_group`, `team`/`members`/`leaders` e objetos sem rota própria como `initiative_type`) e o envio de requisições com `apiPrefix="/horizon-v2"`
- [X] T024 [P] [US3] Adicionar teste em `src-tauri/src/registry.rs` (`horizon_v2_pages_only_declare_columns_that_exist`) verificando que todas as 15 páginas em `frontend/src/pages/horizon/*.astro` declaram apenas rotas e colunas válidas de `HORIZON_V2_ENTITIES`

### Implementation for User Story 3

- [X] T025 [US3] Atualizar `frontend/src/components/EntityForm.jsx` e `frontend/src/components/EntityPage.jsx` adicionando suporte às props opcionais `apiPrefix = ""` e `basePath = "/dashboard"`, mapeando colunas de relacionamento singulares/aliases para suas rotas válidas (`organization`/`demandante`/`external_partner` → `organizations`, `research_group`/`external_research_group` → `groups`, `team`/`members`/`leaders` → `researchers`), gerando links `${basePath}/${route}?openId=${item.id}` apenas para rotas das 15 entidades válidas (exibindo objetos auxiliares sem rota própria, como `initiative_type`, em texto estruturado) e formatando valores aninhados (objetos e arrays como `objetivos`, `cronograma` e `palavras_chave` em `enrichment`) de maneira legível nos campos `json_readonly`
- [X] T026 [P] [US3] Criar as páginas de pessoas e produção em `frontend/src/pages/horizon/researchers.astro`, `frontend/src/pages/horizon/students.astro`, `frontend/src/pages/horizon/articles.astro`, `frontend/src/pages/horizon/research_productions.astro` e `frontend/src/pages/horizon/awards.astro` usando `HorizonDashboard`, `apiPrefix="/horizon-v2"`, `basePath="/horizon"` e declarando todas as colunas canônicas de cada tabela em `fields`
- [X] T027 [P] [US3] Criar as páginas de projetos, grupos e orientações em `frontend/src/pages/horizon/initiatives.astro` (incluindo o campo `enrichment` como `json_readonly` e todas as 19 colunas), `frontend/src/pages/horizon/groups.astro` (incluindo `members`, `leaders` e todas as 13 colunas) e `frontend/src/pages/horizon/advisorships.astro` (todas as 9 colunas)
- [X] T028 [P] [US3] Criar as páginas de catálogos institucionais em `frontend/src/pages/horizon/campuses.astro`, `frontend/src/pages/horizon/organizations.astro`, `frontend/src/pages/horizon/fellowships.astro`, `frontend/src/pages/horizon/proficiencies.astro`, `frontend/src/pages/horizon/professional_activities.astro`, `frontend/src/pages/horizon/knowledge_areas.astro` e `frontend/src/pages/horizon/languages.astro` com todas as colunas canônicas declaradas em `fields`

**Checkpoint**: Todas as 3 User Stories implementadas e testáveis de ponta a ponta.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Validação final de rotas embarcadas no binário Tauri, build do front-end e execução completa do guia `quickstart.md`.

- [X] T029 Atualizar a constante `ROUTES` e o teste `contagem_de_rotas_bate_com_o_menu_lateral` em `src-tauri/src/lib.rs` incluindo `/horizon` e as 15 sub-rotas `/horizon/<entidade>` (totalizando 41 rotas verificadas pelo resolvedor de assets do Tauri)
- [X] T030 Executar `npm test -- --run` e `npm run build` em `frontend/` seguidos de `cargo test --manifest-path src-tauri/Cargo.toml` para validar 100% dos testes unitários, de integração, de pacote canônico e de resolução de rotas embarcadas conforme `specs/039-horizon-v2-canonical-tab/quickstart.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Sem dependências — início imediato.
- **Foundational (Phase 2)**: Depende da Phase 1 — bloqueia todas as User Stories.
- **User Story 1 (Phase 3, P1)**: Depende da Phase 2 — pode rodar em paralelo com a Phase 4.
- **User Story 2 (Phase 4, P1)**: Depende da Phase 2 — entrega o domínio completo `horizon_v2` no Rust e na ponte `api.js`.
- **User Story 3 (Phase 5, P2)**: Depende da conclusão de T021/T022 da User Story 2 — entrega as 15 páginas de curadoria `/horizon/*` com todos os campos expostos.
- **Polish (Phase 6)**: Depende da conclusão das Phases 3, 4 e 5.

### Within Each User Story (Strict TDD)

1. Escrever os testes da história primeiro (`T007..T008`, `T011..T015`, `T023..T024`) e confirmar que falham antes da implementação.
2. Implementar a lógica correspondente em Rust (`src-tauri/src/`) e nos componentes/páginas (`frontend/src/`).
3. Verificar que todos os testes da história passam antes de avançar para o próximo checkpoint.

### Parallel Opportunities

- **Phase 1**: `T001` e `T002` atuam em `src-tauri/migrations/` e `frontend/src/layouts/` e podem rodar em paralelo.
- **Phase 2**: `T003` (`registry.rs`) e `T004` (`db.rs`) podem rodar em paralelo.
- **Phase 3 (US1)**: `T007` (Vitest) e `T008` (Cargo test) em paralelo; `T009` e `T010` em paralelo.
- **Phase 4 (US2)**: Todos os testes `T011`, `T012`, `T013`, `T014` e `T015` tocam arquivos distintos e podem ser escritos em paralelo.
- **Phase 5 (US3)**: A criação das 15 páginas Astro (`T026`, `T027`, `T028`) é totalmente paralelizável.

---

## Parallel Example: User Story 2

```bash
# 1. Escrever todos os testes de domínio e IPC da US2 em paralelo:
Task T011: "Escrever testes unitários em src-tauri/src/crud.rs validando CRUD em HORIZON_V2_ENTITIES"
Task T012: "Escrever testes unitários em src-tauri/src/special.rs validando hv2_merge e hv2_link"
Task T013: "Escrever testes de integração em src-tauri/src/import.rs e src-tauri/src/export.rs"
Task T014: "Escrever testes em src-tauri/src/sync_domain.rs e src-tauri/tests/sync_github.rs para horizon_v2"
Task T015: "Escrever testes unitários em frontend/tests/unit/api.test.js cobrindo /horizon-v2/*"
```

---

## Implementation Strategy

### MVP First (User Story 1 + User Story 2)

1. Concluir Phase 1 (Setup) e Phase 2 (Foundational).
2. Concluir Phase 3 (US1: Renomeação para **Horizon Legacy** e atualização dos testes de referência do novo `exports_canonical.zip`).
3. Concluir Phase 4 (US2: Isolamento completo de banco, import/export, CRUD e sync da nova aba **Horizon**).
4. Concluir Phase 5 (US3: 15 páginas de curadoria `/horizon/*` com todos os campos canônicos e `initiatives.enrichment` visíveis) e Phase 6 (validação de rotas embarcadas no binário Tauri).
