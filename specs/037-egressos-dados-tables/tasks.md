---

description: "Task list for feature 037-egressos-dados-tables implementation"
---

# Tasks: Tabelas, CRUD e Sincronização do Repositório `egressos-dados` (`alunos.json` e `data/*.json`)

**Input**: Design documents from `/specs/037-egressos-dados-tables/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/ (`ipc-commands.md`, `frontend-routes.md`), quickstart.md

**Tests**: OBRIGATÓRIOS — A Constituição (Princípio I: TDD Obrigatório; Princípio III: Testes Rigorosos) exige escrever os testes primeiro (vermelho), implementar até passar (verde) e refatorar.

**Organization**: Tasks agrupadas por user story (`US1` Navegação, filtro por categoria `PII`/`Internos`/`Publicáveis` e CRUD completo nas 39 tabelas de `egressos-dados`, `US2` Importação/exportação local de `/home/thiagodeps/egressos-dados` e snapshots em `/egressos-dados/controle`, `US3` Sincronização GitHub com `ifesserra-lab/egressos-dados` reutilizando o token em memória `EgressosDadosAuth`) para entrega e validação independentes.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- Núcleo Rust: `src-tauri/src/` e `src-tauri/migrations/` (testes unitários inline e testes de integração HTTP em `src-tauri/tests/sync_github.rs`)
- Front-end: `frontend/src/` (testes em `frontend/tests/unit/`)
- Contratos de referência: `specs/037-egressos-dados-tables/contracts/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirmar a baseline de testes na branch `037-egressos-dados-tables` antes de iniciar o ciclo TDD

- [X] T001 Executar `cargo test` em `src-tauri/` e `npm test` em `frontend/` para confirmar que a suíte existente passa 100% antes das alterações

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Migração SQLite `004_egressos_dados_tables.sql` (`egressos_dados_files` e `egressos_dados_rows`) e estrutura base do catálogo `EGRESSOS_DADOS_FILES` (39 datasets) em `src-tauri/src/egressos_dados_domain.rs`

**⚠️ CRITICAL**: Nenhuma user story pode começar antes da conclusão desta fase

- [X] T002 TDD (red → green): escrever teste unitário `migration_creates_egressos_dados_domain_tables` em `src-tauri/src/db.rs` verificando a criação das tabelas `egressos_dados_files` (`file_key`, `kind`, `primary_section`, `envelope_json`, `imported_at`) e `egressos_dados_rows` (`id`, `file_key` com FK `ON DELETE CASCADE`, `section_key`, `ord`, `record_key`, `search_text`, `raw_json`), atualizar a contagem total de tabelas em `src-tauri/src/db.rs` de 21 para 23, criar `src-tauri/migrations/004_egressos_dados_tables.sql` e registrá-la em `MIGRATIONS` em `src-tauri/src/db.rs`
- [X] T003 TDD (red → green): criar `src-tauri/src/egressos_dados_domain.rs` (e registrá-lo em `src-tauri/src/lib.rs`) com os enums `FileKind` (`ObjectSections`, `RootArray`, `RootMap`, `NestedMap`), `PublicationKind` (`Pii`, `Interno`, `Publicavel`), `DatasetRole` (`Entrada`, `Saida`), `EntityGroup` (`Alumni`, `Company`, `None`) e o catálogo canônico `EGRESSOS_DADOS_FILES: &[EgressosDadosFileDef; 39]` (`data-model.md` §2.2), acompanhado de teste unitário verificando que os 39 datasets (12 `pii`, 6 `interno`, 21 `publicavel`), seus `relative_path` (`alunos.json` na raiz e 38 `data/*.json`) e suas seções batem com `data-model.md`

**Checkpoint**: Migração `004` e catálogo dos 39 datasets prontos e testados — implementação das user stories liberada.

---

## Phase 3: User Story 1 - Navegar e executar CRUD completo nas 39 tabelas de `egressos-dados` (`alunos.json` + `data/*.json`) (Priority: P1) 🎯 MVP

**Goal**: Exibir na página desbloqueada `/egressos-dados` as 39 abas correspondentes a `alunos.json` e aos 38 arquivos JSON de `data/`, permitindo filtrar por classificação (`Todos`, `PII / Coorte & Coleta`, `Internos`, `Publicáveis`), alternar entre sub-seções internas (ex.: `alunos`, `cursos` e `convencoes` em `alunos.json`), pesquisar, ordenar e realizar CRUD completo (com preservação do campo `"id"` textual via `"_row_id"` e propagação opcional entre tabelas relacionadas).

**Independent Test**: Com sessão autorizada de `ifesserra-lab`, carregar registros nas tabelas de `egressos-dados`, filtrar as 39 abas por categoria (`PII`, `Interno`, `Publicável`), selecionar `alunos` (`alunos.json`), `_novos_reais`, `genero_map` e `vitrine`, criar, editar (com propagação opcional) e excluir registros e editar metadados de arquivo.

### Tests for User Story 1 ⚠️ (TDD — escrever primeiro, ver falhar)

- [X] T004 [P] [US1] Escrever testes unitários (red) em `src-tauri/src/egressos_dados_domain.rs` cobrindo `get_catalog`, `list_rows` (busca textual, ordenação por coluna, paginação), `get_row`, `create_row`, `update_row` (com e sem `propagate: true` para `Alumni` por `id`/`rotulo`/`label` e `Company` por `_key`/`empresa`), `delete_row` (com e sem `propagate: true`), `get_envelope` e `update_envelope`, validando que o campo `"id"` textual de `alunos.json` (`"id": "barbosa"`) e `_novos_reais.json` é preservado intacto enquanto `"_row_id"` expõe o ID numérico do SQLite
- [X] T005 [P] [US1] Escrever testes unitários (red) em `frontend/tests/unit/api.test.js` para as rotas de catálogo, linhas e envelope de `egressos-dados` (`GET /egressos-dados/catalog`, `GET /egressos-dados/files/:fileKey/rows`, `POST /egressos-dados/files/:fileKey/rows`, `GET /egressos-dados/files/:fileKey/envelope`, `PUT /egressos-dados/files/:fileKey/envelope`, `GET /egressos-dados/rows/:id`, `PUT /egressos-dados/rows/:id`, `DELETE /egressos-dados/rows/:id`) em `frontend/src/services/api.js` (`contracts/frontend-routes.md` §1)
- [X] T006 [P] [US1] Criar testes unitários (red) em `frontend/tests/unit/EgressosDadosTablesPage.test.jsx` verificando: (a) renderização das 39 abas com badges `pii`/`interno`/`publicavel` e `entrada`/`saida` e filtro por categoria (`Todos`, `PII`, `Internos`, `Publicáveis`); (b) exibição da coluna `"id"` textual em `alunos` (`"barbosa"`) e alternância para sub-seções (`cursos`, `convencoes`); (c) busca e ordenação por coluna; (d) criação, edição (usando `_row_id` sem apagar o `"id"` textual do formulário e com opção de propagação) e exclusão de registro; (e) edição de metadados do arquivo (`envelope`)

### Implementation for User Story 1

- [X] T007 [US1] Implementar `get_catalog`, `list_rows`, `get_row`, `create_row`, `update_row`, `delete_row`, `get_envelope`, `update_envelope` e a propagação cruzada (`EntityGroup::Alumni` incluindo `id`/`label`/`rotulo`/`perfil`/`_key` e `EntityGroup::Company`) em `src-tauri/src/egressos_dados_domain.rs` — verde para T004
- [X] T008 [US1] Implementar os comandos IPC `egressos_dados_get_catalog`, `egressos_dados_list_rows`, `egressos_dados_get_row`, `egressos_dados_create_row`, `egressos_dados_update_row`, `egressos_dados_delete_row`, `egressos_dados_get_envelope` e `egressos_dados_update_envelope` (todos protegidos por `state.require_egressos_dados_session()?`) em `src-tauri/src/commands.rs` e registrá-los em `tauri::generate_handler!` em `src-tauri/src/lib.rs` (`contracts/ipc-commands.md` §1)
- [X] T009 [P] [US1] Mapear as rotas de catálogo, CRUD de linhas e envelope de `/egressos-dados/*` em `frontend/src/services/api.js` — verde para T005
- [X] T010 [US1] Criar o componente `frontend/src/components/EgressosDadosTablesPage.jsx` com filtros de categoria (`Todos (39)`, `PII / Coorte & Coleta (12)`, `Internos (6)`, `Publicáveis (21)`), busca rápida de aba, badges de privacidade/papel/caminho relativo (`alunos.json` vs `data/*.json`), sub-abas de seção, tabela com busca/ordenação/paginação e modais de CRUD (baseados em `_row_id` preservando `"id"` textual) e de metadados — verde para T006
- [X] T011 [US1] Atualizar `frontend/src/components/EgressosDadosWorkspace.jsx` para atuar como gate de autorização da organização `ifesserra-lab` (exibindo barra compacta da sessão `@login` + badge `ifesserra-lab` + botão "Bloquear aba / Trocar token" no topo e renderizando `children` ou `EgressosDadosTablesPage` quando autorizado) e atualizar `frontend/tests/unit/EgressosDadosWorkspace.test.jsx` e `frontend/src/pages/egressos-dados/index.astro`

**Checkpoint**: User Story 1 funcional e testável isoladamente — as 39 tabelas de `egressos-dados` podem ser navegadas, filtradas por classificação e editadas via CRUD completo dentro da aba protegida `/egressos-dados`.

---

## Phase 4: User Story 2 - Importar e exportar localmente o repositório `egressos-dados` (`alunos.json` + pasta `data/`) (Priority: P2)

**Goal**: Disponibilizar a página `/egressos-dados/controle` no menu lateral de `EgressosDadosDashboard.astro` permitindo importar e exportar localmente os 39 arquivos JSON (`alunos.json` na raiz + 38 arquivos em `data/*.json`) a partir de `/home/thiagodeps/egressos-dados`, com suporte *round-trip* aos 4 formatos estruturais (`object_sections`, `root_array`, `root_map`, `nested_map`), validação atômica e snapshots exclusivos (`egressos-dados-*`).

**Independent Test**: Acessar `/egressos-dados/controle`, importar `/home/thiagodeps/egressos-dados`, verificar que os 39 arquivos JSON são carregados sem perda e exportar para uma pasta temporária confirmando que `alunos.json` na raiz e os 38 arquivos em `data/*.json` são idênticos semanticamente aos originais.

### Tests for User Story 2 ⚠️ (TDD — escrever primeiro, ver falhar)

- [X] T012 [P] [US2] Escrever testes unitários (red) em `src-tauri/src/egressos_dados_domain.rs` verificando: (a) `import_dir` e `export_dir` com gerador sintético dos 39 arquivos cobrindo `ObjectSections`, `RootArray` de objetos (`_novos_reais.json`), `RootArray` de escalares (`_empresas_lista.json`), `RootMap` de escalares (`genero_map.json`), `RootMap` de objetos (`empresas_porte.json`), `NestedMap` (`fontes_estado.json`) e seções vazias (`src_extensao.json`); (b) *round-trip* real contra `/home/thiagodeps/egressos-dados` (quando presente na máquina) comprovando 100% de equivalência JSON nos 39 arquivos (`SC-002`); (c) resolução automática quando o usuário passa `/home/thiagodeps/egressos-dados/data` em vez da raiz; (d) falha atômica quando falta `alunos.json` ou qualquer arquivo de `data/`; (e) criação de snapshot `egressos-dados-pre-import` ao reimportar e isolamento total em relação a `egressos_files` (`FR-002`)
- [X] T013 [P] [US2] Escrever testes unitários (red) em `frontend/tests/unit/api.test.js` para as rotas `POST /egressos-dados/import`, `POST /egressos-dados/export` e `GET /egressos-dados/snapshots` em `frontend/src/services/api.js`
- [X] T014 [P] [US2] Criar testes unitários (red) em `frontend/tests/unit/EgressosDadosDataControl.test.jsx` verificando a importação local (com sugestão de `/home/thiagodeps/egressos-dados`), exportação local e listagem de snapshots `egressos-dados-*`
- [X] T015 [P] [US2] Atualizar o teste de rotas (`ROUTES`) em `src-tauri/src/lib.rs` adicionando `"/egressos-dados/controle"` e ajustando `contagem_de_rotas_bate_com_o_menu_lateral` de 24 para 25

### Implementation for User Story 2

- [X] T016 [US2] Implementar `resolve_repo_root`, `parse_and_validate_single_file` (suportando `ObjectSections`, `RootArray`, `RootMap` de objetos/escalares e `NestedMap`), `import_dir`, `reconstruct_file_json`, `export_files_in_memory` e `export_dir` em `src-tauri/src/egressos_dados_domain.rs` — verde para T012
- [X] T017 [US2] Implementar os comandos IPC `egressos_dados_import_dir`, `egressos_dados_export_dir` e `egressos_dados_list_snapshots` (protegidos por `state.require_egressos_dados_session()?`) em `src-tauri/src/commands.rs` e registrá-los em `tauri::generate_handler!` em `src-tauri/src/lib.rs` (`contracts/ipc-commands.md` §2)
- [X] T018 [P] [US2] Adicionar as rotas `POST /egressos-dados/import`, `POST /egressos-dados/export` e `GET /egressos-dados/snapshots` em `frontend/src/services/api.js` — verde para T013
- [X] T019 [US2] Criar o componente `frontend/src/components/EgressosDadosDataControl.jsx` com cartões de resumo da base (39 arquivos: `alunos.json` + 38 em `data/`), importação local (com atalho para `/home/thiagodeps/egressos-dados`), exportação local e histórico de snapshots `egressos-dados-*` — verde para T014
- [X] T020 [US2] Atualizar `frontend/src/layouts/EgressosDadosDashboard.astro` (incluindo os links `Tabelas` → `/egressos-dados` e `Controle de Dados` → `/egressos-dados/controle` no menu lateral) e criar a página `frontend/src/pages/egressos-dados/controle.astro` protegida pelo gate `EgressosDadosWorkspace.jsx`

**Checkpoint**: User Stories 1 e 2 funcionais — o curador importa `/home/thiagodeps/egressos-dados`, edita qualquer uma das 39 tabelas e exporta de volta para o disco com fidelidade *round-trip* de 100%.

---

## Phase 5: User Story 3 - Baixar e enviar os 39 arquivos JSON de `egressos-dados` para o repositório GitHub `ifesserra-lab/egressos-dados` (Priority: P3)

**Goal**: Permitir em `/egressos-dados/controle` baixar (download) e enviar (upload em commit único atômico) os 39 arquivos JSON (`alunos.json` na raiz + 38 arquivos em `data/*.json`) diretamente contra o repositório privado `ifesserra-lab/egressos-dados` (branch `main`), reutilizando automaticamente o Token do GitHub mantido em memória na sessão `EgressosDadosAuth`.

**Independent Test**: Na página `/egressos-dados/controle`, executar Download do repositório `ifesserra-lab/egressos-dados` sem precisar redigitar o token, confirmar a carga dos 39 arquivos JSON e executar Upload verificando que um único commit atômico é criado com os 39 blobs (`alunos.json` e `data/*.json`).

### Tests for User Story 3 ⚠️ (TDD — escrever primeiro, ver falhar)

- [X] T021 [P] [US3] Escrever testes unitários (red) em `src-tauri/src/sync_domain.rs` para `ProjectId::EgressosDados` (`as_str() == "egressos_dados"`), defaults (`repo: "ifesserra-lab/egressos-dados"`, `branch: "main"`, `path: "."`) e `normalize_egressos_dados_source`
- [X] T022 [P] [US3] Escrever testes de integração `wiremock` (red) em `src-tauri/tests/sync_github.rs`: (a) `download_egressos_dados_repo_baixa_39_jsons_raiz_e_data_e_cria_snapshot` verificando o download atômico de `alunos.json` + 38 `data/*.json` usando o token da sessão; (b) `upload_egressos_dados_repo_publica_39_blobs_em_commit_unico_atomico` verificando o envio dos 39 arquivos via Git Data API em um único commit
- [X] T023 [P] [US3] Escrever testes unitários (red) em `frontend/tests/unit/api.test.js` e `frontend/tests/unit/EgressosDadosDataControl.test.jsx` para as rotas `GET /egressos-dados/sync/config`, `PUT /egressos-dados/sync/config`, `POST /egressos-dados/sync/download` e `POST /egressos-dados/sync/upload` e para o painel de sincronização GitHub integrado em `EgressosDadosDataControl.jsx`

### Implementation for User Story 3

- [X] T024 [US3] Implementar `ProjectId::EgressosDados`, defaults (`DEFAULT_EGRESSOS_DADOS_REPO = "ifesserra-lab/egressos-dados"`, `DEFAULT_EGRESSOS_DADOS_BRANCH = "main"`, `DEFAULT_EGRESSOS_DADOS_PATH = "."`) e `normalize_egressos_dados_source` em `src-tauri/src/sync_domain.rs` — verde para T021
- [X] T025 [US3] Implementar `download_egressos_dados_repo` e `upload_egressos_dados_repo` em `src-tauri/src/sync_github.rs` (operando sobre os 39 caminhos relativos `alunos.json` + `data/*.json` de `EGRESSOS_DADOS_FILES`) — verde para T022
- [X] T026 [US3] Implementar os comandos IPC `egressos_dados_sync_get_config`, `egressos_dados_sync_set_config`, `egressos_dados_sync_download` e `egressos_dados_sync_upload` em `src-tauri/src/commands.rs` (reutilizando `state.require_egressos_dados_session()?.token` em memória) e registrá-los em `tauri::generate_handler!` em `src-tauri/src/lib.rs` (`contracts/ipc-commands.md` §3)
- [X] T027 [US3] Adicionar as rotas `/egressos-dados/sync/*` em `frontend/src/services/api.js` e integrar a seção de Sincronização GitHub (`ifesserra-lab/egressos-dados`, Download e Upload com mensagem de commit) em `frontend/src/components/EgressosDadosDataControl.jsx` — verde para T023

**Checkpoint**: Todas as 3 User Stories completas e independentemente testáveis.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Build estático do front-end, verificação das 25 rotas do asset resolver do Tauri e validação ponta a ponta da suíte completa

- [X] T028 Executar `npm test` e `npm run build` em `frontend/` e `cargo test` em `src-tauri/`, confirmando 100% dos testes unitários e de integração passando (incluindo o *round-trip* dos 39 arquivos de `/home/thiagodeps/egressos-dados` e a resolução das 25 rotas em `src-tauri/src/lib.rs`)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1: T001)**: Sem dependências — início imediato.
- **Foundational (Phase 2: T002–T003)**: Depende do Setup — **BLOQUEIA** todas as user stories.
- **User Story 1 (Phase 3: T004–T011)**: Depende da Foundational (Phase 2) — entrega o MVP das 39 tabelas com filtros por categoria e CRUD completo em `/egressos-dados`.
- **User Story 2 (Phase 4: T012–T020)**: Depende da Foundational (Phase 2) — entrega a importação/exportação local de `/home/thiagodeps/egressos-dados` e snapshots em `/egressos-dados/controle`.
- **User Story 3 (Phase 5: T021–T027)**: Depende da US2 (`import_dir` e `export_files_in_memory`) — entrega o Download e Upload atômico dos 39 arquivos contra `ifesserra-lab/egressos-dados`.
- **Polish (Phase 6: T028)**: Depende da conclusão das User Stories 1 a 3.

### User Story Dependencies

```text
Phase 1 (Setup: T001)
  └── Phase 2 (Foundational: Migration 004 + Catálogo de 39 Datasets: T002–T003)
        ├── Phase 3 (US1: 39 Tabelas + Filtros PII/Interno/Publicável + CRUD: T004–T011) 🎯 MVP
        └── Phase 4 (US2: Import/Export Local /home/thiagodeps/egressos-dados + Snapshots: T012–T020)
              └── Phase 5 (US3: Sync GitHub ifesserra-lab/egressos-dados com Token em Memória: T021–T027)
                    └── Phase 6 (Polish & Validação Final: T028)
```

### Parallel Opportunities

- **Phase 3 (US1)**: `T004` (testes Rust), `T005` (testes `api.test.js`) e `T006` (testes `EgressosDadosTablesPage.test.jsx`) tocam arquivos distintos e podem rodar em paralelo.
- **Phase 4 (US2)**: `T012` (testes Rust), `T013` (`api.test.js`), `T014` (`EgressosDadosDataControl.test.jsx`) e `T015` (`lib.rs`) podem rodar em paralelo.
- **Phase 5 (US3)**: `T021` (`sync_domain.rs`), `T022` (`tests/sync_github.rs`) e `T023` (testes Vitest) podem rodar em paralelo.

---

## Parallel Example: User Story 1

```bash
# Escrever todos os testes da User Story 1 em paralelo:
Task T004: "Escrever testes unitários (red) em src-tauri/src/egressos_dados_domain.rs"
Task T005: "Escrever testes unitários (red) em frontend/tests/unit/api.test.js"
Task T006: "Criar testes unitários (red) em frontend/tests/unit/EgressosDadosTablesPage.test.jsx"
```

---

## Implementation Strategy

### MVP First (User Story 1 + User Story 2 Local Import)

1. Concluir Phase 1 (Setup) e Phase 2 (Foundational: `004_egressos_dados_tables.sql` + `EGRESSOS_DADOS_FILES`).
2. Concluir Phase 3 (US1: 39 tabelas e CRUD em `/egressos-dados`) e Phase 4 (US2: importação/exportação de `/home/thiagodeps/egressos-dados` em `/egressos-dados/controle`).
3. Concluir Phase 5 (US3: sincronização GitHub com `ifesserra-lab/egressos-dados`) e Phase 6 (validação completa).
