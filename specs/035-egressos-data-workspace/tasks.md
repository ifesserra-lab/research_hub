---

description: "Task list for feature 035-egressos-data-workspace implementation"
---

# Tasks: Espaço de Dados do Projeto Egressos com CRUD e Sincronização GitHub

**Input**: Design documents from `/specs/035-egressos-data-workspace/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/ (`egressos-dados-json.md`, `ipc-commands.md`, `frontend-routes.md`), quickstart.md

**Tests**: OBRIGATÓRIOS — A Constituição (Princípio I: TDD Obrigatório; Princípio III: Testes Rigorosos) exige escrever o teste primeiro (vermelho), implementar até passar (verde) e refatorar.

**Organization**: Tasks agrupadas por user story (`US1` seleção e página dedicada, `US2` import/export local da pasta `dados/`, `US3` CRUD nas 21 tabelas + propagação opcional, `US4` download/upload GitHub por token) para permitir implementação e validação independentes.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- Núcleo Rust: `src-tauri/src/` (+ migrações em `src-tauri/migrations/`, testes de integração em `src-tauri/tests/`)
- Front-end: `frontend/src/` (testes em `frontend/tests/unit/`)
- Contratos de referência: `specs/035-egressos-data-workspace/contracts/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Esqueleto do módulo de domínio e confirmação da baseline de testes na branch `035-egressos-data-workspace`

- [X] T001 Criar o módulo esqueleto `src-tauri/src/egressos_domain.rs` (com documentação referenciando `research.md` R1–R5) e registrá-lo com `pub mod egressos_domain;` em `src-tauri/src/lib.rs`
- [X] T002 Executar `cargo test` em `src-tauri/` e `npm test` em `frontend/` para confirmar que a suíte existente passa 100% antes das alterações

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Esquema SQLite (`003_egressos_tables.sql`) e catálogo estático dos 21 arquivos JSON (`EGRESSOS_FILES`) que bloqueiam todas as user stories

**⚠️ CRITICAL**: Nenhuma user story pode começar antes da conclusão desta fase

- [X] T003 [P] TDD (red): escrever teste `migration_creates_egressos_domain_tables` e atualizar `migration_creates_nineteen_tables` (19 → 21 tabelas) em `src-tauri/src/db.rs` verificando a criação de `egressos_files` (`file_key TEXT PRIMARY KEY`, `kind TEXT NOT NULL`, `primary_section TEXT NOT NULL`, `envelope_json TEXT NOT NULL`, `imported_at TEXT NULL`) e `egressos_rows` (`id INTEGER PRIMARY KEY AUTOINCREMENT`, `file_key TEXT NOT NULL REFERENCES egressos_files(file_key) ON DELETE CASCADE`, `section_key TEXT NOT NULL`, `ord INTEGER NOT NULL`, `record_key TEXT NULL`, `search_text TEXT NOT NULL DEFAULT ''`, `raw_json TEXT NOT NULL`) e cascade ao deletar de `egressos_files`
- [X] T004 Criar `src-tauri/migrations/003_egressos_tables.sql` com as tabelas `egressos_files` e `egressos_rows` e índices `idx_egressos_rows_file_section_ord ON egressos_rows(file_key, section_key, ord)` e `idx_egressos_rows_record_key ON egressos_rows(record_key)` (constraints idênticas ao `data-model.md` §1) e registrá-la em `MIGRATIONS` em `src-tauri/src/db.rs` — verde para T003
- [X] T005 [P] TDD (red → green): escrever testes unitários e implementar o catálogo estático `EGRESSOS_FILES: &[EgressosFileDef; 21]` em `src-tauri/src/egressos_domain.rs` cobrindo exatamente os 21 arquivos de `data-model.md` §2 (`analise`, `cargos_ao_longo_do_tempo`, `codigofonte_2026`, `codigofonte_historico`, `consolidado`, `egressos_perfil`, `empresas_aliases`, `empresas_linkedin_data`, `empresas_linkedin_urls`, `empresas_porte`, `fapes_fomento`, `ibge_series`, `impacto`, `mapa_mundi`, `panorama`, `recorte_cursos`, `renda_por_senioridade`, `salario_minimo`, `so_benchmarks`, `trajetoria`, `vitrine`) com `file_key`, `file_name`, `title`, `kind` (`object_sections` | `root_map` | `nested_map`), `primary_section`, `sections`, `columns` e `entity_group` (`alumni` | `company` | `none`)

**Checkpoint**: Esquema SQLite e catálogo das 21 abas prontos e testados — implementação das user stories liberada.

---

## Phase 3: User Story 1 - Selecionar e acessar o espaço de trabalho exclusivo do projeto Egressos (Priority: P1) 🎯 MVP

**Goal**: Exibir o projeto **Egressos** na tela `/projects` ao lado de **Horizon** e **SRC**, abrir a área dedicada (`/egressos` e `/egressos/tabelas`) com menu lateral próprio e permitir alternar entre os três projetos com aviso de alterações não salvas.

**Independent Test**: Autenticar no app, verificar os 3 cartões em `/projects` (`Horizon`, `SRC`, `Egressos`), clicar em `Egressos` para abrir `/egressos` e `/egressos/tabelas` sem qualquer dado ou aba de Horizon/SRC, e usar "Trocar projeto" para retornar a `/projects`.

### Tests for User Story 1 ⚠️ (TDD — escrever primeiro, ver falhar)

- [X] T006 [P] [US1] Escrever testes em `frontend/tests/unit/ProjectSelector.test.jsx` verificando a renderização dos 3 projetos (`Horizon`, `SRC`, `Egressos`), navegação para `/egressos` ao clicar no cartão Egressos e confirmação de segurança quando `pendingWarning` está ativo
- [X] T007 [P] [US1] Atualizar o teste de rotas (`ROUTES`) em `src-tauri/src/lib.rs` exigindo `/egressos` e `/egressos/tabelas` resolvendo no asset resolver do Tauri e atualizando `contagem_de_rotas_bate_com_o_menu_lateral` de 21 para 23

### Implementation for User Story 1

- [X] T008 [P] [US1] Atualizar `frontend/src/components/ProjectSelector.jsx` (layout em grade responsiva para 3 cartões) e `frontend/src/pages/projects.astro` adicionando o cartão `{ id: 'egressos', title: 'Egressos', description: 'Painel de carreira e renda dos egressos do IFES Campus Serra (21 tabelas JSON da pasta dados/)', href: '/egressos' }` e escutando o evento `egressos:dirty` — verde para T006
- [X] T009 [US1] Criar o layout `frontend/src/layouts/EgressosDashboard.astro` com guard de sessão, cabeçalho "Projeto Egressos", links para `/egressos` (Resumo) e `/egressos/tabelas` (21 Tabelas de `dados/`), atalho "↩ Trocar projeto" (`/projects`) com verificação de `egressos:dirty` e botão Sair
- [X] T010 [US1] Criar as páginas iniciais `frontend/src/pages/egressos/index.astro` (Resumo da base Egressos) e `frontend/src/pages/egressos/tabelas.astro` usando `EgressosDashboard.astro`
- [X] T011 [US1] Compilar o front-end (`npm run build` em `frontend/`) e confirmar que os testes de resolução das 23 rotas em `src-tauri/src/lib.rs` passam — verde para T007

**Checkpoint**: User Story 1 funcional e testável isoladamente — seleção entre Horizon, SRC e Egressos e navegação dedicada operacional.

---

## Phase 4: User Story 2 - Importar e exportar localmente a pasta de JSONs do Egressos (Priority: P2)

**Goal**: Importar diretamente a pasta local de 21 arquivos JSON (sugerindo `/home/thiagodeps/egressos/dados`), validar atomicamente todos os 21 arquivos, criar snapshot `egressos-pre-import-*`, substituir a base Egressos e exportar os 21 arquivos JSON preservando 100% da estrutura original (*round-trip*).

**Independent Test**: Importar `/home/thiagodeps/egressos/dados`, verificar `total_files == 21` e contagens por tabela, exportar para um diretório temporário sem editar e confirmar equivalência `serde_json::Value` 100% idêntica nos 21 arquivos.

### Tests for User Story 2 ⚠️ (TDD — escrever primeiro, ver falhar)

- [X] T012 [P] [US2] Escrever testes de validação e snapshot (red) em `src-tauri/src/egressos_domain.rs`: (a) pasta válida com os 21 arquivos JSON carrega `total_files == 21` e popula `egressos_files` e `egressos_rows`; (b) pasta com arquivo obrigatório faltando ou JSON malformado retorna `AppError::Validation` citando o nome do arquivo e preserva a base anterior intacta (`FR-005`); (c) arquivos extras como `.DS_Store` ou `README.md` são ignorados; (d) reimportar sobre base existente cria snapshot `egressos-pre-import-*.db` em `snapshots/` (`FR-006`)
- [X] T013 [P] [US2] Escrever teste de paridade *round-trip* (red) em `src-tauri/src/egressos_domain.rs`: importar os 21 arquivos reais de `/home/thiagodeps/egressos/dados` → exportar para `tempfile::tempdir()` sem edições → comparar cada um dos 21 arquivos exportados contra o original como `serde_json::Value` (100% idênticos, contrato `C5`/`SC-003`)
- [X] T014 [P] [US2] Escrever testes unitários (red) em `frontend/tests/unit/api.test.js` para as rotas `POST /egressos/import`, `POST /egressos/export` e `GET /egressos/catalog` em `frontend/src/services/api.js`
- [X] T015 [P] [US2] Escrever testes unitários (red) em `frontend/tests/unit/EgressosDataControl.test.jsx` verificando importação da pasta padrão `/home/thiagodeps/egressos/dados`, seleção manual de pasta, exportação local, exibição de progresso `egressos-import://progress` e resumo com snapshot

### Implementation for User Story 2

- [X] T016 [US2] Implementar `import_dir`, `export_dir`, `export_files_in_memory` e `get_catalog` em `src-tauri/src/egressos_domain.rs` (decomposição e reconstrução fiel de `object_sections`, `root_map` e `nested_map` preservando `envelope_json` e ordem `ord`, mais snapshot via `rusqlite::backup` com prefixo `egressos-pre-import-`) — verde para T012 e T013
- [X] T017 [US2] Criar os comandos `import_egressos_dir` (com `blocking_pick_folder` sugerindo `/home/thiagodeps/egressos/dados` quando `path` é `None` e emitindo `egressos-import://progress`), `export_egressos_dir` e `egressos_get_catalog` em `src-tauri/src/commands.rs` e registrá-los em `invoke_handler` em `src-tauri/src/lib.rs` (`contracts/ipc-commands.md` §1)
- [X] T018 [US2] Adicionar as rotas `POST /egressos/import`, `POST /egressos/export` e `GET /egressos/catalog` em `frontend/src/services/api.js` antes dos padrões genéricos `/:entity` — verde para T014
- [X] T019 [US2] Criar o componente `frontend/src/components/EgressosDataControl.jsx` (botões para importar direto de `/home/thiagodeps/egressos/dados`, escolher outra pasta local e exportar pasta `dados/`, ouvindo `egressos-import://progress` e disparando `egressos:imported`) — verde para T015
- [X] T020 [US2] Integrar `EgressosDataControl.jsx` e os indicadores de resumo da base (arquivos carregados, total de registros, data da última importação e atalhos para as 21 tabelas) em `frontend/src/pages/egressos/index.astro`

**Checkpoint**: User Stories 1 e 2 funcionais e testáveis independentemente — pasta `/home/thiagodeps/egressos/dados` importa e exporta com fidelidade de 100%.

---

## Phase 5: User Story 3 - Executar CRUD completo nas tabelas do Egressos (Priority: P3)

**Goal**: Disponibilizar navegação 1:1 para as 21 abas de arquivos JSON (com sub-abas para arquivos com múltiplas seções), listagem com busca e ordenação, CRUD completo de registros, editor de metadados do arquivo (`envelope`) e escolha do curador entre edição local da aba (A) ou propagação por chave compartilhada de Egresso/Empresa entre tabelas relacionadas (B).

**Independent Test**: Abrir `/egressos/tabelas`, alternar entre abas de diferentes tipos (`egressos_perfil`, `vitrine`, `empresas_porte`, `codigofonte_historico`), buscar e ordenar registros, criar/editar/excluir um registro com `propagate = false` e com `propagate = true`, editar metadados de um arquivo e verificar que todas as mudanças persistem e aparecem na exportação.

### Tests for User Story 3 ⚠️ (TDD — escrever primeiro, ver falhar)

- [X] T021 [P] [US3] Escrever testes de CRUD e envelope (red) em `src-tauri/src/egressos_domain.rs`: `list_rows` (busca por `search_text`, ordenação asc/desc por coluna JSON, paginação `limit`/`offset`), `get_row`, `create_row` (`ord = MAX(ord) + 1`, rejeição de `_key` vazio ou duplicado em mapas com `AppError::Conflict`), `update_row` e `delete_row` (`propagate = false`), e `get_envelope`/`update_envelope` preservando as seções na exportação (`FR-008`, `FR-009`)
- [X] T022 [P] [US3] Escrever testes de propagação cruzada (red) em `src-tauri/src/egressos_domain.rs` (`FR-008a`): (a) atualizar egresso (`label = "A"`) em `egressos_perfil` com `propagate = true` sincroniza `nome`, `curso`, `cargo`/`cargo_atual`, `empresa`/`empresa_atual`/`empregador_atual` e `lugar`/`local_atual` em `vitrine:perfis`, `panorama:coorte` e `analise:rows`; (b) atualizar empresa (`_key = "AEVO"`) em `empresas_porte` com `propagate = true` sincroniza os campos compartilhados em `empresas_aliases`, `empresas_linkedin_data`, `empresas_linkedin_urls` e `vitrine:empresas*`; (c) excluir com `propagate = true` remove os registros irmãos nas tabelas do mesmo grupo
- [X] T023 [P] [US3] Escrever testes unitários (red) em `frontend/tests/unit/api.test.js` para as rotas `GET|PUT /egressos/files/:fileKey/envelope`, `GET|POST /egressos/files/:fileKey/rows` e `GET|PUT|DELETE /egressos/rows/:id` (incluindo o parâmetro `propagate`)
- [X] T024 [P] [US3] Escrever testes unitários (red) em `frontend/tests/unit/EgressosTablesPage.test.jsx` verificando: renderização das 21 abas principais, troca de sub-abas em arquivos multi-seção (ex.: `vitrine`), busca e ordenação, criação/edição/exclusão de registro, exibição da opção de propagação (`propagate`) para abas dos grupos `alumni` e `company` e edição de metadados do arquivo

### Implementation for User Story 3

- [X] T025 [US3] Implementar `list_rows`, `get_row`, `create_row`, `update_row` (com suporte a `propagate: bool` para os grupos `Alumni` e `Company`), `delete_row` (com suporte a `propagate: bool`), `get_envelope` e `update_envelope` em `src-tauri/src/egressos_domain.rs` — verde para T021 e T022
- [X] T026 [US3] Criar os comandos IPC `egressos_list_rows`, `egressos_get_row`, `egressos_create_row`, `egressos_update_row`, `egressos_delete_row`, `egressos_get_envelope` e `egressos_update_envelope` em `src-tauri/src/commands.rs` e registrá-los em `src-tauri/src/lib.rs` (`contracts/ipc-commands.md` §2)
- [X] T027 [US3] Adicionar as rotas de CRUD e envelope (`/egressos/files/:fileKey/envelope`, `/egressos/files/:fileKey/rows`, `/egressos/rows/:id`) em `frontend/src/services/api.js` — verde para T023
- [X] T028 [US3] Criar `frontend/src/components/EgressosTablesPage.jsx` (navegação das 21 abas + seletor de sub-seções + grade com busca/ordenação/paginação + modal de criar/editar registro com checkbox de propagação `FR-008a` + modal de metadados do arquivo `FR-009` + emissão de `egressos:dirty`) e montá-lo em `frontend/src/pages/egressos/tabelas.astro` e nos links do menu lateral de `frontend/src/layouts/EgressosDashboard.astro` — verde para T024

**Checkpoint**: User Stories 1, 2 e 3 completas — todas as 21 tabelas possuem CRUD completo, edição de metadados e propagação opcional.

---

## Phase 6: User Story 4 - Baixar e enviar os dados do Egressos para o GitHub via token (Priority: P4)

**Goal**: Permitir baixar (download) os 21 arquivos `.json` da pasta `dados/` diretamente de um repositório/branch do GitHub e enviar (upload) o estado atual da base Egressos gerando os 21 arquivos `.json` em um único commit atômico na branch configurada, autenticando por token pessoal.

**Independent Test**: Em um servidor mock (`wiremock`) ou repositório GitHub, configurar destino (`thiagodeps/egressos`, branch `main`, pasta `dados`), baixar os 21 arquivos `.json` verificando a importação completa na base Egressos, editar um registro e executar o upload verificando a criação dos 21 blobs, 1 tree e 1 commit na branch.

### Tests for User Story 4 ⚠️ (TDD — escrever primeiro, ver falhar)

- [X] T029 [P] [US4] Escrever testes unitários (red) em `src-tauri/src/sync_domain.rs`: (a) `PROJECTS` contém `"egressos"`; (b) `check_upload_allowed("egressos")` retorna `Ok(())`; (c) `validate_path("egressos", "dados")` aceita caminho de diretório sem `".."` e rejeita vazio ou `".."`; (d) `normalize_egressos_source` aceita URL de pasta do GitHub (`https://github.com/{owner}/{repo}/tree/{ref}/{dir}` ou `https://github.com/{owner}/{repo}`) extraindo `(owner, repo, ref, dir)`
- [X] T030 [US4] Escrever testes de integração `wiremock` (red) em `src-tauri/tests/sync_github.rs`: (a) `download_egressos_dir` baixa os 21 arquivos `.json` de `/repos/{owner}/{repo}/contents/{dir}/{file_name}?ref={branch}`, valida-os atomicamente, cria snapshot `egressos-pre-import-*` e importa na base Egressos (abortando sem alterar a base se qualquer um dos 21 retornar 404/401/JSON inválido); (b) `upload_egressos_dir` gera os 21 arquivos `.json` da base atual, posta os 21 blobs em `/git/blobs`, lê `/git/ref/heads/{branch}` (falhando com `not_found` se a branch não existir), posta 1 tree com as 21 entradas `{dir}/{file_name}`, cria 1 commit e avança a ref da branch (`FR-011`, `FR-012`)
- [X] T031 [P] [US4] Escrever testes unitários (red) em `frontend/tests/unit/SyncPanel.test.jsx` verificando o comportamento de `SyncPanel` para `project="egressos"` com `allowUpload={true}` (defaults `thiagodeps/egressos`, branch `main`, caminho `dados`, download da pasta e confirmação de upload dos 21 arquivos JSON)

### Implementation for User Story 4

- [X] T032 [US4] Atualizar `src-tauri/src/sync_domain.rs` adicionando `"egressos"` em `PROJECTS`, habilitando `check_upload_allowed("egressos")`, ajustando `validate_path` para aceitar caminho de pasta no projeto `"egressos"` e implementando `normalize_egressos_source` — verde para T029
- [X] T033 [US4] Implementar `download_egressos_dir`, `check_directory_destination` e `upload_directory` (21 blobs + 1 tree + 1 commit atômico via Git Data API) em `src-tauri/src/sync_github.rs` e conectar a ramificação `project == "egressos"` nos comandos `github_download`, `github_check_destination` e `github_upload` em `src-tauri/src/commands.rs` — verde para T030
- [X] T034 [US4] Atualizar `frontend/src/components/SyncPanel.jsx` para suportar `project="egressos"` (textos e placeholders adaptados para pasta `dados/`, exibição de `total_files` e `total_rows` no download, disparo de `egressos:imported` e envio habilitado com `allowUpload={true}`) — verde para T031
- [X] T035 [US4] Montar `<SyncPanel client:load project="egressos" allowUpload={true} />` em `frontend/src/pages/egressos/index.astro` conectado ao resumo da base Egressos

**Checkpoint**: Todas as 4 User Stories estão completas e funcionam de forma independente e integrada.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Auditoria de isolamento entre projetos, proteção do token e validação ponta a ponta do `quickstart.md`

- [X] T036 [P] Escrever teste de isolamento entre os 3 projetos e auditoria de segredo em `src-tauri/tests/sync_github.rs` e `src-tauri/src/egressos_domain.rs`: confirmar que importar/editar/exportar no Egressos não altera nenhuma contagem do Horizon nem do SRC (`FR-002`, `FR-003`, `SC-001`) e que nenhuma falha de sincronização do Egressos expõe o valor do token (`FR-013`)
- [X] T037 Executar a bateria completa de verificação: `cargo test` em `src-tauri/`, `npm test` e `npm run build` em `frontend/`, confirmando 0 erros e 23 rotas válidas no teste de asset resolver
- [X] T038 Executar a validação de `specs/035-egressos-data-workspace/quickstart.md` contra a pasta real `/home/thiagodeps/egressos/dados` (importação < 10 s, *round-trip* dos 21 arquivos JSON com `diff` semântico zero, CRUD e sincronização GitHub)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1: T001–T002)**: Sem dependências — início imediato.
- **Foundational (Phase 2: T003–T005)**: Depende do Setup — **BLOQUEIA** todas as user stories.
- **User Story 1 (Phase 3: T006–T011)**: Depende apenas da Foundational (Phase 2).
- **User Story 2 (Phase 4: T012–T020)**: Depende da Foundational (Phase 2) e integra-se na página `/egressos` criada na US1.
- **User Story 3 (Phase 5: T021–T028)**: Depende da Foundational (Phase 2) e usa a carga de dados da US2 para validação em tela.
- **User Story 4 (Phase 6: T029–T035)**: Depende da US2 (`import_dir` e `export_files_in_memory`) para conectar o download e o upload dos 21 arquivos JSON ao GitHub.
- **Polish (Phase 7: T036–T038)**: Depende da conclusão das User Stories 1 a 4.

### User Story Dependencies

```text
Phase 1 (Setup)
  └── Phase 2 (Foundational: 003_egressos_tables.sql + EGRESSOS_FILES)
        ├── Phase 3 (US1: Seleção de Projeto + Área /egressos) 🎯 MVP
        ├── Phase 4 (US2: Import/Export Local da Pasta dados/)
        │     ├── Phase 5 (US3: CRUD nas 21 Tabelas + Propagação Opcional)
        │     └── Phase 6 (US4: Download/Upload GitHub por Token)
        └── Phase 7 (Polish & Validação Final)
```

### Parallel Opportunities

- **Phase 2**: `T003` (testes de migração em `db.rs`) e `T005` (catálogo `EGRESSOS_FILES` em `egressos_domain.rs`) podem rodar em paralelo.
- **Phase 3 (US1)**: `T006` (`ProjectSelector.test.jsx`) e `T007` (`lib.rs` `ROUTES`) podem rodar em paralelo.
- **Phase 4 (US2)**: `T012`/`T013` (testes Rust de import/round-trip) podem rodar em paralelo com `T014` (`api.test.js`) e `T015` (`EgressosDataControl.test.jsx`).
- **Phase 5 (US3)**: `T021`/`T022` (testes Rust de CRUD e propagação) podem rodar em paralelo com `T023` (`api.test.js`) e `T024` (`EgressosTablesPage.test.jsx`).
- **Phase 6 (US4)**: `T029` (testes em `sync_domain.rs`) e `T031` (`SyncPanel.test.jsx`) podem rodar em paralelo antes da integração em `T033`/`T034`.

---

## Parallel Example: User Story 3

```bash
# Executar tarefas de teste independentes da User Story 3 em paralelo:
Task T021: "Escrever testes de CRUD e envelope (red) em src-tauri/src/egressos_domain.rs"
Task T023: "Escrever testes unitários (red) em frontend/tests/unit/api.test.js para rotas CRUD"
Task T024: "Escrever testes unitários (red) em frontend/tests/unit/EgressosTablesPage.test.jsx"
```

---

## Implementation Strategy

### MVP First (User Story 1 + User Story 2)

1. Concluir Phase 1 (Setup) e Phase 2 (Foundational).
2. Concluir Phase 3 (US1): seleção do projeto Egressos em `/projects` e navegação isolada em `/egressos`.
3. Concluir Phase 4 (US2): importação e exportação local da pasta `/home/thiagodeps/egressos/dados` com validação atômica e *round-trip* 100% fiel.
4. **Validar incremento**: testar navegação e ciclo importar → exportar localmente.

### Incremental Delivery

1. **Incremento 1 (US1)**: Página dedicada `/egressos` e seletor de 3 projetos (`Horizon`, `SRC`, `Egressos`).
2. **Incremento 2 (US2)**: Importação e exportação local dos 21 arquivos JSON da pasta `dados/` com snapshot.
3. **Incremento 3 (US3)**: Navegação pelas 21 abas em `/egressos/tabelas`, busca/ordenação, CRUD completo, edição de metadados e propagação opcional entre tabelas relacionadas.
4. **Incremento 4 (US4)**: Download e upload direto da pasta `dados/` no GitHub autenticado por token em commit único.
