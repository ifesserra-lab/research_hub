# Feature Specification: Adaptação Final do Pacote Canônico Horizon, Indicador de Status de Projetos e Dashboards Analíticos em Todas as Abas

**Feature Branch**: `040-horizon-analytics-dashboard`

**Created**: 2026-11-16

**Status**: Draft

**Input**: User description: "proximas features serao as seguintes : a versao final nova do exports canonical esta na pasta ja , entao pode fazer as adptaçoes necessarias na aba do reserach hub . alem disso coloque indicativo se um projeto esta ativo ou n , por ultimo coloque a opçao de fazer dashboards com graficos das tabelas com varias informaçoes como "quantos projetos estao ativos " , quantidade de pesquisadores projetos e outras tabelas , quanto recebe os pesquisadores , de quais instituiçoes vem , entre outras medidas que vc ia pode descidir que e interresante. o ponto 3 (dashboard) e para todas as abas, com cada uma podendo criar graficos e informaçoes que condizem com o que elas trabalham"

## Contexto e Análise da Versão Final do `exports_canonical.zip`

A versão final do pacote `exports_canonical.zip` presente na raiz do repositório contém **698 entradas no total** (15 tabelas canônicas de curadoria e 683 arquivos complementares de rastreabilidade, grafos, marts e projetos SIGPesq), consolidando **70.545 registros** nas 15 entidades gerenciadas e introduzindo ajustes de esquema e volume em relação à versão intermediária anterior:

| Tabela (`*_canonical.json`) | Linhas (Versão Anterior) | Linhas (Versão Final) | Diferença | Colunas | Mudanças na Versão Final |
|---|---|---|---|---|---|
| `researchers` | 18.429 | **18.334** | -95 | **22** (+1) | **Nova coluna `linkedin_url`** adicionada ao esquema de pessoas |
| `students` | 11.799 | **11.717** | -82 | **22** (+1) | **Nova coluna `linkedin_url`** adicionada ao esquema de pessoas |
| `initiatives` | 15.505 | **15.546** | +41 | 19 | 264 iniciativas enriquecidas via SIGPesq (`enrichment`); status distribuídos entre `Concluded` (13.485), `Active` (1.084), `In Progress` (859), `Cancelled` (113) e `Unknown` (5) |
| `articles` | 8.057 | **8.079** | +22 | 9 | Atualização de volume |
| `professional_activities` | 9.557 | **9.560** | +3 | 17 | Atualização de volume |
| `research_productions` | 3.729 | **3.732** | +3 | 13 | Atualização de volume |
| `fellowships` | 21 | **20** | -1 | 5 | Consolidação de registros de bolsas |
| `knowledge_areas` | 1.563 | **1.563** | 0 | 3 | Inalterada |
| `proficiencies` | 794 | **794** | 0 | 8 | Inalterada |
| `organizations` | 440 | **440** | 0 | 5 | Inalterada |
| `research_groups` | 353 | **353** | 0 | 13 | Inalterada |
| `advisorships` | 190 | **190** | 0 | 9 | Status de projetos de orientação: `Concluded` (141), `Active` (48) e `N/A` (1) |
| `awards` | 183 | **183** | 0 | 5 | Inalterada |
| `campuses` | 23 | **23** | 0 | 6 | Inalterada |
| `languages` | 11 | **11** | 0 | 3 | Inalterada |

---

## Clarifications

### Session 2026-11-16

- Q: Como os gráficos e indicadores devem ser acessados pelo usuário dentro de cada um dos 5 projetos e de suas abas de tabelas? → A: Dashboard Geral do Projeto (visão consolidada em cada um dos 5 projetos) + botão/aba "Ver Gráficos da Tabela" dentro de cada página de tabela individual.
- Q: Onde o indicativo visual de "Ativo ou Inativo" deve ser exibido para o usuário? → A: Em ambos os níveis: selo Ativo/Inativo por registro nas tabelas e formulários de projetos (`initiatives` e `advisorships` no Horizon e Horizon Legacy) e selo de base ativa nos cartões de seleção de projetos (`/projects`).
- Q: Como os gráficos de "quanto recebem os pesquisadores / bolsas" devem apresentar os valores financeiros nas abas Horizon e Horizon Legacy? → A: Visão completa (Orientadores + Bolsistas + Programas): mostra quanto cada pesquisador orientador movimenta/supervisiona em bolsas, quanto cada bolsista recebe e a distribuição financeira por programa (`fellowships`).

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Adaptação Completa à Versão Final do `exports_canonical.zip` (Priority: P1)

O curador importa e exporta a versão final do pacote `exports_canonical.zip` no ambiente Horizon, visualizando e editando todos os campos atualizados das 15 tabelas canônicas — incluindo o novo atributo `linkedin_url` em Pesquisadores (`researchers`) e Alunos (`students`) — e obtendo fidelidade total de dados, tipos e arquivos complementares ao exportar o pacote.

**Why this priority**: É a base de dados oficial da versão final do Horizon; sem o suporte às 22 colunas de `researchers` e `students` e às novas contagens de referência, o ciclo de importação e exportação descartaria a nova coluna `linkedin_url`.

**Independent Test**: Importar o novo `exports_canonical.zip`, verificar a presença dos 70.545 registros distribuídos nas 15 tabelas, abrir os formulários de Pesquisadores e Alunos para inspecionar e editar o campo `linkedin_url`, e exportar o pacote confirmando que todas as 22 colunas e as 683 entradas não gerenciadas permanecem intactas.

**Acceptance Scenarios**:

1. **Given** o pacote final `exports_canonical.zip` na pasta do projeto, **When** o administrador importa o pacote na aba Horizon, **Then** todas as 15 tabelas canônicas são carregadas integralmente (totalizando 70.545 registros), incluindo a coluna `linkedin_url` nas tabelas de Pesquisadores e Alunos.
2. **Given** registros carregados em Pesquisadores (`researchers`) ou Alunos (`students`) na aba Horizon, **When** o administrador abre o formulário de visualização/edição de um registro, **Then** o campo "LinkedIn URL" (`linkedin_url`) está visível e editável junto aos demais 21 campos da entidade.
3. **Given** a base carregada a partir do pacote final `exports_canonical.zip`, **When** o administrador executa a exportação do pacote `.zip`, **Then** o arquivo exportado preserva todas as 698 entradas (15 JSONs canônicos regenerados com 100% das colunas e tipos originais + 683 entradas não gerenciadas preservadas byte a byte).

---

### User Story 2 - Indicativo Visual de Projeto Ativo ou Inativo (Priority: P1)

Ao navegar pelas tabelas e formulários de projetos no ResearchHub (Iniciativas `initiatives` e Orientações `advisorships`), o usuário identifica imediatamente se cada projeto está **Ativo** ou **Inativo/Concluído** por meio de um selo visual semântico claro (ex.: "Ativo" em destaque verde para projetos com status `Active` ou `In Progress`, "Inativo / Concluído" em tom neutro para `Concluded`, e "Inativo / Cancelado" em tom de alerta para `Cancelled`), além de poder visualizar o status destacado na listagem e no formulário de detalhes.

**Why this priority**: Permite ao gestor e curador distinguir instantaneamente quais projetos estão em execução ativa (`Active` / `In Progress`) e quais já foram encerrados (`Concluded` / `Cancelled`) sem precisar interpretar textos brutos em inglês ou abrir registro por registro.

**Independent Test**: Acessar as páginas de Iniciativas e Orientações e verificar que cada linha da tabela e o modal de visualização exibem o selo visual indicando claramente se o projeto está Ativo ou Inativo, acompanhado da situação específica (Em Execução, Em Andamento, Concluído, Cancelado ou Não Informado).

**Acceptance Scenarios**:

1. **Given** que o usuário acessa a listagem de Iniciativas (`initiatives`) ou Orientações (`advisorships`) no Horizon ou Horizon Legacy, **When** visualiza a coluna de status dos projetos, **Then** cada registro exibe um indicador visual colorido e legível classificando o projeto como **Ativo** (quando `status` for `Active` ou `In Progress`) ou **Inativo** (quando `status` for `Concluded`, `Cancelled`, `Unknown`, `N/A` ou vazio), indicando também o estado detalhado traduzido (ex.: `Ativo`, `Ativo (Em Andamento)`, `Inativo (Concluído)`, `Inativo (Cancelado)`).
2. **Given** que o usuário abre o formulário de visualização/edição de uma Iniciativa ou Orientação, **When** consulta o cabeçalho ou o campo de status do projeto, **Then** vê em destaque o indicativo visual se o projeto está **Ativo** ou **Inativo**, mantendo a possibilidade de editar o valor canônico do campo `status`.
3. **Given** que o usuário acessa a tela de seleção de projetos (`/projects`), **When** visualiza os 5 cartões de projetos disponíveis, **Then** cada cartão exibe um indicativo visual informando se aquele ambiente está **Ativo (Com dados carregados)** ou **Sem dados carregados** (junto à contagem resumida de registros quando ativo).

---

### User Story 3 - Dashboards Analíticos Contextualizados para Todas as Abas (Priority: P1)

Em todas as abas do ResearchHub — abrangendo os projetos/ambientes (**Horizon**, **Horizon Legacy**, **SRC**, **Egressos** e **Egressos — Dados**) e suas respectivas abas de tabelas — o usuário dispõe da opção de visualizar **Dashboards com gráficos e indicadores próprios do domínio de cada aba**, calculados sobre os registros reais armazenados localmente:
- **Abas Horizon e Horizon Legacy (Visão Geral e 15 Abas de Entidades)**:
  - Visão Geral consolidada: quantos projetos estão ativos vs. inativos/concluídos, quantidade de registros nas 15 tabelas (pesquisadores, alunos, iniciativas, grupos, artigos, produções, orientações, organizações, etc.), valores e distribuição de bolsas por programa e quanto recebem/supervisionam os pesquisadores, de quais instituições vêm (formação acadêmica e atuação profissional), titulação acadêmica e evolução anual da produção científica;
  - Gráficos específicos em cada uma das 15 abas de tabelas condizentes com os atributos daquela entidade (ex.: em *Pesquisadores*, titulação, instituições de formação, classificação e top pesquisadores por artigos/projetos; em *Iniciativas*, projetos ativos vs. inativos, tipos de iniciativa, evolução por ano, campus e cobertura de enriquecimento SIGPesq; em *Orientações* e *Bolsas*, valores por programa, bolsistas remunerados vs. voluntários e ranking de orientadores; em *Atividades Profissionais*, instituições de origem/vínculo, tipos de vínculo e atividades atuais vs. encerradas; em *Artigos* e *Produções Científicas*, evolução por ano, tipos e veículos/editoras; em *Grupos de Pesquisa*, distribuição por campus, áreas do conhecimento e tamanho das equipes; e demais tabelas).
- **Aba SRC (`/src` e Ações)**:
  - Indicadores e gráficos de ações de extensão e ensino: total de ações e participações, distribuição de ações por natureza e por tipo, ações com maior número de participações, distribuição por coordenador e por campus, e proporção de ações com participações vinculadas.
- **Aba Egressos (`/egressos` e 21 tabelas de `dados/`)**:
  - Indicadores e gráficos sobre carreira e empregabilidade dos egressos: total de perfis e registros nas 21 tabelas, distribuição de egressos por curso, principais empresas empregadoras, distribuição geográfica/localização atual, áreas de atuação, cargos mais frequentes e porte/setor das empresas da vitrine.
- **Aba Egressos — Dados (`/egressos-dados` e 39 datasets)**:
  - Indicadores e gráficos consolidados dos 39 datasets: contagem de registros por dataset e por classificação (`pii`, `interno`, `publicavel`) / papel (`entrada`, `saida`), distribuição da coorte de alunos por curso, taxa de permanência na área de tecnologia (`ainda_em_tech`), principais empresas e locais de atuação.

**Why this priority**: Garante que qualquer aba ou tabela aberta pelo curador ofereça visualizações gráficas imediatas e pertinentes ao contexto daquela tabela/projeto, transformando todas as áreas do ResearchHub em ambientes analíticos completos.

**Independent Test**: Navegar por cada projeto (Horizon, Horizon Legacy, SRC, Egressos, Egressos — Dados) e suas abas de tabelas, acionar a visualização de Dashboard/Gráficos e verificar que os cartões de KPIs e gráficos exibidos refletem fielmente os atributos específicos daquela aba.

**Acceptance Scenarios**:

1. **Given** que o usuário está nos ambientes **Horizon** ou **Horizon Legacy**, **When** acessa o Dashboard principal ou alterna para a visão de gráficos em qualquer uma das 15 abas de entidades, **Then** visualiza gráficos e métricas específicos daquela tabela (incluindo projetos ativos vs. inativos, contagens por tabela, valores de bolsas e quanto recebem/supervisionam os pesquisadores, instituições de origem/vínculo, titulação e produção por ano).
2. **Given** que o usuário está na aba **SRC**, **When** visualiza o painel de gráficos do SRC, **Then** vê gráficos de distribuição das ações por natureza, por tipo, top coordenadores, distribuição por campus e volume de participações por tipo de atividade.
3. **Given** que o usuário está na aba **Egressos** ou **Egressos — Dados**, **When** visualiza o painel de gráficos da base ou da tabela selecionada, **Then** vê gráficos condizentes com os dados de egressos (distribuição por curso, empresas, áreas de atuação, localização, permanência em tech e volume por tabela/dataset).
4. **Given** qualquer aba cujo banco local ainda esteja vazio, **When** o usuário abre a visualização de Dashboard/Gráficos daquela aba, **Then** o sistema apresenta um estado vazio informativo sem falhas de renderização.

---

### Edge Cases

- **Base vazia ou parcialmente preenchida em qualquer aba**: Se o usuário abrir o Dashboard de um projeto ou tabela antes de importar dados naquela aba, todos os indicadores numéricos devem exibir `0` ou estado vazio amigável, sem falha de renderização ou divisão por zero.
- **Valores de status variados ou nulos em projetos**: Caso um registro de `initiatives` ou `advisorships` possua `status` nulo, vazio, `"Unknown"` ou `"N/A"`, o indicador visual deve classificá-lo de forma segura como **Não Ativo / Não Informado** sem quebrar a listagem ou a agregação do gráfico.
- **Registros de formação acadêmica com ruído textual na instituição**: Na tabela `researchers`, alguns currículos possuem entradas de `academic_education` onde o campo `institution` contém textos de carga horária (ex.: `"(Carga Horária: 360h)"`); a agregação de instituições de origem deve filtrar entradas vazias ou que representem apenas rótulos de carga horária para destacar instituições reais nos gráficos.
- **Cálculo de bolsas e valores recebidos**: Como os valores de bolsas existem tanto no catálogo `fellowships` quanto nos vínculos aninhados dentro de `advisorships` (onde cada orientação vincula `supervisor_id`/`supervisor_name`, `person_id`/`person_name` e objeto `fellowship` com `name` e `value`), o painel deve consolidar ambas as visões: valores por modalidade de bolsa e soma de bolsas/valores associados aos pesquisadores e bolsistas.

## Requirements *(mandatory)*

### Functional Requirements

**Adaptação à Versão Final do `exports_canonical.zip`**

- **FR-001**: O sistema DEVE suportar a coluna `linkedin_url` nas entidades de Pesquisadores (`researchers`) e Alunos (`students`), totalizando 22 colunas canônicas para ambas as entidades, preservando seus valores na importação, visualização, edição e exportação.
- **FR-002**: As páginas de Pesquisadores e Alunos na aba Horizon DEVEM expor o campo `linkedin_url` no formulário de visualização e edição.
- **FR-003**: O ciclo de importação e exportação do pacote `exports_canonical.zip` de referência DEVE validar as contagens atualizadas da versão final (70.545 registros nas 15 tabelas canônicas e 698 entradas totais no arquivo `.zip`, com 683 entradas não gerenciadas preservadas byte a byte).

**Indicativo Visual de Projeto Ativo ou Inativo**

- **FR-004**: Nas listagens de tabelas que possuem status de projeto/atividade (Iniciativas `initiatives` e Orientações `advisorships` no Horizon e Horizon Legacy), o sistema DEVE exibir um indicativo visual semântico (selo/badge colorido) informando claramente se o projeto está **Ativo** (`Active` ou `In Progress`) ou **Inativo** (`Concluded`, `Cancelled`, `Unknown`, `N/A` ou ausente), acompanhado da descrição legível da situação, além de exibir nos cartões da tela de seleção de projetos (`/projects`) um indicativo visual informando se cada ambiente está ativo (com dados carregados no banco local) ou vazio.
- **FR-005**: Nos formulários de visualização/edição de Iniciativas e Orientações, o sistema DEVE apresentar o indicativo visual de projeto **Ativo** ou **Inativo** ao lado do campo `status`, atualizando o indicativo dinamicamente conforme o valor do status.

**Dashboards Analíticos com Gráficos para Todas as Abas**

- **FR-006**: O sistema DEVE disponibilizar tanto um **Dashboard Geral do Projeto** (visão consolidada com KPIs e gráficos cruzados em cada um dos 5 ambientes: **Horizon**, **Horizon Legacy**, **SRC**, **Egressos** e **Egressos — Dados**) quanto um modo/painel **"Ver Gráficos da Tabela"** dentro de cada página de tabela individual, exibindo gráficos e indicadores condizentes com os atributos específicos daquela tabela.
- **FR-007**: Toda a consolidação estatística, contagem, agrupamento e agregação de dados para os gráficos dos Dashboards DEVE ser processada no núcleo da aplicação desktop sobre o banco local, entregando os dados agregados prontos para exibição visual sem consultas diretas no front-end e sem necessidade de conexão de rede.
- **FR-008**: Nos ambientes **Horizon** e **Horizon Legacy**, o Dashboard Geral e as abas de entidades DEVEM apresentar gráficos e métricas cobrindo:
  1. **Status de Projetos**: total de projetos, quantos estão ativos (`Active` + `In Progress`) vs. inativos/concluídos (`Concluded`) vs. cancelados (`Cancelled`), divisão por tipo de iniciativa e distribuição por Campus;
  2. **Volume das Tabelas**: quantidade de registros nas 15 tabelas canônicas;
  3. **Remuneração / Bolsas de Pesquisa (Visão Completa: Orientadores + Bolsistas + Programas)**: valores tabelados por modalidade de bolsa (`fellowships`), distribuição de bolsas e montante mensal por programa (PIBIC, PIBITI, PIBIC-JR, PROPÓS, etc.) vs. voluntários, ranking de quanto cada pesquisador orientador movimenta/supervisiona em bolsas e ranking de quanto cada bolsista recebe;
  4. **Instituições de Origem e Vínculo**: principais instituições de atuação profissional (`professional_activities`) e de formação acadêmica (`academic_education`) dos pesquisadores;
  5. **Métricas Específicas por Tabela**: gráficos contextualizados em cada uma das 15 abas de entidades (titulação acadêmica, produção científica por ano e tipo, áreas do conhecimento, idiomas/proficiências, premiações por ano, grupos por campus/área, etc.).
- **FR-009**: Na aba **SRC**, o Dashboard DEVE exibir gráficos e indicadores próprios das ações de extensão e ensino: total de ações e participações, distribuição por natureza e por tipo de ação, ranking de coordenadores, distribuição por campus e distribuição de participações por tipo/atividade.
- **FR-010**: Nas abas **Egressos** (21 tabelas) e **Egressos — Dados** (39 tabelas), os Dashboards DEVEM exibir gráficos e indicadores próprios do estudo de egressos: volume de registros por arquivo/tabela, distribuição de egressos por curso, principais empresas e setores empregadores, áreas e cargos de atuação, distribuição geográfica/localização atual, taxa de atuação em tecnologia (`ainda_em_tech`) e métricas específicas da tabela selecionada.

### Key Entities *(include if feature involves data)*

- **Esquema Atualizado de Pessoas (`researchers`, `students`)**: Entidades canônicas de pessoas agora compostas por 22 atributos, incluindo o novo campo `linkedin_url`.
- **Indicador de Status de Projeto**: Classificação derivada do atributo `status` de Iniciativas (`initiatives`) e Orientações (`advisorships`), distinguindo projetos **Ativos** (`Active`, `In Progress`) de projetos **Inativos** (`Concluded`, `Cancelled`, `Unknown`, `N/A`).
- **Dashboards Analíticos Multidomínio**: Conjunto de indicadores e séries gráficas gerados localmente para cada aba do sistema (Horizon, Horizon Legacy, SRC, Egressos e Egressos — Dados, além das visões por tabela), adaptados aos atributos de negócio de cada contexto.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A importação e exportação (round-trip) da versão final do `exports_canonical.zip` processa **100% dos 70.545 registros** nas 15 tabelas canônicas (incluindo as 22 colunas de `researchers` e `students` com `linkedin_url`) e preserva **100% das 683 entradas não gerenciadas** byte a byte.
- **SC-002**: Em **100% dos registros** exibidos nas tabelas e formulários de Iniciativas e Orientações, o usuário visualiza imediatamente o selo indicativo de projeto **Ativo** ou **Inativo** correspondente ao status do registro.
- **SC-003**: Em **100% das abas de projeto** (Horizon, Horizon Legacy, SRC, Egressos e Egressos — Dados) e suas abas de tabelas, o usuário consegue abrir a visão de Dashboard/Gráficos e visualizar indicadores coerentes com os dados daquela aba em **menos de 2 segundos** em execução local.
- **SC-004**: Os Dashboards respondem visualmente a todas as dimensões solicitadas pelo usuário (projetos ativos vs. inativos, volume das tabelas, valores e distribuição de bolsas por programa e pesquisador, instituições de origem/atuação dos pesquisadores e métricas específicas das abas SRC e Egressos) com dados **100% consistentes** com os registros presentes no banco local.
- **SC-005**: **100% da suíte de testes automatizados** (testes unitários, de integração e de componentes) passa sem regressões.

## Assumptions

- Projetos com `status` igual a `"Active"` ou `"In Progress"` são considerados **Ativos** (no pacote final de referência em `initiatives_canonical.json`, correspondem a 1.084 `Active` + 859 `In Progress` = 1.943 iniciativas em execução/andamento), enquanto projetos com `"Concluded"`, `"Cancelled"`, `"Unknown"` ou `"N/A"` são considerados **Inativos**, exibindo no selo tanto a classificação macro (Ativo / Inativo) quanto o detalhamento específico.
- As informações financeiras sobre bolsas ("quanto recebem") nas abas Horizon / Horizon Legacy são extraídas da tabela `fellowships` e dos vínculos de orientação em `advisorships` (que associam orientadores, estudantes e valores mensais das bolsas).
- As informações de "quais instituições vêm" os pesquisadores são extraídas combinando o histórico de formação acadêmica (`academic_education` em `researchers`) e as atividades profissionais (`professional_activities`).
- Cada ambiente de projeto (Horizon, Horizon Legacy, SRC, Egressos e Egressos — Dados) e suas abas de tabelas exibem gráficos calculados exclusivamente sobre suas próprias tabelas isoladas no banco local.
