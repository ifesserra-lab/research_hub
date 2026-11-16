---

description: "Task list for feature 038-github-branch-upload-src"
---

# Tasks: Upload GitHub em Nova Branch, Seleção de Branches e Liberação de Upload para o Projeto SRC

**Input**: Design documents from `/specs/038-github-branch-upload-src/`
**Prerequisites**: `plan.md` (required), `spec.md` (required for user stories), `research.md`, `data-model.md`, `contracts/ipc-commands.md`, `contracts/frontend-routes.md`, `quickstart.md`

**Tests**: Obrigatórios pela Constituição do projeto (Princípio I — TDD Obrigatório e Princípio III — Testes Rigorosos em Rust e Front-end). Os testes de cada fase DEVEM ser escritos antes da respectiva implementação.

**Organization**: As tarefas estão organizadas por User Story (`US1`, `US2`, `US3`) para permitir implementação e validação independentes de cada história.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Pode ser executada em paralelo (arquivos distintos, sem dependências de tarefas incompletas)
- **[Story]**: Rótulo da User Story (`[US1]`, `[US2]`, `[US3]`) obrigatório apenas nas fases de User Stories
- Caminho exato do arquivo incluído em todas as descrições

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Verificação do ambiente e das suítes de base nos 4 programas suportados (`horizon`, `src`, `egressos`, `egressos_dados`) antes de iniciar o ciclo TDD.

- [X] T001 Verificar a compilação e execução da suíte de testes existente em `src-tauri/src/sync_domain.rs` e `src-tauri/tests/sync_github.rs` via `cargo test`
- [X] T002 [P] Verificar a execução da suíte de testes unitários existente do front-end em `frontend/tests/unit/SyncPanel.test.jsx`, `frontend/tests/unit/EgressosDadosDataControl.test.jsx` e `frontend/tests/unit/api.test.js` via `npm test`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Estruturas de dados de domínio (`RemoteBranchInfo`, `UploadBranchOptions`), evolução de `UploadResult` (`branch`, `created_branch`) e regras de validação de nomes de branch Git em `src-tauri/src/sync_domain.rs` e `src-tauri/src/sync_github.rs` que servem de alicerce para todas as User Stories.

**⚠️ CRITICAL**: Nenhuma User Story pode ser iniciada antes da conclusão desta fase.

### Tests for Foundational Phase (TDD — Red First) ⚠️

- [X] T003 Escrever testes unitários em `src-tauri/src/sync_domain.rs` para `validate_branch` (regras: não pode ser vazia, conter espaços em branco, conter `..`, iniciar com `-` ou `/`, nem terminar com `/` ou `.lock`), `generate_branch_name(project)` (prefixo `update/{project}-{timestamp}` válido em `validate_branch`) e `UploadBranchOptions::resolve`

### Implementation for Foundational Phase

- [X] T004 Definir as estruturas `RemoteBranchInfo` (`name: String`, `commit_sha: String`) e `UploadBranchOptions` (`create_new_branch: bool` default `false`, `base_branch: Option<String>`, `new_branch: Option<String>`, `target_branch: Option<String>`), além de `generate_branch_name(project: &str) -> String` e da validação completa em `validate_branch(branch: &str)` (rejeitando vazio, espaços, `..`, início com `-` ou `/`, fim com `/` ou `.lock`) em `src-tauri/src/sync_domain.rs`
- [X] T005 Atualizar a estrutura `UploadResult` em `src-tauri/src/sync_github.rs` para incluir os campos `pub branch: String` (nome da branch onde o commit foi efetivamente publicado) e `pub created_branch: bool` (`true` se uma nova branch `refs/heads/{branch}` foi criada nesta operação; `false` se atualizou uma branch existente)

**Checkpoint**: Estruturas de domínio de branch e `UploadResult` prontas e testadas — implementação das User Stories pode começar.

---

## Phase 3: User Story 1 - Criar uma nova branch automaticamente (ou com nome personalizado) ao fazer upload para o GitHub em qualquer um dos 4 projetos (Priority: P1) 🎯 MVP

**Goal**: Permitir que o curador, ao fazer upload para o GitHub em qualquer um dos 4 projetos (`horizon`, `src`, `egressos`, `egressos_dados`), use o modo padrão **"Criar nova branch com os novos dados"** (ramificando da branch base selecionada, ex.: `main`, com sugestão editável `update/<project>-...` ou gerada automaticamente se em branco) via `POST /repos/{owner}/{repo}/git/refs`, preservando a branch base configurada intacta e exibindo a branch criada e o link do commit.

**Independent Test**: Em qualquer um dos projetos, selecionar o modo "Criar nova branch", informar ou usar o nome sugerido de nova branch derivada de `main`, executar o upload e verificar via testes `wiremock` e `Vitest` que `POST /repos/{owner}/{repo}/git/refs` cria `refs/heads/<nova-branch>` a partir do `HEAD` da branch base sem chamar `PATCH` na branch `main`.

### Tests for User Story 1 (TDD — Write First, Ensure Fail) ⚠️

- [X] T006 [P] [US1] Escrever testes de integração `wiremock` em `src-tauri/tests/sync_github.rs` cobrindo upload em modo `create_new_branch: true` (com nome explícito e com nome vazio gerando `update/<project>-...`), verificando leitura de `GET /repos/{owner}/{repo}/git/ref/heads/{base_branch}`, checagem de conflito caso `refs/heads/{new_branch}` já exista (`AppError::Conflict`), criação de `POST /repos/{owner}/{repo}/git/refs` com `{"ref": "refs/heads/<new_branch>", "sha": "<commit_sha>"}` e preservação de `ProjectSyncConfig.branch`
- [X] T007 [P] [US1] Escrever testes unitários em `frontend/tests/unit/api.test.js` verificando que `POST /github/upload` e `POST /egressos-dados/sync/upload` repassam `createNewBranch`, `baseBranch`, `newBranch` e `targetBranch` aos comandos Tauri `github_upload` e `egressos_dados_sync_upload`
- [X] T008 [P] [US1] Escrever testes de componente em `frontend/tests/unit/SyncPanel.test.jsx` e `frontend/tests/unit/EgressosDadosDataControl.test.jsx` verificando que o modo **"Criar nova branch"** vem selecionado por padrão com sugestão editável `update/<project>-...`, envia `create_new_branch: true` no upload e exibe na confirmação o nome da nova branch criada e o link do commit

### Implementation for User Story 1

- [X] T009 [US1] Implementar suporte a criação de nova branch (`create_branch_ref` via `POST /repos/{owner}/{repo}/git/refs` e checagem prévia de existência da nova branch) em `GitHubClient::upload_file`, `GitHubClient::upload_directory`, `upload_horizon_export`, `upload_egressos_dir` e `upload_egressos_dados_repo` em `src-tauri/src/sync_github.rs`
- [X] T010 [US1] Atualizar os comandos IPC `github_upload` e `egressos_dados_sync_upload` em `src-tauri/src/commands.rs` para receber `create_new_branch: Option<bool>`, `base_branch: Option<String>`, `new_branch: Option<String>` e `target_branch: Option<String>` e repassá-los via `UploadBranchOptions`
- [X] T011 [US1] Atualizar o mapeamento de `POST /github/upload` e `POST /egressos-dados/sync/upload` em `frontend/src/services/api.js` para encaminhar `createNewBranch`, `baseBranch`, `newBranch` e `targetBranch` ao `invoke` do Tauri
- [X] T012 [P] [US1] Atualizar `frontend/src/components/SyncPanel.jsx` para incluir os dois modos explícitos de branch no upload (**"Criar nova branch com os novos dados"** pré-selecionado por padrão com sugestão editável `update/<project>-...` e **"Enviar na branch selecionada"**), preservando a branch base após o upload e exibindo `branch` e `html_url` no alerta de sucesso
- [X] T013 [P] [US1] Atualizar `frontend/src/components/EgressosDadosDataControl.jsx` para incluir os dois modos explícitos de branch no upload (**"Criar nova branch com os novos dados"** pré-selecionado por padrão com sugestão editável `update/egressos_dados-...` e **"Enviar na branch selecionada"**), preservando a branch base após o upload e exibindo `branch` e `html_url` no alerta de sucesso

**Checkpoint**: User Story 1 (MVP) completa e testada — todos os uploads podem criar uma nova branch no GitHub sem tocar na `main`.

---

## Phase 4: User Story 2 - Escolher qualquer branch existente além da `main` para download, base ou upload nos 4 projetos (Priority: P2)

**Goal**: Permitir listar as branches existentes do repositório GitHub (`GET /repos/{owner}/{repo}/branches`) em qualquer um dos 4 projetos (`horizon`, `src`, `egressos`, `egressos_dados`) e selecionar qualquer branch além da `main` tanto como branch base (para ramificar uma nova branch) quanto como branch de destino direta (no modo "Enviar na branch selecionada", inclusive criando a branch no remoto a partir da branch padrão/`main` caso ela ainda não exista, conforme `FR-008`).

**Independent Test**: Clicar em "Carregar branches" em `SyncPanel` ou `EgressosDadosDataControl`, verificar que a lista de branches do repositório remoto é exibida, selecionar uma branch diferente de `main` (ex.: `develop` ou `homologacao`) e realizar o upload (como branch base ou branch de destino selecionada).

### Tests for User Story 2 (TDD — Write First, Ensure Fail) ⚠️

- [X] T014 [P] [US2] Escrever testes `wiremock` em `src-tauri/tests/sync_github.rs` para `GitHubClient::list_branches` (`GET /repos/{owner}/{repo}/branches?per_page=100`), upload ramificando a partir de uma `base_branch` diferente de `main` (ex.: `develop`), upload direto em `target_branch` existente diferente de `main` e fallback de criação de branch quando `target_branch` ainda não existe no remoto (`FR-008`)
- [X] T015 [P] [US2] Escrever testes unitários em `frontend/tests/unit/api.test.js` para `GET /github/branches?project=<p>&repo=<r>` (`github_list_branches`) e `GET /egressos-dados/sync/branches?repo=<r>` (`egressos_dados_sync_list_branches`)
- [X] T016 [P] [US2] Escrever testes de componente em `frontend/tests/unit/SyncPanel.test.jsx` e `frontend/tests/unit/EgressosDadosDataControl.test.jsx` cobrindo o botão **"Carregar branches"**, a seleção de uma branch diferente de `main` na lista carregada, a inclusão automática de uma nova branch recém-criada na lista de branches disponíveis (`FR-005`) e o envio no modo **"Enviar na branch selecionada"**

### Implementation for User Story 2

- [X] T017 [US2] Implementar `GitHubClient::list_branches(&self, owner: &str, repo: &str) -> Result<Vec<RemoteBranchInfo>, AppError>` (`GET {base_url}/repos/{owner}/{repo}/branches?per_page=100`) e o fallback de criação quando `target_branch` ainda não existir no remoto (`FR-008`) em `src-tauri/src/sync_github.rs`
- [X] T018 [US2] Implementar os comandos IPC `github_list_branches` (guard `state.require_session()?`) e `egressos_dados_sync_list_branches` (guard `state.require_egressos_dados_session()?`) em `src-tauri/src/commands.rs` e registrá-los em `tauri::generate_handler!` em `src-tauri/src/lib.rs`
- [X] T019 [US2] Adicionar as rotas `GET /github/branches` -> `github_list_branches` e `GET /egressos-dados/sync/branches` -> `egressos_dados_sync_list_branches` em `frontend/src/services/api.js`
- [X] T020 [P] [US2] Adicionar o botão **"Carregar branches"**, o seletor de branches existentes além da `main` (para destino/branch base e para modo "Enviar na branch selecionada") e a atualização automática da lista de branches após criar nova branch em `frontend/src/components/SyncPanel.jsx`
- [X] T021 [P] [US2] Adicionar o botão **"Carregar branches"**, o seletor de branches existentes além da `main` e a atualização automática da lista de branches após criar nova branch em `frontend/src/components/EgressosDadosDataControl.jsx`

**Checkpoint**: User Stories 1 e 2 completas e testadas — listagem e seleção de qualquer branch além da `main` funcionando nos 4 projetos.

---

## Phase 5: User Story 3 - Liberar o upload do projeto SRC para o GitHub (Priority: P2)

**Goal**: Remover integralmente o bloqueio de política (`sync_policy`) sobre o envio do projeto **SRC** (`src`), exigir extensão `.json` na validação de caminho do `src`, habilitar `allowUpload={true}` na página `/src` (`frontend/src/pages/src/index.astro`) e publicar no GitHub o JSON consolidado completo gerado por `src_domain::export(&conn)` (byte a byte idêntico ao export manual local) utilizando o mesmo fluxo de criação de nova branch ou seleção de branch existente.

**Independent Test**: Em `/src`, configurar repositório (`owner/repo`) e caminho `.json` (ex.: `dados/src_consolidado.json`), executar o upload para o GitHub (em nova branch ou branch selecionada) e verificar que o JSON consolidado do SRC é publicado sem erro de política e com conteúdo idêntico a `src_domain::export`.

### Tests for User Story 3 (TDD — Write First, Ensure Fail) ⚠️

- [X] T022 [P] [US3] Atualizar e adicionar testes unitários em `src-tauri/src/sync_domain.rs` verificando que `check_upload_allowed("src")` retorna `Ok(())` (`FR-001`, `FR-002`) e que `validate_path("src", path)` exige extensão `.json` e rejeita caminhos sem `.json` (`FR-003`)
- [X] T023 [P] [US3] Escrever testes de integração `wiremock` em `src-tauri/tests/sync_github.rs` verificando que o upload do projeto `"src"` publica o JSON consolidado gerado por `src_domain::export(&conn)` (tanto criando uma nova branch quanto atualizando uma branch existente) e produz conteúdo byte a byte idêntico ao export local (`SC-003`)
- [X] T024 [P] [US3] Atualizar os testes em `frontend/tests/unit/SyncPanel.test.jsx` para verificar que `<SyncPanel project="src" allowUpload={true} />` exibe a seção completa de envio para o GitHub com exemplos de caminho `.json` e suporte a criação/seleção de branch

### Implementation for User Story 3

- [X] T025 [US3] Atualizar `check_upload_allowed` em `src-tauri/src/sync_domain.rs` para autorizar `"horizon" | "src" | "egressos" | "egressos_dados" => Ok(())` e atualizar `validate_path` em `src-tauri/src/sync_domain.rs` para exigir que caminhos do projeto `"src"` terminem em `.json` (mensagem: `"O export do SRC é o JSON consolidado: o caminho deve terminar em '.json'."`)
- [X] T026 [US3] Atualizar `upload_horizon_export` em `src-tauri/src/sync_github.rs` (usando mensagem de commit `"Export consolidado SRC ResearchHub"` quando `project == "src"`) e `github_upload` em `src-tauri/src/commands.rs` para que, quando `project == "src"`, gere os bytes via `crate::src_domain::export(&conn)?.into_bytes()` e publique no GitHub
- [X] T027 [US3] Atualizar `frontend/src/pages/src/index.astro` alterando `<SyncPanel client:load project="src" allowUpload={false} />` para `<SyncPanel client:load project="src" allowUpload={true} />` e ajustar os placeholders de caminho do projeto `"src"` em `frontend/src/components/SyncPanel.jsx` (ex.: `dados/src_consolidado.json`)

**Checkpoint**: Todas as 3 User Stories (`US1`, `US2`, `US3`) estão funcionais e testadas de forma independente nos 4 programas (`horizon`, `src`, `egressos`, `egressos_dados`).

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Validação cruzada de segurança de token (`SC-004`), build estático Astro e execução completa do `quickstart.md`.

- [X] T028 [P] Verificar em `src-tauri/tests/sync_github.rs` que falhas HTTP em `list_branches` e na criação de nova branch (`POST /git/refs`) jamais vazam o valor do token de acesso pessoal em mensagens de erro (`SC-004`)
- [X] T029 Executar a validação completa de `specs/038-github-branch-upload-src/quickstart.md`: `cd frontend && npm test`, `cd frontend && npm run build` (`astro check && astro build`) e `cd src-tauri && cargo test`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Sem dependências — início imediato.
- **Foundational (Phase 2)**: Depende da Phase 1 — **BLOQUEIA** todas as User Stories.
- **User Story 1 (Phase 3 — P1 🎯 MVP)**: Depende da conclusão da Phase 2.
- **User Story 2 (Phase 4 — P2)**: Depende da Phase 2 e integra-se aos seletores de branch adicionados na Phase 3 (`US1`).
- **User Story 3 (Phase 5 — P2)**: Depende da Phase 2 (pode rodar em paralelo com `US1`/`US2` na camada de domínio Rust e finalizar junto com o `SyncPanel.jsx`).
- **Polish (Phase 6)**: Depende da conclusão de todas as User Stories (`US1`, `US2`, `US3`).

### User Story Dependencies

- **User Story 1 (P1)**: Independente após a fase Foundational. Entrega o modo padrão "Criar nova branch" nos 4 projetos.
- **User Story 2 (P2)**: Constrói sobre as estruturas de branch da Fase 2 e complementa a UI da `US1` com listagem remota (`GET /branches`) e seleção de qualquer branch além da `main`.
- **User Story 3 (P2)**: Independente na camada de domínio/backend (`check_upload_allowed("src")`, `validate_path("src", ...)`, `src_domain::export`) e habilita `allowUpload={true}` em `/src` reutilizando imediatamente as melhorias de `US1` e `US2` no `SyncPanel.jsx`.

### Parallel Opportunities

- **Phase 1**: `T001` e `T002` rodam em paralelo.
- **Phase 3 (`US1`)**: Testes `T006`, `T007` e `T008` podem ser escritos em paralelo; atualizações de UI `T012` (`SyncPanel.jsx`) e `T013` (`EgressosDadosDataControl.jsx`) rodam em paralelo.
- **Phase 4 (`US2`)**: Testes `T014`, `T015` e `T016` podem ser escritos em paralelo; atualizações de UI `T020` (`SyncPanel.jsx`) e `T021` (`EgressosDadosDataControl.jsx`) rodam em paralelo.
- **Phase 5 (`US3`)**: Testes `T022`, `T023` e `T024` podem ser escritos em paralelo.

---

## Parallel Example: User Story 1

```bash
# Escrever testes de US1 em paralelo (Rust wiremock + Vitest API + Vitest UI):
Task: "T006 [P] [US1] Escrever testes de integração wiremock em src-tauri/tests/sync_github.rs"
Task: "T007 [P] [US1] Escrever testes unitários em frontend/tests/unit/api.test.js"
Task: "T008 [P] [US1] Escrever testes de componente em frontend/tests/unit/SyncPanel.test.jsx e frontend/tests/unit/EgressosDadosDataControl.test.jsx"

# Atualizar os dois painéis de sincronização em paralelo após T009-T011:
Task: "T012 [P] [US1] Atualizar frontend/src/components/SyncPanel.jsx"
Task: "T013 [P] [US1] Atualizar frontend/src/components/EgressosDadosDataControl.jsx"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Concluir Phase 1: Setup (`T001`–`T002`)
2. Concluir Phase 2: Foundational (`T003`–`T005`)
3. Concluir Phase 3: User Story 1 (`T006`–`T013`)
4. **STOP & VALIDATE**: Validar que o envio em modo "Criar nova branch" cria `refs/heads/<nova-branch>` sem modificar a `main`.

### Incremental Delivery

1. Setup + Foundational → Estruturas `RemoteBranchInfo`, `UploadBranchOptions` e `UploadResult` prontas.
2. Adicionar User Story 1 (`P1`) → Criação de nova branch no upload funcionando nos painéis de sincronização.
3. Adicionar User Story 2 (`P2`) → Listagem e seleção de branches existentes além da `main` via GitHub API.
4. Adicionar User Story 3 (`P2`) → Desbloqueio completo do upload do projeto SRC (`src`) em `/src`.
5. Executar Phase 6 (`T028`–`T029`) → Garantir 100% dos testes Rust, Vitest e build Astro passando.
