# Contract: Rotas Front-end e Ponte `api.js` (`frontend-routes.md`)

**Date**: 2026-11-05 | **Feature**: [spec.md](../spec.md)

## 1. Rotas em `frontend/src/services/api.js`

Em conformidade com a diretriz arquitetural do repositório (`AGENTS.md` e Princípio VI da Constituição), toda comunicação JS→Rust passa exclusivamente por `frontend/src/services/api.js`:

| Método + Endpoint (`apiFetch`) | Comando Tauri Invocado | Payload / Parâmetros |
|--------------------------------|------------------------|----------------------|
| `GET /egressos-dados/auth/status` | `invoke('egressos_dados_status')` | — |
| `POST /egressos-dados/auth/login` | `invoke('egressos_dados_login', { token: b?.token ?? '' })` | `{ token: string }` |
| `DELETE /egressos-dados/auth/logout` | `invoke('egressos_dados_logout')` | — |
| `GET /egressos-dados/workspace` | `invoke('egressos_dados_get_workspace')` | — |

> **Ordem em `ROUTES`**: As rotas `/^\/egressos-dados\/.../` devem ser declaradas antes dos padrões genéricos `/^\/([^/?]+)$/`, garantindo que `/egressos-dados/*` nunca seja interpretada como uma entidade do Horizon.

---

## 2. Navegação e Páginas Astro

### 2.1. Tela de Seleção de Projetos (`/projects` e `ProjectSelector.jsx`)
`DEFAULT_PROJECTS` em `ProjectSelector.jsx` e `PROJECTS` em `projects.astro` passam a exibir **4 cartões independentes** (`FR-001`, Clarificação Q1):
1. **Horizon** (`/dashboard`)
2. **SRC** (`/src`)
3. **Egressos** (`/egressos`)
4. **Egressos — Dados (`egressos-dados`)** (`/egressos-dados`) — identificado visualmente como área restrita à organização GitHub `ifesserra-lab`.

### 2.2. Nova Página `/egressos-dados` (`frontend/src/pages/egressos-dados/index.astro`)
- Utiliza o layout exclusivo `EgressosDadosDashboard.astro` (com menu lateral próprio, link `↩ Trocar projeto` para `/projects` e botão `Sair` que encerra a sessão geral).
- Renderiza `EgressosDadosWorkspace.jsx` (`client:load`), que controla os dois estados:
  1. **Estado Bloqueado (`authorized === false`)**:
     - Exibe o formulário de **Login Especial via GitHub Token — Organização `ifesserra-lab`**.
     - Campo de senha (`type="password"`) para o token (`ghp_...` / `github_pat_...`).
     - Mensagens de erro claras para token inválido, conta fora da organização `ifesserra-lab`, convite pendente, falta de escopo `read:org` ou erro de conexão.
     - Oculta completamente a área de tabelas.
  2. **Estado Desbloqueado (`authorized === true`)**:
     - Exibe o painel estrutural desbloqueado com o `@login` do membro autenticado, badge `Organização verificada: ifesserra-lab`, terminação do token (`…xxxx`) e botão **"Bloquear aba / Trocar token"**.
     - Exibe a seção estrutural das tabelas de `egressos-dados` (confirmando o acesso autorizado via `GET /egressos-dados/workspace`), pronta para receber a leitura dos JSONs do GitHub na próxima etapa.

### 2.3. Registro em `src-tauri/src/lib.rs`
- A constante `ROUTES` em `src-tauri/src/lib.rs` inclui `"/egressos-dados"`, elevando o total de rotas verificadas pelo teste `todas_as_rotas_do_menu_resolvem_com_o_href_atual` de 23 para 24.
