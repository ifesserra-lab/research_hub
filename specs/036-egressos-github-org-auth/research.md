# Research: Controle de Acesso Exclusivo à Aba `egressos-dados` por Organização do GitHub (`ifesserra-lab`)

**Date**: 2026-11-05 | **Feature**: [spec.md](./spec.md)

## R1 — Protocolo de Verificação de Membresia Ativa na Organização `ifesserra-lab` no GitHub

- **Decision**: Implementar em `src-tauri/src/sync_github.rs` o método `GitHubClient::verify_org_membership(&self, org: &str) -> Result<OrgMembershipInfo, AppError>`, com constante `REQUIRED_GITHUB_ORG = "ifesserra-lab"`, seguindo uma verificação de 2 passos autenticados via `Bearer <token>`:
  1. **Identificação do usuário (`GET {base_url}/user`)**:
     - `401 Unauthorized` → retorna `AppError::GithubOrgAccessDenied` / `GithubApi { status: 401, hint: "Token do GitHub inválido ou expirado. Informe um token de acesso pessoal válido." }`.
     - `200 OK` → extrai `login` (nome de usuário no GitHub) e o cabeçalho `x-oauth-scopes`.
  2. **Verificação autenticada de filiação na organização (`GET {base_url}/user/memberships/orgs/{org}`)**:
     - `200 OK` → lê o campo `state` do JSON retornado:
       - Se `state == "active"`, o usuário é membro ativo (público ou privado) da organização `ifesserra-lab` → **Acesso Autorizado** (`OrgMembershipInfo { login, organization: "ifesserra-lab", role, state: "active" }`).
       - Se `state == "pending"`, o usuário tem apenas convite pendente → **Acesso Negado** com mensagem explícita: *"Sua participação na organização 'ifesserra-lab' está com convite pendente no GitHub. Aceite o convite em github.com/orgs/ifesserra-lab antes de acessar egressos-dados."*
     - `403 Forbidden` (token sem permissão `read:org` para consultar filiação privada) → executa **fallback público** (`GET {base_url}/orgs/{org}/members/{login}`):
       - `204 No Content` → o usuário é membro público ativo de `ifesserra-lab` → **Acesso Autorizado**.
       - `404 Not Found` / `302` / `403` → **Acesso Negado** com mensagem orientando o escopo necessário: *"O token informado não possui permissão para verificar sua filiação privada na organização 'ifesserra-lab'. Habilite o escopo 'read:org' (Classic PAT) ou permissão de leitura de 'Members' na organização 'ifesserra-lab' (Fine-Grained PAT)."*
     - `404 Not Found` (com escopo `read:org` presente ou como resposta direta de não-membro):
       - Consulta `GET {base_url}/orgs/{org}/members/{login}` como fallback caso `x-oauth-scopes` não contenha `read:org` nem `admin:org` nem `user`; se `204 No Content` → **Acesso Autorizado**.
       - Caso contrário → **Acesso Negado** com mensagem clara: *"A conta GitHub '@{login}' não pertence à organização 'ifesserra-lab'. Apenas membros ativos de https://github.com/orgs/ifesserra-lab têm acesso à aba egressos-dados."*
- **Rationale**:
  - Atende integralmente ao requisito `FR-003`, `FR-004`, `FR-005` e à Clarificação Q3.
  - Membros de organizações no GitHub têm filiação privada por padrão; `GET /user/memberships/orgs/ifesserra-lab` enxerga tanto membros privados quanto públicos e diferencia `state: "active"` de `state: "pending"`.
  - O fallback para `GET /orgs/ifesserra-lab/members/{login}` evita bloquear membros cuja filiação na `ifesserra-lab` já é pública mas que criaram um token apenas com escopo `repo`.
  - Como `GitHubClient` recebe `base_url` injetável, todos os cenários (200 active, 200 pending, 401 inválido, 403 sem `read:org`, 404 não-membro, 204 fallback público e erro de rede) são 100% testáveis offline com `wiremock` em `cargo test` (Princípio III).
- **Alternatives considered**:
  - *Consultar apenas `GET /orgs/ifesserra-lab/members/{username}`*: rejeitado porque retorna `404` para qualquer membro da `ifesserra-lab` cuja visibilidade na organização seja privada (padrão do GitHub).
  - *Validar no front-end via `fetch('https://api.github.com/...')`*: rejeitado porque viola o Princípio VI da Constituição (toda regra de negócio e validação deve residir no processo Rust e proteger os comandos IPC).

## R2 — Armazenamento Exclusivo em Memória na Sessão (`AppState`) e Ciclo de Vida (`FR-008`, `FR-009`)

- **Decision**: Estender `AppState` em `src-tauri/src/state.rs` com um slot protegido por `Mutex`:
  - `pub egressos_dados_auth: Mutex<Option<EgressosDadosAuth>>`
  - Onde `EgressosDadosAuth` contém `{ login: String, organization: String, token: String, authenticated_at: String }`.
  - Métodos de guarda em `AppState`:
    - `open_egressos_dados_session(&self, login: String, organization: String, token: String) -> Result<(), AppError>`
    - `close_egressos_dados_session(&self) -> Result<(), AppError>`
    - `current_egressos_dados_session(&self) -> Result<Option<EgressosDadosAuth>, AppError>`
    - `require_egressos_dados_session(&self) -> Result<EgressosDadosAuth, AppError>` — exige primeiro `self.require_session()?` (sessão local ativa) e depois verifica se `egressos_dados_auth` está preenchido com `organization == "ifesserra-lab"`, retornando `AppError::OrgUnauthorized` caso contrário (`FR-006`).
  - Além disso, `AppState::close_session(&self)` (chamado no comando `logout` geral do aplicativo) limpa **também** `egressos_dados_auth`, garantindo que o encerramento da sessão principal sempre revogue a sessão especial de `egressos-dados` (`US3-2`, `FR-008`).
- **Rationale**:
  - Cumpre `FR-008` e `FR-009` (Clarificação Q2): o token do login especial nunca é gravado em disco (`sync_config.json` ou SQLite), vive exclusivamente na memória do processo Rust enquanto a sessão estiver aberta, permanece ativo ao alternar entre `/projects`, `/dashboard`, `/src`, `/egressos` e `/egressos-dados`, e é destruído ao clicar em "Bloquear aba / Sair do login especial", ao fazer logout geral ou ao fechar o aplicativo.
  - Na próxima feature (quando o repositório de `egressos-dados` for conectado para ler os JSONs e criar as tabelas), os comandos de `egressos-dados` simplesmente chamarão `state.require_egressos_dados_session()?` para obter tanto a autorização validada na `ifesserra-lab` quanto o `token` em memória.
- **Alternatives considered**:
  - *Salvar o token em `localStorage` ou `sessionStorage` do navegador/WebView*: rejeitado porque exporia o token no lado JavaScript e não permitiria que o backend Rust bloqueasse chamadas IPC de forma autônoma (`FR-006`, `FR-009`).
  - *Sobrescrever `sync_config.json` dos outros projetos*: rejeitado porque `FR-008` e `FR-009` determinam que o login especial de `egressos-dados` deve ficar apenas em memória durante a sessão e não interferir em `Horizon`, `SRC` ou `Egressos`.

## R3 — Posicionamento na Navegação (`/projects`) e Estrutura da Aba `/egressos-dados` (`FR-001`, `FR-007`)

- **Decision**:
  1. Atualizar `DEFAULT_PROJECTS` em `frontend/src/components/ProjectSelector.jsx` e `PROJECTS` em `frontend/src/pages/projects.astro` para incluir o 4º card independente:
     - `id: 'egressos-dados'`, `title: 'Egressos — Dados (IFES Serra Lab)'`, `description: 'Acesso restrito a membros da organização GitHub ifesserra-lab (login via GitHub Token)'`, `href: '/egressos-dados'`.
  2. Criar a página `frontend/src/pages/egressos-dados/index.astro`, o layout `frontend/src/layouts/EgressosDadosDashboard.astro` e o componente React `frontend/src/components/EgressosDadosWorkspace.jsx`.
  3. Registrar `"/egressos-dados"` na constante `ROUTES` de `src-tauri/src/lib.rs` (atualizando a contagem de rotas verificadas pelo teste automatizado de 23 para 24).
  4. Comportamento de `EgressosDadosWorkspace.jsx`:
     - Ao montar, consulta `apiFetch('/egressos-dados/auth/status')`.
     - **Enquanto `authorized === false`**: renderiza exclusivamente o cartão de **Login Especial via GitHub Token (`ifesserra-lab`)** com campo `type="password"` para o token, explicação da exigência de membresia ativa em `https://github.com/orgs/ifesserra-lab/dashboard`, nota sobre a permissão `read:org` e exibição clara de erros tipados (`FR-002`, `FR-005`). Nenhuma tabela ou conteúdo interno é renderizado.
     - **Quando `authorized === true`**: renderiza o painel estrutural desbloqueado de `egressos-dados` contendo:
       - Cabeçalho de identificação do membro autenticado (`@{login}`), badge de verificação `Membro ativo: ifesserra-lab` e indicação mascarada do token (`…xxxx`);
       - Botão **"Bloquear aba / Trocar token"** (`DELETE /egressos-dados/auth/logout`), que limpa imediatamente a autorização em memória e retorna à tela de bloqueio (`FR-008`);
       - Área estrutural de tabelas desbloqueada (pronta para a próxima etapa em que o repositório dos JSONs de `egressos-dados` será conectado).
- **Rationale**:
  - Atende às Clarificações Q1 e Q4 (`FR-001`, `FR-002`, `FR-007`, `FR-010`), mantendo `Horizon`, `SRC` e `Egressos` 100% inalterados e isolados.
- **Alternatives considered**:
  - *Criar uma tela de login separada em `/egressos-dados/login`*: manter o gate diretamente dentro de `/egressos-dados` (controlado pelo estado retornado pelo Rust `egressos_dados_auth_status`) é mais seguro porque impede que navegação direta para `/egressos-dados` exiba o painel sem autorização no backend.
