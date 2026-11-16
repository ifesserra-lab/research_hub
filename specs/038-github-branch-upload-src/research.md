# Research: Upload GitHub em Nova Branch, Seleção de Branches e Liberação de Upload para o Projeto SRC

**Feature**: `038-github-branch-upload-src`  
**Date**: 2026-11-05  

## R1. Criação de Nova Branch via GitHub Git Data API (`POST /repos/{owner}/{repo}/git/refs`)

- **Decision**: Quando o modo de envio for **"Criar nova branch"** (`create_new_branch: true`), o `GitHubClient` em `src-tauri/src/sync_github.rs`:
  1. Valida o nome da nova branch via `sync_domain::validate_branch` (ou gera automaticamente `sync_domain::generate_branch_name(project)` no formato `update/<project>-<unix_ts>` se o campo vier vazio);
  2. Verifica se `refs/heads/{new_branch}` já existe no repositório (`GET /repos/{owner}/{repo}/git/ref/heads/{new_branch}`): se retornar `200`, aborta antes de enviar blobs com `AppError::Conflict` informando que a branch já existe;
  3. Lê o `base_sha` da branch base selecionada (`head_ref(owner, repo, base_branch)`);
  4. Cria o(s) blob(s) (`POST /git/blobs`), a árvore (`POST /git/trees` com `base_tree: base_sha`) e o commit (`POST /git/commits` com `parents: [base_sha]`);
  5. Cria a nova referência remota via `POST /repos/{owner}/{repo}/git/refs` com payload `{"ref": "refs/heads/<new_branch>", "sha": "<commit_sha>"}`.
- **Rationale**: A Git Data API do GitHub permite criar o commit completo sobre a árvore da branch base e registrar a nova branch em uma única chamada `POST /git/refs` ao final, mantendo a branch base (`main` ou outra escolhida) 100% intocada.
- **Alternatives considered**:
  - Criar a branch antes de enviar os blobs e depois dar `PATCH`: descartado porque em caso de falha de rede no meio do upload deixaria uma branch vazia/duplicada da `main` no repositório remoto.

## R2. Listagem e Seleção de Branches Existentes Além da `main` (`GET /repos/{owner}/{repo}/branches`)

- **Decision**: Adicionar no `GitHubClient` o método `list_branches(owner, repo) -> Result<Vec<RemoteBranchInfo>, AppError>` que consulta `GET {base_url}/repos/{owner}/{repo}/branches?per_page=100` e retorna a lista de branches (`name`, `commit_sha`), garantindo que a branch atualmente configurada (ex.: `main`) apareça sempre na lista no front-end.
- **Rationale**: Permite que o curador carregue com 1 clique todas as branches existentes do repositório configurado nos 4 projetos (`SyncPanel.jsx` para `horizon`, `src`, `egressos` e `EgressosDadosDataControl.jsx` para `egressos_dados`) e escolha qualquer branch além da `main` tanto como **branch base** (no modo "Criar nova branch") quanto como **branch de destino** (no modo "Enviar na branch selecionada").
- **Alternatives considered**:
  - Exigir que o usuário digite manualmente o nome de qualquer branch diferente da `main`: mantido como fallback caso o repositório ainda não tenha sido consultado, mas complementado pelo seletor populado via API.

## R3. Liberação do Upload GitHub para o Projeto SRC (`src`)

- **Decision**:
  1. Atualizar `sync_domain::check_upload_allowed` para autorizar `"src"` junto com `"horizon"`, `"egressos"` e `"egressos_dados"`.
  2. Atualizar `sync_domain::validate_path` para exigir extensão `.json` quando `project == "src"` (`"O export do SRC é o JSON consolidado: o caminho deve terminar em '.json'."`).
  3. Em `commands::github_upload`, quando `project == "src"`, invocar `crate::src_domain::export(&conn)?.into_bytes()` e enviá-lo via `upload_horizon_export` (que publica um único arquivo via Git Data API, usando mensagem de commit `"Export consolidado SRC ResearchHub"` quando `project == "src"`).
  4. Em `frontend/src/pages/src/index.astro`, alterar `<SyncPanel client:load project="src" allowUpload={false} />` para `<SyncPanel client:load project="src" allowUpload={true} />`.
- **Rationale**: Reutiliza exatamente o motor `src_domain::export(&conn)` já validado para a exportação local (`SC-003`) e o pipeline de upload de arquivo único do `GitHubClient`, garantindo paridade byte a byte entre o export manual local e o upload para o GitHub.
- **Alternatives considered**: Criar uma função duplicada de upload exclusiva para o SRC — descartado por violar DRY, já que tanto `horizon` (`.zip`) quanto `src` (`.json`) enviam um único blob para um caminho de arquivo.

## R4. Preservação da Branch Base Após Criação de Nova Branch

- **Decision**: Conforme decidido na Clarificação Q2, quando o upload ocorre no modo **"Criar nova branch"**, a configuração persistida do projeto (`ProjectSyncConfig.branch`, ex.: `main`) permanece apontando para a branch base, enquanto o `UploadResult` retorna `branch: new_branch_name` e `created_branch: true`, e a interface adiciona a branch recém-criada à lista de branches disponíveis e sugere um novo nome para o próximo envio.
- **Rationale**: Evita que envios subsequentes criem ramificações em cascata sem intenção, preservando `main` (ou a branch base escolhida) como ponto de partida limpo.
