# Feature Specification: Espaço de Dados do Projeto Egressos com CRUD e Sincronização GitHub

**Feature Branch**: `035-egressos-data-workspace`

**Created**: 2026-10-22

**Status**: Draft

**Input**: User description: "o researchhub possui compatibilidade com o horizon e o src , para a proxima feature , estaremos adicionando compatibilidade com o programa egressos que esta nesse computador , os jsons que estao para fazer as tabelas estao em /home/thiagodeps/egressos/dados , e uma pasta e nao tem json unificado , preciso que faça o sistema do research hub funcionar em uma pagina separa , como funciona o horizon e o src , alem disso , coloque o sistema de upload e baixar do github por meio de token , e os cruds tbm , por ultimo faz em uma nova branch essas mudanças , para n ocorrer problemas caso der erro"

## Contexto

O Research Hub atualmente oferece espaços de trabalho independentes para dois projetos: **Horizon** (15 tabelas canônicas do DataLake) e **SRC** (arquivo JSON consolidado único de ações e participações). Esta feature adiciona suporte ao terceiro projeto institucional: **Egressos** (painel executivo e de dados abertos sobre a trajetória profissional e de renda dos egressos do IFES Campus Serra).

Diferentemente do SRC — que concentra tudo em um único arquivo JSON — o projeto **Egressos** armazena seus dados em uma **pasta de 21 arquivos JSON independentes** (localizada em `/home/thiagodeps/egressos/dados`), sem um JSON unificado. Além disso, enquanto o SRC tem envio ao GitHub bloqueado por conter dados pessoais sensíveis (CPF), o conjunto de dados em `dados/` do projeto Egressos compõe o repositório e painel institucional do projeto e exige **tanto download quanto upload para o GitHub autenticados por token**, além de importação/exportação local da pasta e **CRUD completo** em todas as tabelas derivadas dos arquivos JSON.

## Clarifications

### Session 2026-10-22

- Q: Como os 21 arquivos JSON da pasta `dados/` — alguns contendo múltiplas listas e seções internas (como `vitrine.json` e `analise.json`) — devem ser organizados em abas e tabelas para o CRUD na página do Egressos? → A: **Opção A** — Uma aba para cada um dos 21 arquivos JSON, exibindo a coleção principal como tabela CRUD e permitindo alternar para sub-tabelas/metadados internos do mesmo arquivo quando houver mais de uma seção.
- Q: Quando o curador cria, edita ou exclui um registro em uma das 21 abas do Egressos, a alteração deve modificar apenas o arquivo JSON daquela aba (A) ou propagar vínculos básicos entre arquivos (B)? → A: **Escolha do curador na interface (A ou B)** — Por padrão a edição aplica-se ao arquivo da aba atual (A), mas a interface oferece ao curador a opção explícita de propagar campos compartilhados de uma mesma entidade (ex.: egresso pelo `rotulo`/`label` ou empresa pelo nome) para os demais arquivos relacionados da pasta `dados/` (B).
- Q: No sistema de download e upload do GitHub por meio de token para o projeto Egressos, como os 21 arquivos JSON devem ser lidos e gravados no repositório remoto? → A: **Opção A** — Sincronizar diretamente a pasta de arquivos `.json` no repositório (ler e commitar os 21 arquivos individuais dentro do diretório configurado, por padrão `dados/`, na branch informada).
- Q: Na importação e exportação local (no computador), o curador deve interagir diretamente com uma pasta contendo os 21 arquivos JSON (como `/home/thiagodeps/egressos/dados`) ou o sistema também deve aceitar pacotes `.zip`? → A: **Opção A** — Operar diretamente com pasta local (`dados/`), lendo e gravando os 21 arquivos `.json` diretamente na pasta selecionada (sugerindo `/home/thiagodeps/egressos/dados` por padrão quando presente).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Selecionar e acessar o espaço de trabalho exclusivo do projeto Egressos (Priority: P1)

Ao autenticar-se no Research Hub, o curador visualiza na página principal de seleção de projetos uma terceira opção claramente identificada — **Egressos** — ao lado de **Horizon** e **SRC**. Ao selecionar Egressos, o aplicativo abre uma página/área de trabalho dedicada exclusivamente aos dados do projeto Egressos, com suas próprias abas de tabelas, busca, contadores e controles de importação, exportação e sincronização. O curador pode alternar entre Horizon, SRC e Egressos a qualquer momento durante a sessão, sem reiniciar o aplicativo e sem que dados, abas ou snapshots de um projeto apareçam no outro.

**Why this priority**: É a base estrutural e de segurança para operar um terceiro projeto no Research Hub sem risco de misturar dados ou sobrescrever as bases do Horizon ou do SRC.

**Independent Test**: Autenticar no aplicativo, verificar as três opções na tela de seleção de projetos (Horizon, SRC e Egressos), entrar na área Egressos, verificar que apenas as abas do Egressos são exibidas e alternar entre os três projetos confirmando isolamento total.

**Acceptance Scenarios**:

1. **Given** o curador autenticado na página de seleção de projetos, **When** a tela é carregada, **Then** os três projetos (**Horizon**, **SRC** e **Egressos**) são apresentados de forma claramente distinguível.
2. **Given** o menu de seleção de projetos, **When** o curador escolhe **Egressos**, **Then** o aplicativo abre a página dedicada do Egressos exibindo exclusivamente as abas correspondentes aos arquivos JSON de `dados/` do Egressos, sem nenhuma tabela ou registro do Horizon ou do SRC.
3. **Given** o curador trabalhando na área do Egressos, **When** ele alterna para Horizon ou SRC pelo seletor de projeto, **Then** a transição ocorre sem reiniciar o aplicativo e exibe apenas os dados do projeto selecionado.
4. **Given** o curador com uma edição não salva aberta na área Egressos, **When** ele tenta trocar de projeto, **Then** o sistema solicita confirmação antes de descartar a edição em andamento.

---

### User Story 2 - Importar e exportar localmente a pasta de JSONs do Egressos (Priority: P2)

Na área de trabalho do Egressos, o curador importa diretamente a pasta local de dados do projeto contendo os 21 arquivos JSON (sugerindo `/home/thiagodeps/egressos/dados` por padrão quando presente no computador). O sistema valida todos os 21 arquivos JSON antes de alterar a base, cria um snapshot de segurança caso já exista base carregada, substitui integralmente os dados do Egressos (sem mesclar) e informa o total de arquivos e registros carregados. Ao exportar localmente, o sistema grava novamente todos os 21 arquivos JSON diretamente na pasta local selecionada, preservando fielmente a estrutura original de cada arquivo, seus registros, campos aninhados e metadados.

**Why this priority**: Como o Egressos não possui um JSON unificado, a capacidade de ler os 21 arquivos JSON da pasta `dados/` e regravá-los sem perda estrutural é pré-requisito para qualquer curadoria local ou sincronização com o GitHub.

**Independent Test**: Importar a pasta `/home/thiagodeps/egressos/dados` no espaço Egressos, verificar que os 21 conjuntos de dados foram carregados com suas contagens corretas, exportar para uma nova pasta sem realizar edições e comprovar que os 21 arquivos exportados são estrutural e semanticamente equivalentes aos originais.

**Acceptance Scenarios**:

1. **Given** uma pasta válida contendo os 21 arquivos JSON do Egressos (`/home/thiagodeps/egressos/dados`), **When** o curador realiza a importação, **Then** o sistema carrega todas as 21 tabelas, preserva os metadados de cada arquivo e exibe o resumo de arquivos e registros importados.
2. **Given** uma base Egressos já carregada no aplicativo, **When** o curador importa uma nova pasta `dados/`, **Then** o sistema cria e informa um snapshot de segurança exclusivo do Egressos antes de substituir a base anterior.
3. **Given** uma base Egressos carregada, **When** o curador aciona a exportação local, **Then** o sistema grava os 21 arquivos JSON individuais com os mesmos nomes de arquivo, chaves de topo, metadados e estrutura esperada pelo painel e pipeline do Egressos.
4. **Given** uma pasta selecionada na importação em que falta um arquivo JSON obrigatório ou há um arquivo com JSON malformado/incompatível, **When** o curador tenta importar, **Then** a operação é abortada com mensagem clara indicando exatamente qual arquivo falhou, mantendo a base anterior do Egressos intacta.

---

### User Story 3 - Executar CRUD completo nas tabelas do Egressos (Priority: P3)

Dentro da área de trabalho do Egressos, o curador navega entre as abas/tabelas correspondentes aos 21 arquivos JSON de `dados/` (incluindo perfis de egressos, vitrine, empresas e seus portes/aliases/LinkedIn, panorama do coorte, análise, impacto, trajetória, fomento FAPES, recortes de cursos, pesquisas salariais Código Fonte e Stack Overflow, salário mínimo, séries IBGE, renda por senioridade, cargos ao longo do tempo e mapa-múndi). Em cada tabela, o curador lista registros com busca e ordenação por colunas relevantes e executa todas as operações de **CRUD (criar, visualizar, editar e excluir)** sobre os registros, além de visualizar e editar metadados/seções auxiliares do respectivo arquivo JSON quando aplicável.

**Why this priority**: Entrega o valor central de curadoria de dados no Research Hub (Princípio IV da Constituição), permitindo manter os dados do painel de Egressos atualizados diretamente pela interface gráfica.

**Independent Test**: Selecionar diferentes abas na área Egressos (tanto tabelas baseadas em listas de objetos, como `egressos_perfil` e `vitrine`, quanto tabelas baseadas em mapas por chave, como `empresas_porte` e `empresas_aliases`), criar um novo registro, buscar e ordenar registros, editar um registro existente e excluir um registro com confirmação, verificando que todas as alterações persistem na base e se refletem na exportação dos respectivos arquivos JSON.

**Acceptance Scenarios**:

1. **Given** a base Egressos carregada, **When** o curador seleciona qualquer uma das 21 tabelas na navegação da área Egressos, **Then** a tabela exibe seus registros em colunas legíveis com suporte a busca textual e ordenação por coluna.
2. **Given** uma tabela do Egressos aberta, **When** o curador aciona a criação de um novo registro e preenche seus campos obrigatórios, **Then** o registro é salvo na base local, aparece na listagem da tabela e passa a compor o arquivo JSON correspondente na exportação.
3. **Given** um registro existente em qualquer tabela do Egressos, **When** o curador edita seus campos (incluindo listas ou objetos aninhados quando existentes no registro) e salva, **Then** os novos valores são persistidos imediatamente e refletidos no arquivo JSON correspondente ao exportar.
4. **Given** um registro existente em uma tabela do Egressos, **When** o curador solicita sua exclusão e confirma no diálogo de segurança, **Then** o registro é removido da base e não aparece mais na listagem nem no arquivo JSON exportado.
5. **Given** um arquivo JSON do Egressos que possui metadados ou blocos auxiliares no topo do arquivo (além da lista principal de registros), **When** o curador edita registros da tabela ou edita os metadados do arquivo, **Then** os metadados e blocos auxiliares são preservados sem perda na exportação.

---

### User Story 4 - Baixar e enviar os dados do Egressos para o GitHub via token (Priority: P4)

Na área de trabalho do Egressos, o curador sincroniza a pasta `dados/` diretamente com o repositório GitHub do projeto Egressos utilizando um token de acesso pessoal. O curador pode **baixar (download)** os 21 arquivos JSON diretamente da pasta configurada em uma branch do repositório GitHub (público ou privado), passando pelas mesmas validações e criação de snapshot da importação local. O curador também pode **enviar (upload)** o estado atual curado da base Egressos diretamente para a pasta configurada (por padrão `dados/`) na branch do repositório GitHub por meio do token, gerando os 21 arquivos JSON atualizados no momento do envio e registrando o commit na branch de destino.

**Why this priority**: Automatiza o ciclo de publicação e obtenção dos dados do repositório GitHub do Egressos sem exigir manipulação manual de pastas e comandos Git fora do aplicativo.

**Independent Test**: Configurar o token de acesso e o destino GitHub do projeto Egressos (repositório, branch e caminho da pasta `dados/`), executar o download a partir do repositório verificando a carga completa das 21 tabelas, realizar uma edição em uma tabela e executar o upload para a branch configurada, confirmando o commit gerado com os arquivos JSON atualizados.

**Acceptance Scenarios**:

1. **Given** um repositório GitHub (público ou privado com token válido) contendo a pasta `dados/` com os 21 arquivos JSON do Egressos, **When** o curador informa a origem/configuração do repositório e confirma o download na área Egressos, **Then** o sistema baixa todos os 21 arquivos JSON, valida seu conteúdo, cria snapshot da base anterior (se houver) e substitui a base local do Egressos exibindo as contagens carregadas.
2. **Given** uma base Egressos carregada e uma configuração de destino GitHub válida (repositório, branch existente e caminho da pasta como `dados/`) com token de permissão de escrita, **When** o curador confirma o upload, **Then** o aplicativo gera os 21 arquivos JSON a partir do estado atual da base e publica as alterações como um commit na branch informada, exibindo a confirmação e identificação do commit gerado.
3. **Given** uma tentativa de upload ou download em repositório privado sem token configurado ou com token sem permissão suficiente, **When** o curador confirma a operação, **Then** o sistema exibe um erro claro e acionável orientando sobre a configuração/permissão do token, sem alterar a base local nem gravar dados parciais no repositório remoto.
4. **Given** uma branch de destino que não existe no repositório GitHub informado, **When** o curador tenta realizar o upload da base Egressos, **Then** o sistema recusa a operação com mensagem clara informando que a branch não existe, sem criar branches automaticamente.
5. **Given** qualquer operação de sincronização com o GitHub (sucesso ou falha), **When** mensagens de progresso, confirmação ou erro são exibidas, **Then** o valor do token de acesso pessoal nunca é exposto.

---

### Edge Cases

- **Pasta `dados/` incompleta ou com arquivos extras**: Se a pasta local ou remota não contiver todos os 21 arquivos JSON canônicos do Egressos, a importação falha de forma atômica informando a lista exata de arquivos ausentes; arquivos estranhos (ex.: `.DS_Store`, `README.md`) são ignorados sem interromper a importação.
- **Arquivos JSON com diferentes estruturas raiz**: Os 21 arquivos em `/home/thiagodeps/egressos/dados` dividem-se entre objetos com listas principais + metadados (ex.: `egressos_perfil.json`, `vitrine.json`, `panorama.json`) e mapas indexados por chave textual (ex.: `empresas_porte.json`, `empresas_aliases.json`, `empresas_linkedin_data.json`, `empresas_linkedin_urls.json`). O sistema deve representar a chave de cada entrada de mapa como coluna identificadora editável na tabela e reconstruir o objeto indexado por chave na exportação.
- **Chaves duplicadas ao criar/editar registros em tabelas baseadas em mapa**: Caso o curador tente criar ou renomear a chave identificadora de um registro (ex.: nome da empresa em `empresas_porte`) para uma chave já existente na mesma tabela, o sistema bloqueia a operação com aviso claro evitando sobrescrita acidental.
- **Queda de conexão durante download ou upload de múltiplos arquivos no GitHub**: O download só substitui a base local após todos os 21 arquivos JSON terem sido baixados e validados integralmente; o upload publica o conjunto completo em um único commit atômico na branch (ou falha sem deixar arquivos parcialmente atualizados no destino).
- **Preservação de acentuação, ordenação de chaves e tipos numéricos/booleanos**: Valores nulos, números decimais, booleanos e textos com caracteres especiais/UTF-8 são preservados integralmente após importação, edição e exportação/upload.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: O sistema MUST incluir o projeto **Egressos** na página principal de seleção de projetos após o login, ao lado de **Horizon** e **SRC**, permitindo alternar entre os três projetos durante a sessão sem reiniciar o aplicativo.
- **FR-002**: O sistema MUST manter isolamento completo de dados, abas, buscas, contagens, snapshots e configurações de sincronização entre **Horizon**, **SRC** e **Egressos**.
- **FR-003**: O sistema MUST manter o funcionamento das áreas **Horizon** e **SRC** inalterado pela adição do projeto **Egressos**.
- **FR-004**: O sistema MUST permitir importar localmente a base de dados do Egressos diretamente a partir de uma pasta local contendo os 21 arquivos `.json` do projeto (sugerindo `/home/thiagodeps/egressos/dados` como caminho padrão quando presente no computador).
- **FR-005**: O sistema MUST validar a presença e a estrutura JSON dos 21 arquivos do Egressos antes de efetivar qualquer importação (local ou via GitHub), abortando a operação de forma atômica com mensagem clara caso qualquer arquivo obrigatório esteja ausente ou inválido.
- **FR-006**: O sistema MUST criar automaticamente um snapshot de segurança exclusivo do projeto Egressos antes de substituir uma base Egressos já existente durante a importação local ou download do GitHub.
- **FR-007**: O sistema MUST disponibilizar na área de trabalho do Egressos navegação de 1 para 1 com **21 abas principais**, correspondentes aos 21 arquivos JSON da pasta `dados/`: `analise`, `cargos_ao_longo_do_tempo`, `codigofonte_2026`, `codigofonte_historico`, `consolidado`, `egressos_perfil`, `empresas_aliases`, `empresas_linkedin_data`, `empresas_linkedin_urls`, `empresas_porte`, `fapes_fomento`, `ibge_series`, `impacto`, `mapa_mundi`, `panorama`, `recorte_cursos`, `renda_por_senioridade`, `salario_minimo`, `so_benchmarks`, `trajetoria` e `vitrine`.
- **FR-008**: O sistema MUST exibir, ao abrir cada uma das 21 abas do Egressos, a coleção principal do arquivo como tabela com busca textual, ordenação por colunas e **CRUD completo** (criar, visualizar, editar e excluir registros com confirmação de exclusão), e oferecer seletores/sub-abas internos na mesma tela para alternar entre sub-tabelas ou seções secundárias quando o arquivo JSON contiver múltiplas listas.
- **FR-008a**: Ao editar ou excluir registros que possuam chave de entidade compartilhada entre múltiplos arquivos de `dados/` (como o `rotulo`/`label` do egresso entre `egressos_perfil`, `vitrine`, `panorama`, `analise` e `consolidado`, ou o nome da empresa entre `empresas_porte`, `empresas_aliases`, `empresas_linkedin_data` e `empresas_linkedin_urls`), o sistema MUST permitir que o curador escolha entre aplicar a alteração apenas na tabela/arquivo atual ou propagar a atualização dos campos equivalentes para as tabelas relacionadas.
- **FR-009**: O sistema MUST preservar e permitir visualizar/editar os metadados de nível de arquivo e sub-tabelas presentes nos arquivos JSON do Egressos dentro da própria aba do arquivo, garantindo que nenhum campo fora da coleção principal seja perdido ao exportar ou enviar ao GitHub.
- **FR-010**: O sistema MUST exportar localmente a base do Egressos gravando diretamente os 21 arquivos `.json` individuais na pasta local escolhida pelo curador, mantendo os nomes exatos dos arquivos e a estrutura JSON compatível com o painel e o pipeline do projeto Egressos.
- **FR-011**: O sistema MUST permitir baixar (download) diretamente os 21 arquivos `.json` da pasta configurada (por padrão `dados/`) em um repositório e branch do GitHub (público ou privado autenticado por token), aplicando as mesmas validações atômicas e criação de snapshot da importação local.
- **FR-012**: O sistema MUST permitir enviar (upload) a base atual do Egressos diretamente para a pasta configurada (por padrão `dados/`) em um repositório e branch existente no GitHub usando autenticação por token pessoal, gerando os 21 arquivos `.json` individuais no momento do envio e registrando-os em um único commit atômico na branch de destino.
- **FR-013**: O sistema MUST compartilhar o armazenamento local seguro do token de acesso do GitHub (com opções de salvar, testar e remover sem jamais exibi-lo em logs ou mensagens de erro), mantendo configurações de repositório, branch e caminho de diretório independentes para o projeto Egressos.

### Key Entities *(include if feature involves data)*

- **Projeto Egressos (Contexto de Trabalho)**: Terceiro espaço de dados independente do Research Hub, dedicado ao conjunto de 21 arquivos JSON do estudo de Egressos do IFES Campus Serra, com banco local, snapshots e configuração de sincronização GitHub próprios.
- **Catálogo de Arquivos/Tabelas do Egressos (`dados/*.json`)**: Conjunto canônico dos 21 arquivos JSON que compõem a base do Egressos, divididos em:
  - *Tabelas centradas em coleção de registros com metadados de arquivo*: `egressos_perfil`, `vitrine`, `panorama`, `consolidado`, `analise`, `trajetoria`, `impacto`, `fapes_fomento`, `renda_por_senioridade`, `cargos_ao_longo_do_tempo`, `codigofonte_2026`, `salario_minimo`, `so_benchmarks`, `mapa_mundi`.
  - *Tabelas indexadas por chave (dicionários de entidades/séries)*: `empresas_porte`, `empresas_aliases`, `empresas_linkedin_data`, `empresas_linkedin_urls`, `codigofonte_historico`, `ibge_series`, `recorte_cursos`.
- **Registro de Tabela do Egressos**: Linha individual editável via CRUD dentro de uma das 21 tabelas do Egressos, identificada por seu código/rótulo/chave canônica e contendo atributos escalares e estruturas aninhadas do respectivo arquivo JSON.
- **Envelope de Metadados de Arquivo JSON**: Atributos de cabeçalho, notas metodológicas e objetos auxiliares de cada um dos 21 arquivos JSON que acompanham os registros principais e são reconstruídos fielmente na exportação e no upload para o GitHub.
- **Configuração de Sincronização GitHub do Egressos**: Parâmetros locais do projeto Egressos contendo repositório (`owner/repo`), branch e diretório alvo (`dados/`) para download e upload atômico dos 21 arquivos JSON via token pessoal.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A partir do login, o curador seleciona o projeto **Egressos** em 1 clique e alterna entre **Horizon**, **SRC** e **Egressos** em menos de 5 segundos, com 0% de vazamento de registros, abas ou snapshots entre os projetos.
- **SC-002**: A importação local da pasta `/home/thiagodeps/egressos/dados` carrega 100% dos 21 arquivos JSON em menos de 10 segundos, disponibilizando todas as 21 tabelas para navegação e consulta imediata.
- **SC-003**: Exportar localmente uma base Egressos recém-importada (sem edições) gera 21 arquivos JSON cujo conteúdo é 100% equivalente semanticamente (mesmas chaves, listas, tipos de dados e valores) aos arquivos originais de `/home/thiagodeps/egressos/dados`.
- **SC-004**: O curador consegue realizar o ciclo completo de CRUD (criar, visualizar, editar e excluir um registro) em qualquer uma das 21 tabelas do Egressos em menos de 2 minutos, vendo a alteração refletida tanto na listagem quanto no respectivo arquivo JSON exportado.
- **SC-005**: O download e o upload dos 21 arquivos JSON da pasta `dados/` contra um repositório GitHub (usando token de acesso) concluem em menos de 30 segundos em conexão padrão, gerando um único commit no upload e mantendo a base local intacta em 100% dos cenários de falha de rede ou autenticação.

## Assumptions

- O conjunto canônico do projeto Egressos é composto exatamente pelos 21 arquivos `.json` presentes em `/home/thiagodeps/egressos/dados`. O Research Hub opera diretamente sobre esses 21 arquivos (não sobre a pasta `api/` ou os arquivos `.html` derivados do repositório Egressos).
- Diferentemente do projeto SRC (cujo envio ao GitHub foi bloqueado por conter CPF e dados pessoais restritos), o envio (upload) para o GitHub no projeto Egressos **está habilitado**, conforme solicitado explicitamente pelo usuário e compatível com a natureza do repositório do painel Egressos (`thiagodeps/egressos`).
- Como os dados do Egressos residem em uma pasta com 21 arquivos JSON e não em um arquivo único, a sincronização com o GitHub no espaço Egressos opera sobre o diretório de dados do repositório (por padrão `dados/` na branch configurada), lendo os 21 arquivos no download e publicando os 21 arquivos atualizados em um único commit no upload.
- Para arquivos JSON que possuem múltiplos blocos/coleções ou metadados no nível raiz (ex.: `vitrine.json`, `analise.json`, `consolidado.json`), a aba da tabela apresenta os registros da coleção principal para CRUD direto e permite inspecionar/editar as seções secundárias e metadados do arquivo, garantindo fidelidade total (*round-trip*) na exportação.
- O trabalho desta feature é desenvolvido em uma branch Git isolada (`035-egressos-data-workspace`), criada antes da especificação, preservando a estabilidade da branch principal.
