# Feature Specification: Upload GitHub em Nova Branch, Seleção de Branches e Liberação de Upload para o Projeto SRC

**Feature Branch**: `038-github-branch-upload-src`  
**Created**: 2026-11-05  
**Status**: Draft  
**Input**: User description: "proxima feature , vale para todas os programas suportaddos (horizon , src , egressos e egressos_data) , agora quando houver upload para o github , precisa criar uma nova branch com os novos dados ao inves de ir na branch selecionada , ou poder escolher a branch alem da main , alem disso pode liberar o src para realizar a funçao de upload pro github que estava bloquada antes"

## Clarifications

### Session 2026-11-05

- Q: No envio para o GitHub nos quatro programas (`horizon`, `src`, `egressos` e `egressos_dados`), o usuário deve poder alternar entre criar uma nova branch e enviar para uma branch existente escolhida na lista, ou todo envio deve criar obrigatoriamente uma nova branch? (FR-004) → A: Dois modos explícitos no painel de upload: "Criar nova branch" pré-selecionado por padrão (ramificando da branch base selecionada, com nome sugerido editável) e opção "Enviar na branch selecionada" (permitindo escolher `main` ou qualquer outra branch existente do repositório).
- Q: Após concluir um upload no modo "Criar nova branch", a branch configurada do projeto deve continuar sendo a branch base (ex.: `main`) para futuros envios, ou deve passar automaticamente para a nova branch recém-criada? (FR-005) → A: Manter a branch base configurada (ex.: `main`) como referência padrão para futuras ramificações e atualizar a lista de branches disponíveis incluindo a recém-criada.
- Q: Ao liberar o envio do projeto SRC (`src`) para o GitHub, o sistema deve enviar diretamente o JSON consolidado completo (idêntico à exportação manual local) seguindo o mesmo fluxo dos demais projetos? (FR-002) → A: Envio direto do JSON consolidado completo do SRC (`src_domain::export`, byte a byte idêntico ao export manual local), sem bloqueio de política e usando o mesmo fluxo de criação/seleção de branch dos demais projetos.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Criar uma nova branch automaticamente (ou com nome personalizado) ao fazer upload para o GitHub em qualquer um dos 4 projetos (Priority: P1) 🎯 MVP

Um curador autenticado trabalhando em qualquer um dos quatro programas suportados pelo ResearchHub (**Horizon**, **SRC**, **Egressos** ou **Egressos — Dados (`egressos-dados`)**) deseja enviar suas alterações locais para o repositório GitHub sem sobrescrever diretamente a branch principal (`main`). Ao acionar o envio para o GitHub, o sistema permite criar uma **nova branch** no repositório remoto (a partir de uma branch base, como `main`) contendo o commit com os novos dados exportados — seja gerando automaticamente um nome padronizado com data/hora (ex.: `update/horizon-20261105-140800`) ou permitindo que o curador informe o nome desejado para a nova branch.

**Why this priority**: Evitar commits diretos acidentais na branch `main` e permitir fluxo de revisão/Pull Request em todos os 4 projetos é a mudança central solicitada e protege os repositórios de produção.

**Independent Test**: Em qualquer um dos 4 projetos (`horizon`, `src`, `egressos`, `egressos_dados`), selecionar o modo "Criar nova branch no envio", informar (ou usar a sugestão automática de) nome da nova branch derivada de `main`, executar o upload e verificar que a nova referência `refs/heads/<nova-branch>` é criada no GitHub com o commit dos dados exportados, mantendo a branch `main` intacta.

**Acceptance Scenarios**:

1. **Given** que o curador está no painel de sincronização de `horizon`, `src`, `egressos` ou `egressos_dados` com o modo **"Criar nova branch no upload"** selecionado (padrão recomendado), **When** clica em enviar para o GitHub, **Then** o sistema obtém o commit atual da branch base selecionada (ex.: `main`), gera o(s) blob(s), a árvore e o commit com os novos dados e cria uma nova branch remota no GitHub apontando para esse novo commit, exibindo o nome da branch criada e o link para o commit/comparação.
2. **Given** que o curador escolhe criar uma nova branch mas deixa o campo de nome da nova branch em branco (ou clica em gerar nome automático), **When** realiza o upload, **Then** o sistema utiliza um nome de branch único e descritivo contendo o identificador do projeto e o carimbo de data/hora, criando a branch sem erro.
3. **Given** que o curador informa como nome de "nova branch" uma branch que já existe no repositório remoto, **When** tenta realizar o upload no modo "Criar nova branch", **Then** o sistema avisa claramente que a branch já existe e oferece usar o modo de envio para branch existente ou escolher outro nome.

---

### User Story 2 - Escolher qualquer branch existente além da `main` para download, base ou upload nos 4 projetos (Priority: P2)

Um curador precisa trabalhar com múltiplas branches no repositório GitHub (por exemplo, `develop`, `homologacao`, `curadoria-2026` ou uma branch criada anteriormente pelo próprio ResearchHub) em vez de ficar restrito à branch `main`. No painel de sincronização dos quatro projetos (`horizon`, `src`, `egressos` e `egressos_dados`), o curador pode carregar/listar as branches existentes no repositório configurado e escolher qual branch utilizar tanto como branch base (para criar uma nova branch a partir dela) quanto como branch de destino direta (caso opte por atualizar uma branch específica existente além da `main`).

**Why this priority**: Complementa a criação de novas branches permitindo que o curador selecione branches existentes do repositório remoto sem precisar memorizar nomes ou ficar preso à `main`.

**Independent Test**: Configurar um repositório válido em qualquer um dos 4 projetos, clicar em listar/atualizar branches do repositório, selecionar uma branch diferente de `main` no seletor e executar o upload (direto na branch escolhida ou criando uma nova branch derivada dela).

**Acceptance Scenarios**:

1. **Given** um repositório válido configurado em `horizon`, `src`, `egressos` ou `egressos_dados`, **When** o painel de sincronização carrega ou o curador clica em **" Carregar branches"**, **Then** o sistema consulta as branches existentes no repositório GitHub e apresenta uma lista selecionável (além de permitir digitação livre) contendo `main` e todas as demais branches disponíveis.
2. **Given** que o curador alterna para o modo **"Enviar para branch selecionada"** e escolhe uma branch existente (ex.: `curadoria-lote-2` ou `main`), **When** confirma o envio, **Then** o sistema publica o commit diretamente na branch escolhida.
3. **Given** que o curador escolhe uma branch diferente de `main` (ex.: `develop`) como **branch base** no modo **"Criar nova branch"**, **When** executa o upload, **Then** a nova branch é ramificada a partir do topo (`HEAD`) de `develop`.

---

### User Story 3 - Liberar o upload do projeto SRC para o GitHub (Priority: P2)

Um curador do projeto **SRC** (Ações de Extensão e Ensino / Participações) terminou de revisar e curar os registros localmente na aba `/src` e deseja publicar o arquivo JSON consolidado diretamente no repositório GitHub do SRC, utilizando os mesmos controles de configuração de repositório, escolha de branch ou criação de nova branch já disponíveis nos demais projetos.

**Why this priority**: O projeto SRC era o único dos 4 programas cujo envio para o GitHub estava bloqueado por política da versão anterior; sua liberação foi explicitamente solicitada pelo usuário para unificar a capacidade de upload nos 4 projetos.

**Independent Test**: Acessar `/src`, configurar o repositório de destino e o caminho do arquivo `.json` (ex.: `data/src_consolidado.json`), escolher criar uma nova branch (ou selecionar uma branch existente) e executar o upload, verificando que o JSON consolidado exportado pela base local do SRC é publicado no GitHub com sucesso.

**Acceptance Scenarios**:

1. **Given** que o curador acessa a página do projeto **SRC** (`/src`), **When** visualiza o painel de sincronização com o GitHub, **Then** a seção de envio (upload) está visível e habilitada, permitindo configurar repositório (`owner/repo`), caminho do arquivo `.json` (ex.: `src_consolidado.json`), modo de branch (nova branch vs. branch selecionada) e botão de envio.
2. **Given** que há ações e participações carregadas na base local do SRC, **When** o curador executa o upload para o GitHub, **Then** o sistema gera o JSON consolidado canônico do SRC (idêntico ao export manual local) e publica-o no repositório/branch de destino sem bloqueio de política (`sync_policy`).
3. **Given** que o curador informa um caminho de destino para o projeto SRC que não termina em `.json`, **When** tenta salvar a configuração ou enviar, **Then** o sistema exibe uma mensagem de validação clara informando que o export do SRC deve apontar para um arquivo `.json`.

---

### Edge Cases

- **O que acontece quando o repositório remoto ainda não possui a branch informada no modo "Enviar para branch selecionada"?** Se a branch selecionada não existir no remoto, o sistema pode criá-la automaticamente a partir da branch padrão do repositório (ou da `main`) ou orientar o curador, garantindo que o upload não falhe por falta de referência prévia.
- **O que acontece se o nome da nova branch contiver caracteres inválidos para o Git (espaços, `..`, iniciar com `-` ou `/`)?** O sistema valida o nome da branch localmente antes de qualquer requisição de rede e apresenta mensagem acionável.
- **O que acontece se o curador tentar listar branches de um repositório privado sem token configurado (ou com token sem permissão)?** O sistema informa claramente que é necessário configurar/validar o Token do GitHub para listar as branches daquele repositório privado, mantendo a possibilidade de digitar o nome da branch manualmente.
- **Como o upload do `egressos-dados` se comporta com a criação de nova branch?** O painel em `/egressos-dados/controle` reutiliza automaticamente o Token do GitHub da sessão `ifesserra-lab` para listar branches de `ifesserra-lab/egressos-dados` e criar a nova branch (ou enviar para a branch escolhida) com os 39 arquivos JSON em um único commit atômico.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: O sistema DEVE suportar o envio (upload) para o GitHub em **todos os 4 programas** do ResearchHub: **Horizon** (`horizon`, arquivo `.zip`), **SRC** (`src`, arquivo `.json` consolidado), **Egressos** (`egressos`, diretório com 21 arquivos `.json`) e **Egressos — Dados** (`egressos_dados`, 39 arquivos `.json` na raiz e em `data/`).
- **FR-002**: O sistema DEVE remover integralmente o bloqueio anterior de política (`sync_policy`) sobre o projeto **SRC**, habilitando o painel de upload na página `/src` (`allowUpload={true}`) e permitindo publicar diretamente no GitHub o JSON consolidado completo gerado a partir da base SQLite local do SRC (`src_domain::export`), com o mesmo fluxo de criação/seleção de branch dos demais projetos.
- **FR-003**: Para o projeto **SRC**, a validação de caminho de destino no repositório GitHub DEVE exigir extensão `.json` (ex.: `src_consolidado.json` ou `dados/consolidado.json`).
- **FR-004**: Em todos os 4 projetos (`horizon`, `src`, `egressos`, `egressos_dados`), o fluxo de upload para o GitHub DEVE oferecer dois modos explícitos de destino de branch:
  1. **Criar nova branch com os novos dados** (modo padrão no envio): cria uma nova branch no repositório GitHub a partir de uma branch base selecionada (por padrão `main`), gravando os novos dados nessa nova branch sem alterar a branch base.
  2. **Enviar na branch selecionada**: grava o commit diretamente na branch escolhida pelo curador (`main` ou qualquer outra branch selecionada/informada).
- **FR-005**: Quando o modo **"Criar nova branch"** estiver ativo, o sistema DEVE pré-preencher (ou gerar automaticamente caso o campo seja deixado em branco) uma sugestão de nome de branch no formato `update/<projeto>-<YYYYMMDD-HHMMSS>` (permitindo que o curador edite livremente o nome da nova branch antes de enviar). Após a conclusão do upload em nova branch, a branch base configurada do projeto (ex.: `main`) DEVE permanecer inalterada como referência padrão para próximas ramificações, enquanto a nova branch criada passa a constar na lista de branches disponíveis do repositório.
- **FR-006**: O sistema DEVE permitir consultar e listar as branches existentes do repositório GitHub configurado (`GET /repos/{owner}/{repo}/branches`) em todos os 4 projetos, para que o curador possa escolher qualquer branch existente além da `main` tanto como branch base quanto como branch de destino.
- **FR-007**: Ao criar uma nova branch durante o upload no GitHub, o sistema DEVE:
  1. Resolver o SHA do commit `HEAD` da branch base selecionada (`GET /repos/{owner}/{repo}/git/ref/heads/{base_branch}`);
  2. Criar o(s) blob(s), a árvore Git (`base_tree` apontando para o commit base) e o novo commit (`parents: [base_sha]`);
  3. Criar a nova referência remota via Git Data API (`POST /repos/{owner}/{repo}/git/refs` com `ref: "refs/heads/{new_branch}"` e `sha: "{commit_sha}"`).
- **FR-008**: Mesmo no modo **"Enviar na branch selecionada"**, caso a branch informada pelo curador ainda não exista no repositório remoto, o sistema DEVE permitir criá-la a partir da branch base (`main`) em vez de abortar com erro, ou atualizar a referência existente (`PATCH /repos/{owner}/{repo}/git/refs/heads/{branch}`) quando ela já existir.
- **FR-009**: Após a conclusão do upload (seja em nova branch ou em branch existente), a interface DEVE exibir o nome da branch onde o commit foi publicado e o link direto para o commit/branch no GitHub.
- **FR-010**: Toda a lógica de validação de nomes de branch, geração de pacotes/JSONs e comunicação com a Git Data API do GitHub DEVE residir no núcleo Rust (`src-tauri/src/`), mantendo `frontend/src/services/api.js` como único ponto de acoplamento entre o front-end e o núcleo.

### Key Entities *(include if feature involves data)*

- **Modo de Destino de Branch (`BranchUploadMode`)**: Define se o envio criará uma nova branch (`create_branch`, com `base_branch` e `new_branch_name`) ou atualizará uma branch selecionada (`existing_branch`, com `target_branch`).
- **Branch Remota (`RemoteBranchInfo`)**: Representa uma branch existente no repositório GitHub consultado (`name`, `commit_sha`), usada para popular o seletor de branches além da `main`.
- **Resultado de Upload (`UploadResult`)**: Informações retornadas após a publicação no GitHub, incluindo `commit_sha`, `html_url`, `branch` (nome da branch efetivamente criada ou atualizada), `created_branch` (indicando se uma nova branch foi criada) e `replaced`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% dos 4 programas suportados (`horizon`, `src`, `egressos`, `egressos_dados`) conseguem publicar seus respectivos pacotes/arquivos em uma **nova branch** criada automaticamente no GitHub sem modificar o `HEAD` da branch base (`main`).
- **SC-002**: 100% dos 4 programas permitem listar e selecionar qualquer branch existente no repositório remoto além da `main` em menos de 3 segundos em condições normais de rede.
- **SC-003**: O envio do projeto **SRC** para o GitHub produz no repositório remoto um arquivo `.json` byte a byte equivalente ao gerado pela exportação manual local do SRC.
- **SC-004**: Zero vazamento de tokens de acesso pessoal em logs, mensagens de erro ou arquivos de banco de dados durante a listagem de branches e criação de novas branches nos 4 projetos.

## Assumptions

- Para os projetos `horizon`, `src` e `egressos`, o sistema utiliza o Token de Acesso Pessoal do GitHub configurado no `SyncPanel` (`sync_config.json`), enquanto para `egressos_dados` continua reutilizando automaticamente o Token em memória da sessão autenticada da organização `ifesserra-lab`.
- O modo padrão selecionado na interface de upload passa a ser **"Criar nova branch com os novos dados"** (com `main` ou a branch configurada como branch base), garantindo segurança por padrão, mas com alternância em 1 clique para **"Enviar na branch selecionada"**.
