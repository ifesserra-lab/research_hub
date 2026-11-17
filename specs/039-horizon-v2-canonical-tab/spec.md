# Feature Specification: Aba Horizon (Nova Versão Canônica) e Preservação do Horizon Legacy

**Feature Branch**: `039-horizon-v2-canonical-tab`

**Created**: 2026-11-12

**Status**: Draft

**Input**: User description: "uma nova versao do horizomn foi lançada , e nela tem um novo exports canonical , com mudanças nas tabelas em relaçao ao ultimo , e outras mudanças ainda vao ser implementadas , renomeie a versao da aba do reshearch hub d do horizon como legacy, sendo ainda funcional como esta , e crie um nova aba copiando e adaptando o que precisa ser adaptado para uma nova aba do horizon , alem disso analise as tabelas e me fale o que mudlou, o novo export canonical ja esta nessa pasta"

## Contexto e Análise do Novo Pacote (`exports_canonical.zip`)

A comparação direta entre o pacote `exports_canonical.zip` da versão anterior do Horizon (395 entradas no total) e o novo `exports_canonical.zip` lançado (698 entradas no total, +303 arquivos) revelou as seguintes mudanças:

### 1. Resumo das 15 Tabelas Canônicas de Curadoria

Todas as 15 entidades gerenciadas pelo painel de curadoria continuam presentes na raiz do arquivo `.zip`, mantendo os mesmos nomes de colunas, porém com **expansão expressiva de volume** (de **28.102** para **70.615** registros no total — crescimento de **+151%**) e **mudança estrutural de conteúdo na tabela `initiatives`**:

| Tabela (`*_canonical.json`) | Linhas (Anterior) | Linhas (Novo) | Diferença | Colunas | Mudanças de Estrutura / Conteúdo |
|---|---|---|---|---|---|
| `researchers` | 9.694 | 18.429 | +8.735 | 21 | Mesmas colunas; expansão de base |
| `students` | 6.282 | 11.799 | +5.517 | 21 | Mesmas colunas; expansão de base |
| `initiatives` | 4.044 | 15.505 | +11.461 | 19 | **Coluna `enrichment` passou de 100% nula para objeto estruturado em 267 registros** (dados extraídos do SIGPesq); `start_date` agora admite `null` além de texto |
| `articles` | 2.133 | 8.057 | +5.924 | 9 | Mesmas colunas; expansão de base |
| `professional_activities` | 2.099 | 9.557 | +7.458 | 17 | Mesmas colunas; expansão de base |
| `research_productions` | 975 | 3.729 | +2.754 | 13 | Mesmas colunas; expansão de base |
| `knowledge_areas` | 1.552 | 1.563 | +11 | 3 | Mesmas colunas |
| `proficiencies` | 220 | 794 | +574 | 8 | Mesmas colunas; expansão de base |
| `organizations` | 139 | 440 | +301 | 5 | Mesmas colunas; expansão de base |
| `awards` | 52 | 183 | +131 | 5 | Mesmas colunas; expansão de base |
| `advisorships` | 188 | 190 | +2 | 9 | Mesmas colunas |
| `fellowships` | 19 | 21 | +2 | 5 | Mesmas colunas |
| `languages` | 8 | 11 | +3 | 3 | Mesmas colunas |
| `research_groups` | 353 | 353 | 0 | 13 | Inalterada em esquema e contagem |
| `campuses` | 23 | 23 | 0 | 6 | Inalterada em esquema e contagem |

### 2. Detalhamento da Mudança em `initiatives_canonical.json` (`enrichment`)

Na versão anterior, a coluna `enrichment` de `initiatives_canonical.json` possuía apenas valores `null` em todas as 4.044 linhas. No novo pacote, 267 iniciativas possuem um objeto estruturado com 15 chaves provenientes do cruzamento com arquivos de projetos do SIGPesq:
- `source`, `source_file`, `project_code`, `match_strategy`, `origin`
- `extracted_at`, `extraction_model`
- `needs_review`, `reviewed_at`, `reviewed_by`
- `area_conhecimento`, `linha_pesquisa`, `palavras_chave`, `objetivos` (`geral`, `especificos`), `cronograma`

### 3. Novos Arquivos Canônicos e Metadados na Raiz do Pacote (+6 arquivos)

O novo pacote introduz **3 novos arquivos `*_canonical.json`** cujo formato raiz é um **objeto JSON com metadados de versão (`rule_version: 5`, `generated_at`) envolvendo a lista de registros**, diferentemente das demais tabelas que são listas JSON puras na raiz:
1. **`horizon_ids_canonical.json`** (novo): objeto `{ rule_version, generated_at, ids: [...] }` com **75.804 registros** de identificadores determinísticos (`entity_type`, `id`, `horizon_id`, `rule`, `natural_key`, `parent_horizon_id`).
2. **`horizon_id_aliases_canonical.json`** (novo): objeto `{ rule_version, generated_at, aliases: [...] }` com **9.545 registros** de histórico/aposentadoria de IDs (`alias_horizon_id`, `horizon_id`, `reason`, `created_at`).
3. **`source_document_states_canonical.json`** (novo): objeto `{ rule_version, generated_at, documents: [...] }` com **2.377 registros** de controle de estado de documentos ingeridos (`source_system`, `document_key`, `status`, `last_seen_at`, `processed_at`, `downloaded_at`).
4. **`export_manifest.json`** (novo): manifesto de autenticidade do pacote (`{"key_fingerprint": "c833fb942765fd0c", "official": true}`).
5. **`initiatives_analytics_mart.json`** (novo): mart analítico consolidado de iniciativas (`campus`, `campuses`, `summary`, `evolution`, `team_composition`, `knowledge_areas`).
6. **`knowledge_areas_mart.json`** (novo): mart de áreas de conhecimento com **1.503 registros** (`area_id`, `area_name`, `groups_count`, `groups`, `campuses`, `campus`).

### 4. Novo Diretório `project_sigpesq_files_json/` (+297 arquivos)

- Contém **297 arquivos JSON individuais** (`PJ_*.json`) extraídos de projetos cadastrados no SIGPesq (`codigo`, `titulo`, `descricao`, `palavras_chave`, `area_conhecimento`, `linha_pesquisa`, `objetivos`, `coordenador`, `equipe`, `datas`, `cronograma`, `financiamento`, `bolsas`, `_meta`).

### 5. Mudanças nas Tabelas Auxiliares de Rastreabilidade e Linhagem

- **`attribute_assertions_canonical.json`**: saltou de 99.720 para **249.102 linhas** (+149.382); `value_json` passou a armazenar também objetos estruturados do enriquecimento SIGPesq.
- **`source_records_canonical.json`**: saltou de 19.404 para **40.255 linhas** (+20.851); `raw_payload_json` passou a incluir payloads brutos dos arquivos `PJ_*.json` do SIGPesq.
- **`entity_matches_canonical.json`**: saltou de 19.235 para **40.088 linhas** (+20.853).
- **`entity_change_logs_canonical.json`**: saltou de 20.226 para **38.350 linhas** (+18.124); `source_record_id` e `after_json` agora admitem `null`, e `before_json`/`after_json` registram alterações de `enrichment`, `parent_id` e campos individuais de artigos e iniciativas.
- **`null_researchers_canonical.json`**: 204 → **713 linhas** (`knowledge_areas` agora vazio `[]`).
- **`outside_ifes_canonical.json`**: 691 → **3.338 linhas**.
- **`researchers_only_canonical.json`**: 2.517 → **2.579 linhas**.
- **`production_authors_canonical.json`**: 985 → **3.850 linhas**.
- **`ingestion_runs_canonical.json`**: 7 → **16 linhas**.
- **`advisorships_tracking.json`**: 3.141 → **11.876 linhas**.
- **`initiatives_tracking.json`**: 854 → **3.620 linhas** (`attributes` agora inclui `enrichment`).
- **`researchers_tracking.json`**: 6.661 → **6.799 linhas**.

---

## Clarifications

### Session 2026-11-12

- Q: Quais tabelas do novo `exports_canonical.zip` devem aparecer como abas no menu lateral da nova área Horizon? → A: Manter as 15 abas de curadoria atuais adaptadas ao novo pacote (incluindo `enrichment` em Iniciativas e todos os campos das 15 tabelas), preservando os demais arquivos novos intactos no export.
- Q: Como o novo campo estruturado `enrichment` da tabela de Iniciativas (`initiatives`) deve se comportar no formulário da nova aba Horizon? → A: Visualização estruturada somente-leitura no formulário, preservando o objeto `enrichment` original intacto ao salvar e exportar.
- Q: O que deve acontecer com os dados locais e configurações de sincronização que o usuário já possui salvos na versão atual do aplicativo quando a aba original for renomeada para Horizon Legacy? → A: Manter os dados, snapshots e configurações existentes vinculados ao Horizon Legacy, e iniciar a nova aba Horizon com armazenamento e configuração próprios e isolados.

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Renomear a Aba Atual para "Horizon Legacy" Mantendo Funcionamento Integral (Priority: P1)

O administrador acessa a tela de seleção de projetos e encontra a versão anterior do ambiente identificada claramente como **Horizon Legacy**, podendo abrir seu painel, consultar, criar, editar, excluir, fundir e vincular registros nas 15 tabelas legadas, importar e exportar pacotes e sincronizar com o repositório configurado sem nenhuma regressão ou perda de funcionalidade.

**Why this priority**: Garante que o fluxo de trabalho existente continue 100% operacional enquanto a nova versão do Horizon evolui em paralelo.

**Independent Test**: Acessar a seleção de projetos, verificar que o cartão anteriormente chamado "Horizon" agora exibe "Horizon Legacy", abrir seu painel e executar o ciclo completo de visualização, edição, importação e exportação sem afetar outros projetos.

**Acceptance Scenarios**:

1. **Given** que o administrador está autenticado na tela de seleção de projetos, **When** visualiza os projetos disponíveis, **Then** vê o projeto renomeado para "Horizon Legacy" ao lado do novo projeto "Horizon" e dos demais projetos existentes.
2. **Given** que o administrador seleciona "Horizon Legacy", **When** navega pelo menu lateral e interage com qualquer uma das 15 entidades, **Then** todas as operações de listagem, busca, ordenação, criação, edição, exclusão, fusão, vínculo, importação, exportação e sincronização funcionam exatamente como antes.
3. **Given** que o administrador possui dados carregados no "Horizon Legacy", **When** importa ou edita dados na nova aba "Horizon", **Then** os dados, arquivos originais de referência, backups automáticos e configurações de sincronização do "Horizon Legacy" permanecem completamente isolados e inalterados.

---

### User Story 2 - Nova Aba "Horizon" Isolada e Adaptada ao Novo `exports_canonical.zip` (Priority: P1)

O administrador seleciona o novo projeto **Horizon** na tela de seleção de projetos e acessa um ambiente de curadoria próprio e independente, adaptado para importar, curar, exportar e sincronizar o novo pacote `exports_canonical.zip` (698 entradas, 70.615 registros nas 15 tabelas canônicas, novos arquivos canônicos de IDs/estados e enriquecimento de iniciativas via SIGPesq).

**Why this priority**: Entrega o novo espaço de trabalho dedicado à nova geração do Horizon, permitindo curar o novo pacote canônico e receber futuras adaptações sem risco de quebrar o ambiente legado.

**Independent Test**: Abrir o novo projeto "Horizon" na seleção de projetos, importar o novo `exports_canonical.zip`, verificar a carga das 15 tabelas canônicas com todas as 70.615 linhas, visualizar os campos atualizados (incluindo o enriquecimento de iniciativas) e exportar o pacote preservando todas as 683 entradas complementares intactas.

**Acceptance Scenarios**:

1. **Given** que o administrador seleciona o novo projeto "Horizon", **When** importa o novo `exports_canonical.zip`, **Then** o sistema carrega as 15 tabelas canônicas no banco isolado do novo Horizon (totalizando 70.615 registros no pacote de referência), cria snapshot prévio próprio caso haja dados anteriores e preserva o arquivo original de referência próprio do projeto.
2. **Given** o novo pacote carregado na aba "Horizon", **When** o administrador acessa a aba de Iniciativas (`initiatives`) e abre um registro enriquecido pelo SIGPesq, **Then** visualiza os dados estruturados do campo `enrichment` (código do projeto, objetivos, linha de pesquisa, cronograma, palavras-chave e status de revisão) além de todos os demais atributos canônicos da entidade.
3. **Given** o novo ambiente "Horizon", **When** o administrador cria, edita, exclui, funde ou vincula registros em qualquer uma das 15 tabelas canônicas, **Then** as alterações são salvas exclusivamente nas tabelas do novo Horizon, sem interferir na base do "Horizon Legacy".
4. **Given** uma base carregada no novo "Horizon" a partir do novo `exports_canonical.zip`, **When** o administrador exporta o pacote `.zip`, **Then** o arquivo gerado contém os 15 arquivos `{tabela}_canonical.json` com seus tipos originais restaurados (incluindo objetos estruturados em `enrichment`) e preserva byte a byte todas as 683 entradas não gerenciadas (incluindo `export_manifest.json`, `horizon_ids_canonical.json`, `horizon_id_aliases_canonical.json`, `source_document_states_canonical.json`, marts e os 297 arquivos de `project_sigpesq_files_json/`).
5. **Given** o painel de sincronização na nova aba "Horizon", **When** o administrador configura o repositório/branch/caminho ou realiza download e upload, **Then** a configuração é persistida de forma independente da configuração do "Horizon Legacy".

---

### User Story 3 - Visibilidade Completa dos Campos Canônicos na Nova Aba "Horizon" (Priority: P2)

Ao abrir o formulário de visualização/edição de uma entidade na nova aba **Horizon**, o curador tem acesso a todos os campos existentes na tabela canônica correspondente (como `enrichment`, `status`, `description`, `start_date`, `end_date`, `demandante`, `research_group`, `external_partner` e `external_research_group` em Iniciativas; `members` e `leaders` em Grupos de Pesquisa; `doi`, `year`, `type`, `journal_conference`, `volume` e `pages` em Artigos), enquanto o "Horizon Legacy" mantém os formulários originais inalterados.

**Why this priority**: No painel legado, diversas colunas presentes nos arquivos canônicos não apareciam nos formulários de visualização/edição; na nova aba Horizon, expor esses atributos (especialmente `enrichment` recém-populado em `initiatives`) permite curadoria completa e prepara a aba para as próximas evoluções do Horizon.

**Independent Test**: Abrir um registro de cada uma das 15 entidades na nova aba "Horizon" e verificar que os campos presentes na tabela canônica (texto, números e estruturas de vínculos/enriquecimento somente-leitura) são exibidos corretamente no formulário.

**Acceptance Scenarios**:

1. **Given** um registro na nova aba "Horizon", **When** o curador clica em "Visualizar", **Then** o formulário exibe os campos escalares editáveis e os campos estruturados (vínculos e `enrichment`) em visualização estruturada legível.
2. **Given** um vínculo clicável dentro do formulário da nova aba "Horizon", **When** o curador clica no vínculo para navegar até a entidade relacionada, **Then** é direcionado para a página correspondente dentro da própria aba "Horizon" (e nunca enviado acidentalmente para o "Horizon Legacy").

---

### Edge Cases

- **Isolamento entre Horizon e Horizon Legacy**: Importar um pacote no novo "Horizon" nunca deve apagar as tabelas nem sobrescrever o arquivo `original.zip` ou os snapshots do "Horizon Legacy", e vice-versa.
- **Novos arquivos `*_canonical.json` em formato de dicionário (`horizon_ids_canonical.json`, `horizon_id_aliases_canonical.json`, `source_document_states_canonical.json`)**: Embora terminem com o sufixo `_canonical.json`, sua raiz é um objeto JSON (`{"rule_version": 5, ...}`) e não uma tabela gerenciada pelas 15 abas de curadoria; a importação e a exportação devem preservá-los intactos byte a byte sem tentar interpretá-los como listas de registros nem falhar na inferência de tipos.
- **Coluna `enrichment` nula vs. objeto estruturado em `initiatives`**: Registros sem enriquecimento têm `enrichment: null`, enquanto registros enriquecidos têm um objeto JSON; o exportador deve inferir o tipo estruturado a partir do primeiro registro não nulo e emitir objeto JSON real para os enriquecidos e `null` para os demais.
- **Navegação de vínculos (deep-linking `openId`)**: Clicar em um vínculo dentro do "Horizon Legacy" deve abrir o registro correspondente no "Horizon Legacy"; clicar em um vínculo dentro do novo "Horizon" deve abrir o registro correspondente no novo "Horizon".

## Requirements *(mandatory)*

### Functional Requirements

**Renomeação e Preservação do Horizon Legacy**

- **FR-001**: O sistema DEVE renomear a opção existente do projeto Horizon na tela de seleção de projetos para **Horizon Legacy**, mantendo seu comportamento, páginas, formulários e operações de curadoria integralmente funcionais como estão.
- **FR-002**: O ambiente **Horizon Legacy** DEVE manter vinculados a si todos os dados locais, tabelas preexistentes, arquivo original de exportação, snapshots de segurança e configuração de sincronização com repositório remoto já existentes no aplicativo, completamente isolados da nova aba **Horizon** e dos demais projetos.

**Criação e Isolamento da Nova Aba Horizon**

- **FR-003**: O sistema DEVE disponibilizar um novo projeto chamado **Horizon** na tela de seleção de projetos, com navegação própria para suas 15 abas de entidades canônicas e painel de controle/sincronização dedicado.
- **FR-004**: O novo projeto **Horizon** DEVE armazenar seus registros em tabelas próprias e isoladas no banco local, utilizar um arquivo de referência original próprio e gerar snapshots de segurança com prefixo exclusivo, garantindo que operações no novo Horizon jamais alterem ou apaguem dados do Horizon Legacy (e vice-versa).
- **FR-005**: O novo projeto **Horizon** DEVE possuir chave de configuração própria para sincronização com repositório remoto (download, verificação de destino, listagem de branches e envio), independente da configuração do Horizon Legacy.
- **FR-006**: O novo projeto **Horizon** DEVE suportar todas as operações diretas de curadoria sobre as 15 entidades canônicas: listagem paginada, busca textual, ordenação (por valor ou por quantidade de vínculos), criação, edição, exclusão, fusão de registros duplicados e criação de vínculos entre entidades.

**Adaptação ao Novo `exports_canonical.zip`**

- **FR-007**: A importação e a exportação do novo **Horizon** DEVEM processar corretamente o novo `exports_canonical.zip` (70.615 registros distribuídos nas 15 tabelas canônicas gerenciadas e 683 entradas complementares não gerenciadas, totalizando 698 entradas no arquivo).
- **FR-008**: O sistema DEVE preservar byte a byte no export todas as entradas não gerenciadas do novo `exports_canonical.zip`, incluindo os novos arquivos `export_manifest.json`, `horizon_ids_canonical.json`, `horizon_id_aliases_canonical.json`, `source_document_states_canonical.json`, `initiatives_analytics_mart.json`, `knowledge_areas_mart.json` e todos os arquivos do diretório `project_sigpesq_files_json/`.
- **FR-009**: Na nova aba **Horizon**, os formulários de visualização e edição das entidades DEVEM expor todos os campos canônicos da respectiva tabela, exibindo o campo estruturado `enrichment` em Iniciativas (`initiatives`) e os demais objetos/listas de vínculos em modo de visualização estruturada somente-leitura (preservando seu conteúdo original intacto ao salvar e exportar).
- **FR-010**: A navegação por links de vínculos (deep-linking por identificador) dentro da nova aba **Horizon** DEVE direcionar o usuário para a rota correspondente dentro do próprio ambiente **Horizon**, enquanto links no **Horizon Legacy** DEVEM permanecer no ambiente **Horizon Legacy**.
- **FR-011**: Os testes automatizados de referência do pacote canônico DEVEM validar o novo `exports_canonical.zip` presente no repositório (contagens atualizadas das 15 tabelas, 698 entradas totais, 683 entradas preservadas e fidelidade de tipos no round-trip incluindo `enrichment`).

### Key Entities *(include if feature involves data)*

- **Projeto Horizon Legacy**: Ambiente de curadoria original preservado sem alterações de interface ou regras, operando sobre sua própria base e arquivos de referência.
- **Projeto Horizon (Novo)**: Novo ambiente de curadoria independente adaptado para a nova versão do `exports_canonical.zip` e preparado para evoluções futuras das tabelas do Horizon.
- **Novo Pacote Canônico (`exports_canonical.zip`)**: Pacote com 698 arquivos contendo as 15 tabelas canônicas de curadoria (70.615 linhas no total), 11 tabelas canônicas auxiliares em lista, 3 arquivos canônicos em formato de objeto com metadados (`horizon_ids`, `horizon_id_aliases`, `source_document_states`), manifesto de exportação (`export_manifest.json`), marts analíticos e 297 arquivos de projetos SIGPesq (`project_sigpesq_files_json/PJ_*.json`).
- **Enriquecimento de Iniciativa (`initiatives.enrichment`)**: Objeto estruturado associado a iniciativas identificadas nos arquivos de projetos do SIGPesq, contendo código do projeto, estratégia de vínculo, objetivos geral e específicos, cronograma, área de conhecimento, linha de pesquisa, palavras-chave e metadados de revisão.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: O curador consegue alternar entre **Horizon** e **Horizon Legacy** na seleção de projetos e importar pacotes diferentes em cada um com **0% de interferência** (nenhum registro, arquivo original ou configuração de sincronização de um ambiente é sobrescrito pelo outro).
- **SC-002**: A importação do novo `exports_canonical.zip` na aba **Horizon** carrega exatamente as 15 tabelas canônicas com **70.615 registros totais** (100% de correspondência de linhas por tabela com os JSONs do pacote).
- **SC-003**: O ciclo completo de importar e exportar (round-trip) o novo `exports_canonical.zip` na aba **Horizon** produz um pacote `.zip` com **698 entradas** (15 JSONs canônicos regenerados com tipos e objetos `enrichment` preservados + **683 entradas complementares preservadas byte a byte**).
- **SC-004**: Na nova aba **Horizon**, 100% das 15 entidades exibem em seus formulários todos os campos canônicos da tabela (incluindo o objeto `enrichment` nas 267 iniciativas enriquecidas), e 100% dos links de vínculos navegam dentro do próprio ambiente em que o usuário está.
- **SC-005**: 100% dos testes automatizados existentes do **Horizon Legacy** e os novos testes automatizados do **Horizon** passam sem regressões.

## Assumptions

- As 15 tabelas de curadoria ativa no novo **Horizon** permanecem sendo as mesmas 15 entidades de domínio (`researchers`, `students`, `articles`, `research_groups`, `initiatives`, `advisorships`, `awards`, `campuses`, `organizations`, `fellowships`, `proficiencies`, `professional_activities`, `knowledge_areas`, `languages`, `research_productions`), enquanto os novos arquivos de controle de pipeline (`horizon_ids_canonical.json`, `horizon_id_aliases_canonical.json`, `source_document_states_canonical.json`, `export_manifest.json`, marts e `project_sigpesq_files_json/`) são preservados integralmente no pacote durante o ciclo de importação e exportação, prontos para futuras expansões.
- O ambiente **Horizon Legacy** preserva todas as telas e comportamentos originais sem alterações, enquanto o novo **Horizon** opera em um espaço de navegação e armazenamento separado para permitir evoluções independentes.
- Campos estruturados complexos (como `enrichment` em iniciativas, `members` e `leaders` em grupos de pesquisa) são exibidos no formulário em modo de consulta estruturada (somente-leitura), mantendo o padrão de integridade relacional do aplicativo, enquanto a edição de vínculos entre entidades é realizada pelas operações dedicadas de vínculo e fusão.
