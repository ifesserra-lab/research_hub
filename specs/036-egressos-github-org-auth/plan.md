# Implementation Plan: Controle de Acesso Exclusivo à Aba `egressos-dados` por Organização do GitHub (`ifesserra-lab`)

**Branch**: `036-egressos-github-org-auth` | **Date**: 2026-11-05 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/036-egressos-github-org-auth/spec.md`

## Summary

Implementar a nova aba/área independente **`egressos-dados`** no Research Hub (como 4º projeto na página `/projects`, ao lado de **Horizon**, **SRC** e **Egressos**) protegida por um **sistema de login especial via Token do GitHub** restrito exclusivamente a membros ativos da organização **`ifesserra-lab`** (`https://github.com/orgs/ifesserra-lab/dashboard`). Toda a validação ocorre no processo Rust (`sync_github.rs`, `state.rs` e `commands.rs`): o cliente HTTP consulta `GET /user` e `GET /user/memberships/orgs/ifesserra-lab` (com fallback para `GET /orgs/ifesserra-lab/members/{login}` caso o token não tenha `read:org` e a filiação seja pública), aprovando apenas membros com estado `active` (`FR-003`..`FR-005`). A sessão especial (`EgressosDadosAuth`) e o token permanecem exclusivamente em memória no `AppState` durante a sessão ativa (nunca gravados em disco, e limpos automaticamente no logout da aba ou no logout geral do aplicativo — `FR-008`, `FR-009`), bloqueando chamadas diretas aos comandos de dados de `egressos-dados` via `state.require_egressos_dados_session()?` (`FR-006`) e exibindo o painel estrutural desbloqueado com `@login` + badge `ifesserra-lab` pronto para a leitura dos JSONs e tabelas na próxima etapa (`FR-007`).

## Technical Context

**Language/Version**: Rust (edition 2021, Tauri 2.x) no núcleo + JavaScript/React sobre Astro (build estático embarcado) no front-end.

**Primary Dependencies**: Existentes no `Cargo.toml` — `reqwest` (rustls), `serde` / `serde_json 1.0`, `thiserror` e `wiremock` (em `dev-dependencies` para testes HTTP determinísticos sem rede real). Nenhuma dependência externa nova é necessária.

**Storage**: Exclusivamente em memória (`Mutex<Option<EgressosDadosAuth>>` dentro de `AppState` em `src-tauri/src/state.rs`), conforme `FR-008` e `FR-009` (Clarificação Q2). Nenhuma alteração no schema SQLite nem gravação do token especial em `sync_config.json`.

**Testing**: `cargo test` (testes unitários de estado e guarda de sessão em `state.rs`, testes de erro tipado `OrgUnauthorized` em `error.rs`, testes de integração HTTP com `wiremock` em `tests/sync_github.rs` cobrindo membro ativo, fallback público, convite pendente, não-membro, falta de `read:org` e token inválido, mais teste de resolução de rotas `ROUTES` em `lib.rs`) e `npm test` (Vitest para `api.js`, `ProjectSelector.jsx` e `EgressosDadosWorkspace.jsx`).

**Target Platform**: Desktop Linux e Windows (Tauri 2.0).

**Project Type**: Aplicação desktop autocontida de processo único (`src-tauri/` + `frontend/`).

**Performance Goals**: Verificação de membresia na organização `ifesserra-lab` e desbloqueio da aba em < 5 segundos em conexão padrão (`SC-001`); alternância entre projetos já autenticados na sessão em < 1 segundo.

**Constraints**: Regra de negócio 100% em Rust (`Princípio VI`); acoplamento JS→Rust restrito a `frontend/src/services/api.js`; token do login especial mantido somente em memória e jamais exposto em mensagens de erro, logs ou retornos completos (`FR-008`, `FR-009`, `SC-003`); zero impacto ou regressão nos projetos existentes **Horizon**, **SRC** e **Egressos** (`FR-010`, `SC-004`).

**Scale/Scope**: 1 variante nova em `AppError` (`OrgUnauthorized`), 1 struct de estado em memória (`EgressosDadosAuth` + `EgressosDadosAuthStatus` em `state.rs`), 1 método novo em `GitHubClient` (`verify_org_membership` em `sync_github.rs`), 4 comandos IPC novos (`egressos_dados_login`, `egressos_dados_status`, `egressos_dados_logout`, `egressos_dados_get_workspace` em `commands.rs`), 4 rotas em `api.js`, 1 layout Astro novo (`EgressosDadosDashboard.astro`), 1 página Astro nova (`/egressos-dados/index.astro`), 1 componente React novo (`EgressosDadosWorkspace.jsx`) e atualização de `ProjectSelector.jsx` / `projects.astro` para 4 projetos (`ROUTES`: 23 → 24).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Princípio | Status (pré-Phase 0) | Evidência / Justificativa | Re-check pós-Phase 1 |
|-----------|----------------------|---------------------------|----------------------|
| I. TDD Obrigatório | ✅ PASS | Cada unidade (`OrgUnauthorized`, guarda em `state.rs`, `verify_org_membership` contra `wiremock`, comandos IPC, rotas em `api.js`, `ProjectSelector` e `EgressosDadosWorkspace`) terá testes automatizados escritos antes da implementação. | ✅ Mantido — ver `quickstart.md` §2. |
| II. Paridade Funcional Antes de Melhoria | ✅ PASS | Os projetos existentes (**Horizon**, **SRC** e **Egressos**) permanecem 100% intocados e acessíveis com o login padrão (`FR-010`, `SC-004`). | ✅ Mantido. |
| III. Testes Rigorosos (Rust e Front-end) | ✅ PASS | Toda a verificação contra a API do GitHub é testada sem rede real via `wiremock` em `cargo test`; estado de sessão testado em `state.rs`; componentes e ponte IPC testados em `Vitest` (`npm test`); resolução da rota `/egressos-dados` verificada no asset resolver em `lib.rs`. | ✅ Mantido. |
| IV. CRUD Completo e Operações Diretas | ✅ PASS | Esta feature constrói especificamente o pré-requisito de autenticação por organização (`ifesserra-lab`) na aba `egressos-dados` antes da leitura dos JSONs e criação das tabelas na próxima etapa, conforme solicitado pelo usuário. | ✅ Mantido. |
| V. Aplicação Desktop Local e Autocontida | ✅ PASS | Processo único Tauri 2.0; nenhuma dependência de servidor intermediário; o uso offline de Horizon, SRC e Egressos local permanece intacto (`SC-004`). | ✅ Mantido. |
| VI. Lógica de Negócio em Rust | ✅ PASS | A validação do token, a checagem de filiação ativa em `ifesserra-lab` e o bloqueio de comandos (`require_egressos_dados_session`) vivem inteiramente em Rust (`sync_github.rs`, `state.rs`, `commands.rs`), e todo tráfego JS→Rust passa por `frontend/src/services/api.js`. | ✅ Mantido. |

Nenhuma violação da Constituição — tabela de Complexity Tracking vazia.

## Project Structure

### Documentation (this feature)

```text
specs/036-egressos-github-org-auth/
├── plan.md                         # Este arquivo (/speckit-plan)
├── research.md                     # Phase 0 output
├── data-model.md                   # Phase 1 output
├── quickstart.md                   # Phase 1 output
├── contracts/                      # Phase 1 output
│   ├── ipc-commands.md             # Contrato dos comandos IPC egressos_dados_*
│   └── frontend-routes.md          # Contrato das rotas em api.js e páginas Astro
└── tasks.md                        # Phase 2 output (/speckit-tasks — gerado na próxima etapa)
```

### Source Code (repository root)

```text
src-tauri/
├── src/
│   ├── error.rs                    # MODIFICAR: adicionar variante AppError::OrgUnauthorized(String) (kind: "org_unauthorized")
│   ├── state.rs                    # MODIFICAR: adicionar EgressosDadosAuth, EgressosDadosAuthStatus, slot egressos_dados_auth em AppState e limpeza em close_session()
│   ├── sync_github.rs              # MODIFICAR: adicionar REQUIRED_EGRESSOS_DADOS_ORG ("ifesserra-lab"), OrgMembershipInfo e GitHubClient::verify_org_membership()
│   ├── commands.rs                 # MODIFICAR: adicionar egressos_dados_login, egressos_dados_status, egressos_dados_logout e egressos_dados_get_workspace
│   └── lib.rs                      # MODIFICAR: registrar comandos em invoke_handler e incluir "/egressos-dados" em ROUTES (23 → 24)
└── tests/
    └── sync_github.rs              # MODIFICAR: testes wiremock para verify_org_membership (active, pending, fallback público, 404 não-membro, 403 sem read:org, 401 inválido)

frontend/
├── src/
│   ├── pages/
│   │   ├── projects.astro          # MODIFICAR: incluir 4º cartão Egressos — Dados (/egressos-dados)
│   │   └── egressos-dados/
│   │       └── index.astro         # NOVO: página da aba egressos-dados com EgressosDadosDashboard e EgressosDadosWorkspace
│   ├── layouts/
│   │   └── EgressosDadosDashboard.astro # NOVO: layout exclusivo da área egressos-dados com menu lateral e indicador de acesso restrito ifesserra-lab
│   ├── components/
│   │   ├── ProjectSelector.jsx     # MODIFICAR: incluir 4º projeto egressos-dados em DEFAULT_PROJECTS e ajustar grid
│   │   └── EgressosDadosWorkspace.jsx # NOVO: gate de login especial por GitHub Token (ifesserra-lab) + painel estrutural desbloqueado + botão de bloquear aba
│   └── services/
│       └── api.js                  # MODIFICAR: adicionar rotas /egressos-dados/auth/* e /egressos-dados/workspace
└── tests/
    └── unit/                       # MODIFICAR/NOVO: testes Vitest para api.js, ProjectSelector e EgressosDadosWorkspace
```

**Structure Decision**: Reutiliza a estrutura consolidada do Research Hub (`src-tauri/src/` + `frontend/src/`), concentrando a verificação HTTP da organização `ifesserra-lab` em `sync_github.rs`, o estado de sessão em memória em `state.rs` e o acoplamento IPC em `frontend/src/services/api.js`.

## Complexity Tracking

> Nenhuma violação da Constituição — tabela vazia.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| — | — | — |
