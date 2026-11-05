---

description: "Task list for feature 036-egressos-github-org-auth implementation"
---

# Tasks: Controle de Acesso Exclusivo à Aba `egressos-dados` por Organização do GitHub (`ifesserra-lab`)

**Input**: Design documents from `/specs/036-egressos-github-org-auth/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/ (`ipc-commands.md`, `frontend-routes.md`), quickstart.md

**Tests**: OBRIGATÓRIOS — A Constituição (Princípio I: TDD Obrigatório; Princípio III: Testes Rigorosos) exige escrever o teste primeiro (vermelho), implementar até passar (verde) e refatorar.

**Organization**: Tasks agrupadas por user story (`US1` autenticação por token como membro ativo de `ifesserra-lab` e acesso ao painel estrutural de `egressos-dados`, `US2` bloqueio de tokens inválidos/sem escopo/fora da organização `ifesserra-lab` e guarda IPC, `US3` encerramento/troca do login especial na aba e limpeza automática no logout geral) para permitir implementação e validação independentes.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- Núcleo Rust: `src-tauri/src/` (testes unitários inline e testes de integração HTTP em `src-tauri/tests/sync_github.rs`)
- Front-end: `frontend/src/` (testes em `frontend/tests/unit/`)
- Contratos de referência: `specs/036-egressos-github-org-auth/contracts/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirmar a baseline de testes na branch `036-egressos-github-org-auth` antes de iniciar o ciclo TDD

- [X] T001 Executar `cargo test` em `src-tauri/` e `npm test` em `frontend/` para confirmar que a suíte existente passa 100% antes das alterações

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Erro tipado `OrgUnauthorized` (`error.rs`) e estruturas/guardas de sessão em memória (`EgressosDadosAuth` e `EgressosDadosAuthStatus` em `state.rs`) que bloqueiam todas as user stories

**⚠️ CRITICAL**: Nenhuma user story pode começar antes da conclusão desta fase

- [X] T002 [P] TDD (red → green): escrever teste unitário em `src-tauri/src/error.rs` verificando que `AppError::OrgUnauthorized("acesso restrito à organização ifesserra-lab".into())` serializa com `kind == "org_unauthorized"` e mensagem legível, e implementar a variante `OrgUnauthorized(String)` em `src-tauri/src/error.rs`
- [X] T003 TDD (red): escrever testes unitários em `src-tauri/src/state.rs` verificando o contrato de `data-model.md` §1 e §3: (a) `current_egressos_dados_status` retorna `{ authorized: false, login: None, organization: "ifesserra-lab", token_hint: None }` por padrão; (b) `require_egressos_dados_session` falha com `AppError::Unauthenticated` se a sessão principal estiver fechada e com `AppError::OrgUnauthorized` se `egressos_dados_auth` for `None`; (c) `open_egressos_dados_session("membro", "ifesserra-lab", "ghp_123456789f2a")` armazena a sessão apenas em memória, expõe `token_hint == Some("9f2a")` sem jamais serializar o token completo em `EgressosDadosAuthStatus`, e faz `require_egressos_dados_session` retornar a sessão ativa; (d) `close_egressos_dados_session` e `close_session` (logout geral) limpam imediatamente `egressos_dados_auth` da memória
- [X] T004 Implementar `EgressosDadosAuth` (`login: String`, `organization: String`, `token: String`, `token_hint: String`), `EgressosDadosAuthStatus` (`authorized: bool`, `login: Option<String>`, `organization: String`, `token_hint: Option<String>`), o campo `egressos_dados_auth: Mutex<Option<EgressosDadosAuth>>` em `AppState` e os métodos `open_egressos_dados_session`, `close_egressos_dados_session`, `current_egressos_dados_status` e `require_egressos_dados_session` (mais limpeza em cascata dentro de `close_session`) em `src-tauri/src/state.rs` — verde para T003

**Checkpoint**: Erro tipado `org_unauthorized` e estado em memória de `egressos-dados` prontos e testados — implementação das user stories liberada.

---

## Phase 3: User Story 1 - Autenticar-se com Token do GitHub como membro da organização `ifesserra-lab` para acessar a aba `egressos-dados` (Priority: P1) 🎯 MVP

**Goal**: Disponibilizar `egressos-dados` como 4º projeto na tela `/projects`, apresentar a tela de bloqueio por Token do GitHub ao entrar sem autorização, validar a participação ativa (`state == "active"` ou membro público `204`) na organização `ifesserra-lab` e desbloquear o painel estrutural de `egressos-dados` exibindo `@login` e o selo da organização `ifesserra-lab`.

**Independent Test**: Clicar no 4º cartão `Egressos — Dados` em `/projects`, ver a tela de login especial por Token do GitHub, informar um token válido de membro ativo da organização `ifesserra-lab` e confirmar que o painel estrutural de `/egressos-dados` é desbloqueado com `@login` + badge `ifesserra-lab` e permanece desbloqueado ao alternar entre projetos na mesma sessão.

### Tests for User Story 1 ⚠️ (TDD — escrever primeiro, ver falhar)

- [X] T005 [P] [US1] Escrever testes de integração `wiremock` (red) em `src-tauri/tests/sync_github.rs` para `GitHubClient::verify_org_membership("ifesserra-lab")`: (a) `GET /user` (`200 OK`, `login: "membro-lab"`) + `GET /user/memberships/orgs/ifesserra-lab` (`200 OK`, `state: "active"`) autoriza o acesso retornando `OrgMembershipInfo { login: "membro-lab", organization: "ifesserra-lab", state: "active" }`; (b) fallback público quando `GET /user/memberships/orgs/ifesserra-lab` retorna `403` (sem `read:org`) mas `GET /orgs/ifesserra-lab/members/membro-lab` retorna `204 No Content` autoriza o acesso (`FR-003`, Clarificação Q3)
- [X] T006 [P] [US1] Escrever testes unitários (red) em `frontend/tests/unit/api.test.js` para as rotas `GET /egressos-dados/auth/status`, `POST /egressos-dados/auth/login`, `DELETE /egressos-dados/auth/logout` e `GET /egressos-dados/workspace` em `frontend/src/services/api.js` (`contracts/frontend-routes.md` §1)
- [X] T007 [P] [US1] Atualizar testes (red) em `frontend/tests/unit/ProjectSelector.test.jsx` verificando a renderização dos 4 projetos (`Horizon`, `SRC`, `Egressos` e `Egressos — Dados` com `id: 'egressos-dados'` e `href: '/egressos-dados'`)
- [X] T008 [P] [US1] Criar testes unitários (red) em `frontend/tests/unit/EgressosDadosWorkspace.test.jsx` verificando para US1: (a) quando `GET /egressos-dados/auth/status` retorna `authorized: false`, renderiza o formulário de login especial via Token do GitHub (`type="password"`) exclusivo da organização `ifesserra-lab` e oculta o painel de tabelas; (b) ao submeter um token válido (`POST /egressos-dados/auth/login`), desbloqueia a aba e exibe `@login`, badge `ifesserra-lab`, terminação do token (`…9f2a`) e o painel estrutural carregado via `GET /egressos-dados/workspace`; (c) ao montar com `authorized: true` (retorno à aba na mesma sessão), abre diretamente o painel desbloqueado sem pedir o token novamente (`US1-3`)
- [X] T009 [P] [US1] Atualizar o teste de rotas (`ROUTES`) em `src-tauri/src/lib.rs` incluindo `"/egressos-dados"` e ajustando `contagem_de_rotas_bate_com_o_menu_lateral` de 23 para 24

### Implementation for User Story 1

- [X] T010 [US1] Implementar `REQUIRED_EGRESSOS_DADOS_ORG = "ifesserra-lab"`, `OrgMembershipInfo` e `GitHubClient::verify_org_membership(&self, org: &str)` em `src-tauri/src/sync_github.rs` consultando `GET /user` e `GET /user/memberships/orgs/{org}` com fallback para `GET /orgs/{org}/members/{login}` — verde para T005
- [X] T011 [US1] Implementar os comandos IPC `egressos_dados_login`, `egressos_dados_status`, `egressos_dados_logout` e `egressos_dados_get_workspace` em `src-tauri/src/commands.rs` e registrá-los em `tauri::generate_handler!` em `src-tauri/src/lib.rs` (`contracts/ipc-commands.md` §1)
- [X] T012 [P] [US1] Adicionar as rotas `GET /egressos-dados/auth/status`, `POST /egressos-dados/auth/login`, `DELETE /egressos-dados/auth/logout` e `GET /egressos-dados/workspace` em `frontend/src/services/api.js` antes dos padrões genéricos `/:entity` — verde para T006
- [X] T013 [P] [US1] Atualizar `frontend/src/components/ProjectSelector.jsx` e `frontend/src/pages/projects.astro` adicionando o 4º cartão `{ id: 'egressos-dados', title: 'Egressos — Dados', description: 'Área restrita à organização GitHub ifesserra-lab (acesso exclusivo via GitHub Token)', href: '/egressos-dados' }` — verde para T007
- [X] T014 [US1] Criar o componente `frontend/src/components/EgressosDadosWorkspace.jsx` com tela de bloqueio por Token do GitHub (`type="password"`, aviso da organização `ifesserra-lab` e escopo `read:org`) e painel estrutural desbloqueado (`@login`, badge `ifesserra-lab`, `token_hint` e área pronta para conexão dos JSONs e tabelas na próxima etapa) — verde para T008
- [X] T015 [US1] Criar o layout `frontend/src/layouts/EgressosDadosDashboard.astro` (menu lateral próprio de `egressos-dados`, guard de sessão local, link `↩ Trocar projeto` para `/projects` e botão `Sair`) e a página `frontend/src/pages/egressos-dados/index.astro` montando `EgressosDadosWorkspace.jsx`
- [X] T016 [US1] Compilar o front-end (`npm run build` em `frontend/`) e confirmar que o teste de resolução das 24 rotas em `src-tauri/src/lib.rs` passa — verde para T009

**Checkpoint**: User Story 1 funcional e testável isoladamente — membros ativos de `ifesserra-lab` autenticam com seu Token do GitHub e desbloqueiam a aba `/egressos-dados`.

---

## Phase 4: User Story 2 - Bloquear acesso a tokens inválidos, sem permissão ou de usuários fora da organização `ifesserra-lab` (Priority: P2)

**Goal**: Recusar com mensagens distintas e claras qualquer tentativa de login especial na aba `egressos-dados` com token inválido/expirado (`401`), usuário válido no GitHub mas fora da organização `ifesserra-lab` (`404`), convite pendente (`state: "pending"`), token sem permissão `read:org` para verificar membresia privada (`403`) ou falha de rede, mantendo a aba e o comando `egressos_dados_get_workspace` estritamente bloqueados sem jamais vazar o valor do token.

**Independent Test**: Tentar autenticar na aba `egressos-dados` com token inválido, token de conta fora da `ifesserra-lab`, token de membro com convite pendente e token privado sem `read:org`; confirmar que em todos os casos a aba continua bloqueada, o motivo exato é exibido e o valor do token não aparece em nenhuma mensagem.

### Tests for User Story 2 ⚠️ (TDD — escrever primeiro, ver falhar)

- [X] T017 [P] [US2] Escrever testes de rejeição e sigilo `wiremock` (red) em `src-tauri/tests/sync_github.rs`: (a) usuário válido no GitHub mas não-membro (`404` em `/user/memberships/orgs/ifesserra-lab` e `404` em `/orgs/ifesserra-lab/members/{login}`) retorna `AppError::OrgUnauthorized` citando `@login` e `ifesserra-lab` (`US2-1`); (b) membro com convite pendente (`200 OK` com `state: "pending"`) retorna `AppError::OrgUnauthorized` informando convite pendente (`Edge Cases`, `FR-004`); (c) token sem escopo `read:org` e não listado publicamente (`403` em `/user/memberships/orgs/ifesserra-lab` + `404` em `/orgs/ifesserra-lab/members/{login}`) retorna `AppError::OrgUnauthorized` orientando habilitar `read:org` (`US2-3`); (d) token inválido/revogado (`401` em `/user`) retorna erro claro de token inválido sem jamais conter a string do token (`US2-2`, `SC-003`)
- [X] T018 [P] [US2] Adicionar testes unitários (red) em `frontend/tests/unit/EgressosDadosWorkspace.test.jsx` verificando para US2: (a) submissão de token vazio exibe erro de validação sem chamar o backend; (b) falha com `kind: "org_unauthorized"` (usuário fora de `ifesserra-lab`, convite pendente ou sem `read:org`), `kind: "github_api"` (token inválido) ou `kind: "network"` exibe a mensagem acionável em alerta de erro e mantém o conteúdo interno oculto (`FR-004`, `FR-005`)

### Implementation for User Story 2

- [X] T019 [US2] Refinar o tratamento de todos os códigos de status e mensagens de recusa em `GitHubClient::verify_org_membership` (`src-tauri/src/sync_github.rs`) e garantir em `egressos_dados_login` (`src-tauri/src/commands.rs`) que qualquer tentativa falha limpa eventual `egressos_dados_auth` anterior em `AppState` — verde para T017
- [X] T020 [US2] Refinar a validação de entrada e a exibição de erros tipados (`org_unauthorized`, `github_api`, `network`, `validation`) no formulário de login especial em `frontend/src/components/EgressosDadosWorkspace.jsx` garantindo que o campo de token nunca ecoe o segredo em mensagens na tela — verde para T018

**Checkpoint**: User Stories 1 e 2 completas — tanto o caminho feliz de membros de `ifesserra-lab` quanto todos os bloqueios de segurança funcionam e estão cobertos por testes automatizados.

---

## Phase 5: User Story 3 - Encerrar ou trocar a credencial do login especial na aba `egressos-dados` (Priority: P3)

**Goal**: Permitir que o membro autenticado encerre o login especial ("Bloquear aba / Trocar token") a qualquer momento dentro de `/egressos-dados`, retornando imediatamente à tela de bloqueio, e garantir que o logout geral do aplicativo também revogue a autorização de `egressos-dados`.

**Independent Test**: Com a aba `egressos-dados` desbloqueada, clicar em "Bloquear aba / Trocar token" e verificar que a aba retorna imediatamente à tela de login por token e `GET /egressos-dados/auth/status` volta a `authorized: false`.

### Tests for User Story 3 ⚠️ (TDD — escrever primeiro, ver falhar)

- [X] T021 [P] [US3] Adicionar testes unitários (red) em `frontend/tests/unit/EgressosDadosWorkspace.test.jsx` verificando para US3: ao clicar no botão "Bloquear aba / Trocar token" no painel desbloqueado, `DELETE /egressos-dados/auth/logout` é chamado, os dados do painel interno desaparecem imediatamente e a tela volta a exibir o formulário de login especial por Token do GitHub (`US3-1`, `FR-008`)
- [X] T022 [P] [US3] Adicionar teste unitário em `src-tauri/src/state.rs` confirmando que reautenticar com falha após uma sessão autorizada ou chamar `close_session()` (logout geral do app) revoga imediatamente `egressos_dados_auth` (`US3-2`)

### Implementation for User Story 3

- [X] T023 [US3] Implementar o fluxo completo do botão "Bloquear aba / Trocar token" em `frontend/src/components/EgressosDadosWorkspace.jsx` limpando o estado local do componente, o campo de input e invocando `DELETE /egressos-dados/auth/logout` — verde para T021 e T022

**Checkpoint**: Todas as 3 User Stories estão completas e funcionam de forma independente.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Verificação de não-regressão nos demais projetos (`Horizon`, `SRC` e `Egressos`), auditoria de sigilo do token (`SC-003`) e validação ponta a ponta do `quickstart.md`

- [X] T024 [P] Verificar em `src-tauri/tests/sync_github.rs` e `src-tauri/src/state.rs` que o bloqueio de `egressos-dados` (`require_egressos_dados_session`) não afeta nenhuma operação de `Horizon`, `SRC` ou `Egressos` (`FR-010`, `SC-004`) e que `sync_config.json` no disco não é criado nem modificado pelo login especial de `egressos-dados` (`FR-008`, `FR-009`)
- [X] T025 Executar a bateria completa de verificação: `npm test` e `npm run build` em `frontend/` + `cargo test` em `src-tauri/`, confirmando 100% dos testes passando e 24 rotas resolvidas no asset resolver do Tauri (`quickstart.md` §2)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1: T001)**: Sem dependências — início imediato.
- **Foundational (Phase 2: T002–T004)**: Depende do Setup — **BLOQUEIA** todas as user stories.
- **User Story 1 (Phase 3: T005–T016)**: Depende da Foundational (Phase 2) — entrega o MVP completo do login especial na organização `ifesserra-lab` e a nova aba `/egressos-dados`.
- **User Story 2 (Phase 4: T017–T020)**: Depende da Foundational (Phase 2) e complementa a verificação e interface criadas na US1 com todos os cenários negativos de bloqueio.
- **User Story 3 (Phase 5: T021–T023)**: Depende da US1 (painel desbloqueado) para acionar o encerramento/troca de token.
- **Polish (Phase 6: T024–T025)**: Depende da conclusão das User Stories 1 a 3.

### User Story Dependencies

```text
Phase 1 (Setup: T001)
  └── Phase 2 (Foundational: OrgUnauthorized + EgressosDadosAuth em memória: T002–T004)
        ├── Phase 3 (US1: Aba /egressos-dados + Login Especial ifesserra-lab: T005–T016) 🎯 MVP
        ├── Phase 4 (US2: Bloqueio de Não-Membros, Convites Pendentes e Tokens Inválidos: T017–T020)
        └── Phase 5 (US3: Encerrar / Trocar Token e Limpeza no Logout: T021–T023)
              └── Phase 6 (Polish & Validação Final: T024–T025)
```

### Parallel Opportunities

- **Phase 2**: `T002` (`error.rs`) e `T003` (`state.rs`) tocam arquivos distintos e podem rodar em paralelo.
- **Phase 3 (US1)**: As tarefas de teste `T005` (`tests/sync_github.rs`), `T006` (`api.test.js`), `T007` (`ProjectSelector.test.jsx`), `T008` (`EgressosDadosWorkspace.test.jsx`) e `T009` (`lib.rs`) tocam arquivos distintos e podem ser escritas em paralelo; na implementação, `T012` (`api.js`) e `T013` (`ProjectSelector.jsx`) podem rodar em paralelo com `T010` (`sync_github.rs`).
- **Phase 4 (US2)**: `T017` (testes Rust `wiremock`) e `T018` (testes Vitest `EgressosDadosWorkspace.test.jsx`) podem rodar em paralelo.
- **Phase 5 (US3)**: `T021` (testes Vitest) e `T022` (teste Rust em `state.rs`) podem rodar em paralelo.

---

## Parallel Example: User Story 1

```bash
# Escrever todos os testes da User Story 1 em paralelo (arquivos independentes):
Task T005: "Escrever testes de integração wiremock (red) em src-tauri/tests/sync_github.rs"
Task T006: "Escrever testes unitários (red) em frontend/tests/unit/api.test.js"
Task T007: "Atualizar testes (red) em frontend/tests/unit/ProjectSelector.test.jsx"
Task T008: "Criar testes unitários (red) em frontend/tests/unit/EgressosDadosWorkspace.test.jsx"
Task T009: "Atualizar o teste de rotas (ROUTES) em src-tauri/src/lib.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Concluir Phase 1 (Setup) e Phase 2 (Foundational: `OrgUnauthorized` + `EgressosDadosAuth` em `state.rs`).
2. Concluir Phase 3 (US1): 4º cartão em `/projects`, rota `/egressos-dados`, verificação de membro ativo em `ifesserra-lab` e painel estrutural desbloqueado.
3. **STOP and VALIDATE**: Confirmar que a aba `/egressos-dados` abre bloqueada e só libera o painel estrutural após validar um token de membro ativo de `ifesserra-lab`.

### Incremental Delivery

1. **Incremento 1 (US1)**: Navegação para `/egressos-dados` e desbloqueio por membro ativo da organização `ifesserra-lab`.
2. **Incremento 2 (US2)**: Cobertura completa de bloqueio e mensagens específicas para não-membros, convites pendentes, falta de escopo `read:org` e tokens inválidos.
3. **Incremento 3 (US3)**: Botão de bloqueio imediato / troca de token na aba e revogação automática no logout geral.
