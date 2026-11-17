# Quickstart & Validation Guide: Controle de Acesso `egressos-dados` (`036-egressos-github-org-auth`)

**Date**: 2026-11-05 | **Feature**: [spec.md](./spec.md)

## 1. Pré-requisitos

- Ambiente Rust (`cargo`) e Node.js (`npm`) configurados no repositório `/home/thiagodeps/research_hub`.
- Para teste manual real contra o GitHub: um Personal Access Token (PAT) de uma conta membro ativa da organização [`ifesserra-lab`](https://github.com/orgs/ifesserra-lab/dashboard) (com escopo `read:org` caso a visibilidade do membro na organização seja privada).

## 2. Execução da Suíte Automatizada (TDD — Princípios I e III)

### 2.1. Testes do Núcleo Rust (`cargo test`)

```bash
cd /home/thiagodeps/research_hub/src-tauri
cargo test
```

**O que os testes verificam**:
1. **Estado e Guarda de Sessão (`state.rs`)**:
   - `require_egressos_dados_session` recusa chamadas quando `egressos_dados_auth` é `None` (`OrgUnauthorized`) ou quando a sessão principal do app está fechada (`Unauthenticated`).
   - `open_egressos_dados_session` autoriza a sessão em memória com `login`, `organization: "ifesserra-lab"` e `token` (mascarando o `token_hint` com apenas os últimos 4 caracteres).
   - `close_egressos_dados_session` e `close_session` (logout geral) limpam imediatamente a autorização e o token da memória (`FR-008`, `US3-2`).
2. **Verificação de Membresia na Organização `ifesserra-lab` (`tests/sync_github.rs` via `wiremock`)**:
   - **Membro ativo (`200 OK` com `state: "active"` em `/user/memberships/orgs/ifesserra-lab`)**: login especial autorizado com retorno de `login` e `organization: "ifesserra-lab"`.
   - **Membro público com fallback (`403` em `/user/memberships/orgs/ifesserra-lab` + `204 No Content` em `/orgs/ifesserra-lab/members/{login}`)**: login especial autorizado.
   - **Convite pendente (`200 OK` com `state: "pending"`)**: acesso negado com mensagem explícita de convite pendente (`Edge Cases`, `FR-004`).
   - **Usuário fora da organização (`404` em `/user/memberships/orgs/ifesserra-lab` e `404` em `/orgs/ifesserra-lab/members/{login}`)**: acesso negado informando que `@{login}` não pertence à organização `ifesserra-lab` (`US2-1`).
   - **Token sem escopo `read:org` e não público (`403` em `/user/memberships/orgs/ifesserra-lab` e `404` em `/orgs/ifesserra-lab/members/{login}`)**: acesso negado orientando habilitar `read:org` (`US2-3`).
   - **Token inválido/revogado (`401` em `/user`)**: acesso negado informando token inválido, garantindo que o valor do token **nunca** aparece na mensagem de erro (`US2-2`, `SC-003`).
3. **Resolução de Rotas (`lib.rs`)**:
   - Todas as 24 rotas em `ROUTES` (incluindo `"/egressos-dados"`) resolvem no asset resolver do binário Tauri.

### 2.2. Testes do Front-end (`npm test`)

```bash
cd /home/thiagodeps/research_hub/frontend
npm test
```

**O que os testes verificam**:
- `ProjectSelector`: exibe os 4 projetos (**Horizon**, **SRC**, **Egressos** e **Egressos — Dados (`egressos-dados`)**) e navega para `/egressos-dados`.
- `api.js`: mapeia corretamente `GET /egressos-dados/auth/status`, `POST /egressos-dados/auth/login`, `DELETE /egressos-dados/auth/logout` e `GET /egressos-dados/workspace`.
- `EgressosDadosWorkspace`:
  - Renderiza a tela de bloqueio por Token do GitHub quando `authorized === false`, ocultando o painel interno.
  - Exibe mensagens de erro claras quando o login falha (token inválido, usuário fora da `ifesserra-lab`, convite pendente, falta de `read:org`).
  - Desbloqueia e exibe o painel estrutural com `@login`, badge `ifesserra-lab` e botão de desconectar quando o login especial tem sucesso.
  - Ao clicar em **"Bloquear aba / Trocar token"**, invoca o logout especial e retorna imediatamente à tela bloqueada.

## 3. Validação Manual Rápida no Aplicativo Desktop

1. Compilar o front-end (`cd frontend && npm run build`) e iniciar o aplicativo (`cargo tauri dev`).
2. Fazer login local (`admin@admin.com` / `admin123`) e verificar os 4 cartões em `/projects`.
3. Clicar em **Egressos — Dados (`egressos-dados`)** e verificar que a aba abre bloqueada exigindo Token do GitHub de membro da organização `ifesserra-lab`.
4. Testar um token inválido ou de conta fora da `ifesserra-lab` e confirmar o bloqueio com a mensagem correspondente.
5. Inserir um token válido de membro da `ifesserra-lab`, confirmar o desbloqueio do painel estrutural com `@login` + badge `ifesserra-lab`, alternar para outro projeto e voltar para `/egressos-dados` verificando que permanece desbloqueado na sessão, e por fim clicar em **Bloquear aba / Trocar token** para confirmar o retorno imediato ao bloqueio.
