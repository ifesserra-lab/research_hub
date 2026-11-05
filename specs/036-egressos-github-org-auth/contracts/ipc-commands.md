# Contract: Comandos IPC Tauri para Autenticação da Aba `egressos-dados` (`ipc-commands.md`)

**Date**: 2026-11-05 | **Feature**: [spec.md](../spec.md)

Todos os comandos abaixo exigem sessão local autenticada (`state.require_session()?`) e operam exclusivamente em Rust (`Princípio VI`), mantendo os comandos de `Horizon`, `SRC` e `Egressos` 100% inalterados (`FR-010`).

## 1. Comandos de Autenticação por Organização (`ifesserra-lab`)

### 1.1. `egressos_dados_login`
Autentica o usuário na aba `egressos-dados` validando o Token do GitHub informado e comprovando filiação ativa na organização `ifesserra-lab`.

- **Argumentos**:
  ```json
  {
    "token": "ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
  }
  ```
- **Comportamento**:
  1. Verifica sessão local (`state.require_session()?`).
  2. Faz `.trim()` no `token`; se vazio, limpa eventual sessão especial anterior e retorna `Err(AppError::Validation("Informe o Token de Acesso Pessoal do GitHub."))`.
  3. Instancia `GitHubClient` com `token` e executa `verify_org_membership("ifesserra-lab")`.
  4. Se aprovado (`state == "active"`), grava `EgressosDadosAuth` em memória no `AppState` e retorna `Ok(EgressosDadosAuthStatus)` com `authorized: true`.
  5. Se recusado (token inválido, usuário fora da `ifesserra-lab`, convite pendente, falta de escopo `read:org` ou erro de rede), limpa `egressos_dados_auth` e retorna o `AppError` correspondente (sem nunca incluir o token na mensagem).
- **Retorno de Sucesso**:
  ```json
  {
    "authorized": true,
    "login": "membro-ifesserra",
    "organization": "ifesserra-lab",
    "token_hint": "9f2a"
  }
  ```
- **Erros Tipados (`{ kind, message }`)**:
  - `validation`: token em branco.
  - `github_api` (401): `"Token do GitHub inválido ou expirado. Gere um novo token em github.com/settings/tokens."`
  - `org_unauthorized` (403/404 de membresia):
    - Não-membro: `"Acesso negado: a conta GitHub '@{login}' não é membro ativo da organização 'ifesserra-lab'."`
    - Convite pendente: `"Acesso negado: seu convite para a organização 'ifesserra-lab' ainda está pendente. Aceite o convite no GitHub e tente novamente."`
    - Sem permissão `read:org`: `"Não foi possível verificar sua filiação privada na organização 'ifesserra-lab'. Habilite a permissão 'read:org' no seu Token do GitHub."`
  - `network`: `"Sem conexão com github.com — verifique sua internet para validar o acesso à organização 'ifesserra-lab'."`

---

### 1.2. `egressos_dados_status`
Consulta o estado atual em memória da autorização da aba `egressos-dados` (`FR-002`, `FR-007`).

- **Argumentos**: `{}`
- **Retorno**: `Result<EgressosDadosAuthStatus, AppError>`
  - Quando autorizado na sessão:
    ```json
    {
      "authorized": true,
      "login": "membro-ifesserra",
      "organization": "ifesserra-lab",
      "token_hint": "9f2a"
    }
    ```
  - Quando bloqueado:
    ```json
    {
      "authorized": false,
      "login": null,
      "organization": "ifesserra-lab",
      "token_hint": null
    }
    ```

---

### 1.3. `egressos_dados_logout`
Encerra imediatamente a autorização especial e descarta da memória o token da aba `egressos-dados` (`FR-008`).

- **Argumentos**: `{}`
- **Retorno**: `Result<EgressosDadosAuthStatus, AppError>` (retorna o status bloqueado `authorized: false`).

---

### 1.4. Gate de Dados `egressos_dados_get_workspace`
Retorna os metadados do espaço de trabalho desbloqueado da aba `egressos-dados`, protegido por `state.require_egressos_dados_session()?` (`FR-006`, `US2-4`).

- **Argumentos**: `{}`
- **Comportamento**:
  - Chama `state.require_egressos_dados_session()?`.
  - Se a aba não estiver desbloqueada por um membro ativo de `ifesserra-lab`, falha imediatamente com `AppError::OrgUnauthorized` (`kind: "org_unauthorized"`).
  - Se autorizada, retorna os dados do painel estrutural desbloqueado (`organization: "ifesserra-lab"`, `login`, `token_hint`, `tables_ready: false`, mensagem de prontidão para conexão dos JSONs).
