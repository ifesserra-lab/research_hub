# Feature Specification: Controle de Acesso Exclusivo à Aba `egressos-dados` por Organização do GitHub (`ifesserra-lab`)

**Feature Branch**: `036-egressos-github-org-auth`

**Created**: 2026-11-05

**Status**: Draft

**Input**: User description: "a proxima feature e a implementaçao de mais uma aba , a de egressos-dados , porem antes de passar o github dela para ler os jsons e criar as tabelas , precisamos fazer um sistema de login especial , em que somente quem e da organizaçao do github ifesserra-lab (https://github.com/orgs/ifesserra-lab/dashboard) consegue acesso a essa aba com essas tabelas , o metodo de login especial seria por meio de github token"

## Contexto

O Research Hub está recebendo mais uma aba/área de trabalho dedicada a **`egressos-dados`**. No entanto, diferentemente das áreas comuns do aplicativo (acessíveis apenas pelo login local de administrador), os dados e tabelas de **`egressos-dados`** são restritos e exigem uma camada adicional de controle de acesso institucional.

Antes de conectar o repositório do GitHub dessa aba para ler os arquivos JSON e popular as tabelas na próxima etapa, esta feature estabelece a **aba `egressos-dados` protegida por um sistema de login especial via Token do GitHub**: somente usuários cujo token de acesso pessoal do GitHub pertença a um **membro ativo da organização `ifesserra-lab`** (`https://github.com/orgs/ifesserra-lab/dashboard`) conseguem desbloquear e acessar essa aba e a área onde ficarão suas tabelas.

## Clarifications

### Session 2026-11-05

- Q: Onde a nova aba `egressos-dados` deve ser posicionada na navegação do aplicativo e como ela se relaciona com a área `Egressos` já existente? (FR-001) → A: **Opção A** — Criar `egressos-dados` como um 4º card/projeto independente na tela de seleção de projetos (`/projects`) ao lado de `Horizon`, `SRC` e `Egressos`, com sua própria área e menu lateral exclusivos protegidos pelo login da organização `ifesserra-lab`.
- Q: Por quanto tempo a autorização e o token do login especial de `egressos-dados` devem permanecer ativos no computador? (FR-008) → A: **Opção A** — Apenas durante a sessão ativa (em memória): o acesso fica liberado enquanto a sessão estiver aberta (mesmo trocando entre projetos), mas encerrar a sessão ou fechar o aplicativo exige informar o token novamente no próximo acesso.
- Q: Como o sistema deve validar a participação de membros na organização `ifesserra-lab` quando o membro tem visibilidade privada na organização? (FR-003) → A: **Opção A** — Filiação autenticada com fallback público: consulta primeiro a filiação autenticada na organização (`state: active`); se faltar permissão `read:org` no token, consulta a membresia pública da organização e, caso não conste publicamente, orienta o usuário a habilitar `read:org` no token.
- Q: Após o membro da organização `ifesserra-lab` desbloquear a aba `egressos-dados`, o que deve ser exibido na área interna nesta entrega (antes de conectarmos o repositório dos JSONs na próxima etapa)? (FR-007) → A: **Opção A** — Painel estrutural desbloqueado com status do membro: menu lateral próprio de `egressos-dados`, identificação do membro (`@login` + badge `ifesserra-lab`), botão de desconectar/bloquear aba e área pronta aguardando as tabelas da próxima etapa.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Autenticar-se com Token do GitHub como membro da organização `ifesserra-lab` para acessar a aba `egressos-dados` (Priority: P1)

Ao navegar pelo aplicativo e selecionar a nova aba **`egressos-dados`**, o usuário que ainda não realizou o login especial de organização vê uma tela de bloqueio/autenticação dedicada dessa aba (em vez do conteúdo interno). Nessa tela, o usuário informa seu Token de Acesso Pessoal do GitHub e aciona a entrada. O sistema valida o token junto ao GitHub, verifica que a conta associada ao token é membro ativo da organização **`ifesserra-lab`** e, em caso afirmativo, libera imediatamente o acesso à aba **`egressos-dados`**, exibindo o ambiente desbloqueado (pronto para receber as tabelas de dados na próxima etapa) e identificando o membro autenticado.

**Why this priority**: É o requisito central de segurança e pré-requisito obrigatório antes de carregar qualquer JSON ou tabela na aba `egressos-dados`. Sem essa barreira verificada, dados restritos da organização ficariam acessíveis a qualquer usuário local do aplicativo.

**Independent Test**: Abrir o aplicativo, clicar na aba `egressos-dados`, verificar que o conteúdo está bloqueado pelo login especial, inserir um token válido de um membro da organização `ifesserra-lab` e confirmar que a aba é desbloqueada e exibe a identificação do membro da organização.

**Acceptance Scenarios**:

1. **Given** um usuário autenticado no aplicativo que seleciona a aba `egressos-dados` sem estar autenticado na organização, **When** a aba é aberta, **Then** o sistema exibe a tela de login especial por Token do GitHub informando que o acesso é exclusivo para membros da organização `ifesserra-lab`, mantendo a área de tabelas oculta e inacessível.
2. **Given** a tela de login especial da aba `egressos-dados`, **When** o usuário informa um Token do GitHub válido pertencente a um membro ativo da organização `ifesserra-lab` e confirma, **Then** o sistema valida a participação na organização `ifesserra-lab`, autoriza a sessão especial e libera a visualização interna da aba `egressos-dados` com o nome de usuário (`login`) do membro autenticado.
3. **Given** um usuário que já concluiu com sucesso o login especial na aba `egressos-dados` durante a sessão ativa, **When** ele alterna para outras abas/projetos (como Horizon ou SRC) e retorna para `egressos-dados`, **Then** o acesso permanece liberado durante a sessão sem exigir que o token seja digitado novamente a cada clique.

---

### User Story 2 - Bloquear acesso a tokens inválidos, sem permissão ou de usuários fora da organização `ifesserra-lab` (Priority: P2)

Quando um usuário tenta acessar a aba **`egressos-dados`** informando um Token do GitHub inválido/expirado, ou um token válido de uma conta pessoal do GitHub que **não faz parte** da organização **`ifesserra-lab`**, o sistema recusa a entrada com uma mensagem clara e específica para o motivo da recusa, mantendo a aba e qualquer operação de dados de `egressos-dados` totalmente bloqueadas.

**Why this priority**: Garante que a regra de negócio ("somente quem é da organização `ifesserra-lab` consegue acesso") seja efetivamente aplicada tanto na interface quanto nas operações internas, distinguindo claramente entre credencial inválida e falta de vínculo com a organização.

**Independent Test**: Tentar realizar o login especial na aba `egressos-dados` primeiro com um token inválido e depois com um token válido de um usuário do GitHub que não pertence à `ifesserra-lab`, verificando que em ambos os casos o acesso é negado com a explicação correspondente e nenhum conteúdo interno é exibido.

**Acceptance Scenarios**:

1. **Given** a tela de login especial da aba `egressos-dados`, **When** o usuário informa um Token do GitHub válido de uma conta que **não é membro** da organização `ifesserra-lab`, **Then** o sistema nega o acesso, exibe mensagem clara informando que o usuário autenticado não pertence à organização `ifesserra-lab` e mantém a aba bloqueada.
2. **Given** a tela de login especial da aba `egressos-dados`, **When** o usuário informa um token inválido, revogado ou malformado, **Then** o sistema nega o acesso com mensagem informando que o token é inválido ou expirou, sem expor o valor do token na tela ou em logs.
3. **Given** a tela de login especial da aba `egressos-dados`, **When** o usuário informa um token válido que não possui permissão de leitura de organização necessária para verificar membresia privada em `ifesserra-lab`, **Then** o sistema orienta claramente qual permissão (`read:org` / leitura de membros da organização) precisa estar habilitada no token.
4. **Given** uma tentativa de acessar diretamente as rotas ou comandos internos da aba `egressos-dados` sem ter passado pelo login especial da organização `ifesserra-lab`, **When** a requisição é processada, **Then** o sistema recusa a operação com erro de acesso não autorizado à organização.

---

### User Story 3 - Encerrar ou trocar a credencial do login especial na aba `egressos-dados` (Priority: P3)

Dentro da aba **`egressos-dados`** já desbloqueada, o membro autenticado da organização `ifesserra-lab` visualiza sob qual conta do GitHub o acesso foi concedido e pode encerrar esse login especial (sair/bloquear a aba ou trocar de token) a qualquer momento, fazendo com que a aba volte imediatamente ao estado bloqueado.

**Why this priority**: Permite uso seguro em máquinas compartilhadas no laboratório e facilita a troca ou revogação da credencial sem precisar reinstalar ou limpar manualmente arquivos do aplicativo.

**Independent Test**: Desbloquear a aba `egressos-dados` com um token válido da organização `ifesserra-lab`, acionar a opção de sair/bloquear o acesso especial e verificar que a aba retorna imediatamente à tela de login por token e recusa acesso às tabelas até nova autenticação.

**Acceptance Scenarios**:

1. **Given** a aba `egressos-dados` desbloqueada por um membro da organização `ifesserra-lab`, **When** o usuário aciona a opção de encerrar o login especial (desconectar token da aba), **Then** o estado de autorização da organização é encerrado imediatamente e a aba volta a exibir a tela de bloqueio por Token do GitHub.
2. **Given** o usuário que encerrou a sessão principal do aplicativo (logout geral), **When** um novo login é feito no aplicativo, **Then** a autorização especial da aba `egressos-dados` não permanece aberta indevidamente sem validação.

---

### Edge Cases

- **Membro com visibilidade privada na organização `ifesserra-lab`**: Muitos membros de organizações no GitHub têm sua filiação configurada como privada. A verificação não pode depender apenas da listagem pública de membros caso o token pertença a um membro privado; deve consultar a filiação autenticada do próprio usuário na organização `ifesserra-lab` (ou orientar sobre a permissão `read:org` caso o token restrinja a leitura).
- **Convite pendente na organização `ifesserra-lab`**: Se o usuário tiver apenas um convite pendente (`state: pending`) e ainda não for membro ativo (`state: active`) da organização `ifesserra-lab`, o sistema deve recusar o acesso e informar que a participação na organização ainda está pendente de aceitação no GitHub.
- **Ausência de conexão com a internet no momento da verificação**: Como a comprovação de membresia na organização `ifesserra-lab` depende de consulta ao GitHub, se não houver conexão de rede no momento do login especial, o sistema deve exibir erro claro de conectividade e manter a aba `egressos-dados` bloqueada (sem afetar o uso offline do Horizon e do SRC).
- **Proteção contra vazamento do token**: O valor do Token do GitHub nunca deve ser exibido em texto aberto no campo de entrada (usar máscara de senha), nem aparecer em mensagens de erro, eventos ou registros de diagnóstico.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: O sistema MUST disponibilizar **`egressos-dados`** como um 4º card/projeto independente na página principal de seleção de projetos (`/projects`), ao lado de **Horizon**, **SRC** e **Egressos**, com sua própria área de trabalho e menu lateral exclusivos protegidos pelo controle de acesso da organização `ifesserra-lab`.
- **FR-002**: Ao acessar a área **`egressos-dados`** sem autorização ativa da organização, o sistema MUST apresentar uma interface de **login especial via Token do GitHub**, bloqueando a visualização de qualquer tabela ou dado dessa área até a conclusão da verificação.
- **FR-003**: O sistema MUST validar o Token do GitHub informado verificando simultaneamente:
  1. que o token autentica um usuário válido no GitHub; e
  2. que esse usuário possui vínculo de **membro ativo** (`active`) na organização **`ifesserra-lab`** do GitHub, consultando primeiro a filiação autenticada do usuário na organização e, caso o token não possua permissão de leitura de organização (`read:org`), verificando como fallback a listagem pública de membros de `ifesserra-lab` (orientando sobre a permissão `read:org` se não for encontrado publicamente).
- **FR-004**: O sistema MUST negar o acesso à aba **`egressos-dados`** caso o token seja inválido/expirado, caso a filiação à organização `ifesserra-lab` esteja pendente/inexistente, ou caso não seja possível confirmar a participação ativa na organização `ifesserra-lab`.
- **FR-005**: O sistema MUST fornecer mensagens de erro claras e distintas para cada cenário de falha no login especial:
  - token inválido ou expirado;
  - usuário válido no GitHub, porém **não pertencente** à organização `ifesserra-lab` (ou com convite ainda pendente);
  - token sem escopo/permissão suficiente para verificar a participação na organização `ifesserra-lab`;
  - falha de conexão de rede com o GitHub.
- **FR-006**: Toda a lógica de verificação do token e de comprovação de membresia na organização `ifesserra-lab` MUST ser executada na camada de negócio do aplicativo (e não apenas na interface visual), bloqueando também chamadas diretas aos comandos de dados da aba `egressos-dados` quando a sessão não estiver autorizada para a organização `ifesserra-lab`.
- **FR-007**: Após a aprovação no login especial, o sistema MUST desbloquear a aba **`egressos-dados`** e exibir seu painel estrutural próprio (com menu lateral exclusivo, cabeçalho com o `@login` do membro no GitHub, badge de confirmação da organização `ifesserra-lab`, ação de bloquear/desconectar token e área estrutural pronta aguardando a conexão dos JSONs e tabelas na próxima etapa).
- **FR-008**: O sistema MUST manter a autorização especial e o token de `egressos-dados` apenas em memória durante a sessão ativa (nunca gravando esse token em disco), limpando-os automaticamente ao encerrar a sessão principal (logout) ou fechar o aplicativo, além de permitir que o usuário encerre o login especial manualmente a qualquer momento dentro da aba.
- **FR-009**: O Token do GitHub mantido em memória na sessão especial de `egressos-dados` MUST ficar disponível exclusivamente para as futuras operações autenticadas no GitHub dessa aba durante a sessão ativa, de forma independente do token de sincronização salvo em disco pelos outros projetos, e o seu valor completo NUNCA deve ser exposto em mensagens de erro, logs ou retornos para a interface.
- **FR-010**: O funcionamento das demais áreas do aplicativo (**Horizon**, **SRC** e **Egressos**) MUST permanecer inalterado e acessível normalmente pelo login padrão do aplicativo, sem exigir que o curador dessas áreas pertença à organização `ifesserra-lab`.

### Key Entities *(include if feature involves data)*

- **Sessão de Autorização da Organização (`EgressosOrgAuth`)**: Estado de autenticação especial associado à aba `egressos-dados`, contendo o status de verificação (`autorizado` / `bloqueado`), o identificador do usuário no GitHub (`login`), a organização validada (`ifesserra-lab`) e o momento da validação.
- **Credencial de Token do GitHub (`GitHubToken`)**: Token de acesso pessoal fornecido pelo usuário para autenticar-se junto ao GitHub e comprovar participação ativa na organização `ifesserra-lab`, mantido de forma protegida e nunca exposto em texto claro após a entrada.
- **Aba `egressos-dados`**: Espaço de navegação e trabalho no Research Hub cujo conteúdo e tabelas só são renderizados e consultáveis quando há uma `EgressosOrgAuth` válida e ativa para a organização `ifesserra-lab`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% dos usuários com um Token do GitHub válido e participação ativa na organização `ifesserra-lab` conseguem desbloquear e acessar a aba `egressos-dados` em menos de 5 segundos (em conexão de internet padrão).
- **SC-002**: 100% das tentativas de acesso à aba `egressos-dados` com tokens inválidos ou com tokens válidos de contas fora da organização `ifesserra-lab` são bloqueadas tanto na interface quanto nas chamadas internas, exibindo o motivo exato da recusa.
- **SC-003**: 0 ocorrências de exposição do valor do Token do GitHub em mensagens de erro, logs ou elementos visíveis da tela em qualquer fluxo de sucesso ou falha.
- **SC-004**: O bloqueio da aba `egressos-dados` apresenta 0% de impacto ou regressão no acesso e uso offline das abas existentes dos projetos Horizon e SRC.

## Assumptions

- O nome canônico da organização no GitHub a ser validada é exatamente **`ifesserra-lab`** (correspondente a `https://github.com/orgs/ifesserra-lab/dashboard`).
- A verificação de participação na organização considera apenas membros com estado ativo (`active`) na organização `ifesserra-lab`; usuários externos sem vínculo ou com convite pendente não recebem acesso.
- Para que o GitHub confirme a participação de um membro (especialmente quando a filiação do usuário na organização é privada), o Token de Acesso Pessoal (Classic com escopo `read:org` ou `repo`, ou Fine-Grained com permissão de leitura da organização `ifesserra-lab`) é utilizado na consulta autenticada ao GitHub.
- Nesta feature, o escopo na aba `egressos-dados` concentra-se na criação da aba e na barreira de autenticação especial por organização (`ifesserra-lab`) via Token do GitHub; o repositório específico de onde serão lidos os arquivos JSON dessa aba e a construção das respectivas tabelas serão conectados na etapa seguinte, conforme indicado pelo usuário ("antes de passar o github dela para ler os jsons e criar as tabelas").
