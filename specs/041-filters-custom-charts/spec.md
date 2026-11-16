# Feature Specification: Filtros Avançados nas Tabelas e Dashboards e Criação de Novos Gráficos Personalizados nas 5 Abas

**Feature Branch**: `041-filters-custom-charts`

**Created**: 2026-11-16

**Status**: Draft

**Input**: User description: "proxima feature , a opçao de colocar filtros nas tabelas e no dashboard, e a partir deles criar novos graficos no dashboard  , vale para as 5 abas"

## Clarifications

### Session 2026-11-16

- Q: Quando o usuário adiciona dois ou mais filtros simultaneamente em uma tabela ou gráfico, como essas condições devem ser combinadas? → A: `E (AND)` por padrão com opção de alternar para `OU (OR)`.
- Q: Onde um novo gráfico personalizado criado a partir dos filtros de uma tabela deve ficar visível? → A: Em ambos (no painel "Ver Gráficos da Tabela" da página da tabela e no Dashboard Geral daquela aba).
- Q: Quais métricas de cálculo os novos gráficos personalizados devem suportar ao agrupar por uma coluna? → A: Contagem de Registros (padrão) e Soma de Coluna Numérica ou Quantidade de Vínculos.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Filtros Estruturados por Coluna nas Tabelas das 5 Abas (Priority: P1)

Ao consultar qualquer tabela nos 5 ambientes de projeto do ResearchHub (**Horizon**, **Horizon Legacy**, **SRC**, **Egressos** e **Egressos — Dados**), o usuário pode aplicar um ou mais filtros estruturados por coluna (ex.: filtrar Iniciativas onde `status` é `Active` e `campus` contém `Serra`; filtrar Pesquisadores onde `classification` é `researcher`; filtrar Ações do SRC onde `natureza` é `Extensão`; ou filtrar Egressos por `curso` e `empresa_atual`), visualizando imediatamente a listagem e a contagem total de registros atualizadas para o recorte selecionado.

**Why this priority**: A busca textual única busca apenas na coluna principal de nome/título; filtros por coluna permitem segmentar qualquer atributo das tabelas nas 5 abas e servem de base para gerar gráficos direcionados.

**Independent Test**: Acessar uma tabela em qualquer uma das 5 abas, adicionar filtros por coluna (ex.: coluna + operador/valor), verificar que a listagem paginada e o contador total exibem apenas os registros que satisfazem os filtros, e limpar os filtros para restaurar a listagem completa.

**Acceptance Scenarios**:

1. **Given** que o usuário está em uma página de tabela de qualquer uma das 5 abas (**Horizon**, **Horizon Legacy**, **SRC**, **Egressos** ou **Egressos — Dados**), **When** adiciona um filtro selecionando uma coluna da tabela e informando/selecionando o valor ou condição (ex.: contém, igual a, diferente de, preenchido/vazio), **Then** a tabela e o total de registros são atualizados exibindo apenas os registros correspondentes ao filtro.
2. **Given** que o usuário aplicou múltiplos filtros simultâneos em uma tabela (ex.: `status = Active` e `campus contém Serra`), **When** consulta a listagem, **Then** todos os filtros ativos são exibidos como etiquetas (chips) removíveis e os resultados respeitam a combinação de todos os filtros ativos.
3. **Given** filtros ativos em uma tabela, **When** o usuário clica em remover um filtro individual ou em "Limpar filtros", **Then** a listagem é recarregada imediatamente refletindo os filtros restantes ou a base completa.

---

### User Story 2 - Filtros Globais e de Recorte nos Dashboards das 5 Abas (Priority: P1)

Ao abrir o Dashboard Analítico (tanto o Dashboard Geral de cada um dos 5 projetos quanto o painel "Ver Gráficos da Tabela" dentro de cada página de tabela), o usuário pode aplicar filtros de recorte (ex.: filtrar por Campus, Status Ativo/Inativo, Ano, Classificação, Natureza, Curso ou qualquer coluna/valor) para recalcular os indicadores (KPIs) e os gráficos do Dashboard exclusivamente sobre o subconjunto de dados filtrado.

**Why this priority**: Permite análises exploratórias dinâmicas (ex.: "ver todos os gráficos de bolsas, projetos ativos e instituições apenas para o Campus Serra" ou "ver estatísticas de egressos apenas de um curso específico") em todas as 5 abas.

**Independent Test**: Abrir o Dashboard de qualquer uma das 5 abas, aplicar um filtro de recorte no painel de filtros do Dashboard e verificar que os cartões de KPI e os gráficos passam a refletir os valores calculados sobre o subconjunto filtrado.

**Acceptance Scenarios**:

1. **Given** que o usuário está no Dashboard Geral ou no painel de gráficos de uma tabela em qualquer uma das 5 abas, **When** seleciona ou aplica um filtro no Dashboard (ex.: `campus = Serra` ou `status = Active`), **Then** os cartões de resumo (KPIs) e os gráficos são recalculados e atualizados para refletir os registros que atendem ao filtro.
2. **Given** um gráfico exibido no Dashboard, **When** o usuário clica em uma categoria/barra do gráfico para filtrar ou limpa os filtros ativos no topo do Dashboard, **Then** o Dashboard indica claramente quais filtros estão ativos e permite restaurar a visão global com um clique.

---

### User Story 3 - Criação de Novos Gráficos Personalizados no Dashboard a partir dos Filtros (Priority: P1)

A partir dos filtros aplicados (seja diretamente na tabela ou no próprio Dashboard) em qualquer uma das 5 abas, o usuário pode **criar novos gráficos personalizados** e adicioná-los ao Dashboard, escolhendo:
- O título personalizado do gráfico (ou título gerado automaticamente a partir do filtro/coluna);
- A tabela/entidade e a coluna de agrupamento (ex.: agrupar por `status`, `campus`, `classification`, `year`, `institution_name`, `natureza`, `tipo`, `coordenador`, `curso`, `empresa_atual`, `area_atual`, `ainda_em_tech`, etc.);
- O tipo visual do gráfico (**Barras Horizontais**, **Rosca / Proporção**, **Colunas Verticais** ou **Tabela de Ranking**);
- Os filtros específicos vinculados àquele gráfico (incluindo um botão direto **"Criar Gráfico no Dashboard com estes Filtros"** na barra de filtros da tabela).

Os gráficos criados pelo usuário ficam salvos e visíveis no Dashboard da respectiva aba, podendo ser consultados, atualizados ou excluídos a qualquer momento.

**Why this priority**: Atende diretamente à necessidade de transformar qualquer recorte ou cruzamento filtrado nas tabelas em novas visualizações gráficas persistentes no Dashboard das 5 abas.

**Independent Test**: Aplicar um filtro em uma tabela ou abrir o construtor "Novo Gráfico" no Dashboard de qualquer uma das 5 abas, configurar a coluna de agrupamento, o tipo de gráfico e os filtros, salvar o novo gráfico e confirmar que ele passa a ser renderizado com os dados reais no Dashboard e permanece disponível ao reabrir a aba.

**Acceptance Scenarios**:

1. **Given** que o usuário aplicou filtros em uma tabela de qualquer uma das 5 abas (ex.: Iniciativas com `campus contém Serra`), **When** aciona a opção **"Criar Gráfico a partir destes Filtros"** e escolhe a coluna de agrupamento (ex.: `status`) e o tipo de gráfico (ex.: `Rosca` ou `Barras`), **Then** um novo gráfico personalizado é criado e exibido no Dashboard com os dados filtrados e agrupados.
2. **Given** que o usuário está no Dashboard de qualquer uma das 5 abas, **When** clica em **"+ Novo Gráfico Personalizado"**, seleciona a tabela/fonte da aba, a coluna de agrupamento, o tipo de gráfico e adiciona um ou mais filtros opcionais, **Then** o novo gráfico é calculado pelo núcleo da aplicação, exibido imediatamente no Dashboard e persistido localmente para aquela aba.
3. **Given** um gráfico personalizado já criado no Dashboard de uma aba, **When** o usuário clica em remover/excluir esse gráfico personalizado, **Then** o gráfico é removido do Dashboard sem afetar os gráficos padrão nem os dados das tabelas.

---

### Edge Cases

- **Filtro sem registros correspondentes (0 resultados)**: Quando a combinação de filtros em uma tabela ou em um gráfico personalizado não retorna nenhum registro, a tabela deve exibir mensagem clara ("Nenhum registro encontrado para os filtros selecionados" com botão para limpar filtros) e o gráfico deve exibir estado vazio informativo ("0 registros para este filtro") sem erro.
- **Colunas contendo objetos ou listas estruturadas (JSON)**: Quando o usuário filtra ou agrupa por uma coluna que armazena vínculos estruturados (como `campus`, `knowledge_areas`, `research_groups`, `team` ou `academic_education`), o motor de filtro e agrupamento deve inspecionar os nomes/títulos internos dos elementos (ex.: extrair `name` de cada item de `campus` ou `knowledge_areas`) em vez de exibir texto bruto serializado.
- **Isolamento entre as 5 abas**: Filtros aplicados e gráficos personalizados criados em uma aba (ex.: **Horizon**) pertencem exclusivamente àquela aba e nunca interferem nos dados ou gráficos de outra aba (**Horizon Legacy**, **SRC**, **Egressos** ou **Egressos — Dados**).
- **Preservação do pacote de exportação**: Gráficos personalizados e configurações de filtros salvos localmente nunca devem alterar os arquivos exportados (`exports_canonical.zip`, JSON consolidado do SRC ou pastas JSON de Egressos).

## Requirements *(mandatory)*

### Functional Requirements

**Filtros Estruturados nas Tabelas (5 Abas)**

- **FR-001**: Todas as tabelas das 5 abas do ResearchHub (**Horizon**, **Horizon Legacy**, **SRC**, **Egressos** e **Egressos — Dados**) DEVEM disponibilizar uma barra de filtros estruturados permitindo adicionar um ou mais filtros por coluna (selecionando a coluna, o operador de comparação — como `contém`, `igual a`, `diferente de`, `preenchido` ou `vazio` — e o valor desejado).
- **FR-002**: A listagem paginada, a ordenação e a contagem total de registros da tabela DEVEM respeitar simultaneamente a busca textual livre e todos os filtros de coluna ativos, combinando múltiplos filtros com `E (AND)` por padrão e permitindo ao usuário alternar a lógica de combinação para `OU (OR)`.
- **FR-003**: A interface de filtros da tabela DEVE exibir todos os filtros ativos como etiquetas visuais (chips) e permitir remover cada filtro individualmente ou limpar todos os filtros com um único clique.

**Filtros Interativos nos Dashboards (5 Abas)**

- **FR-004**: Os Dashboards Analíticos das 5 abas (tanto o Dashboard Geral de cada projeto quanto o painel "Ver Gráficos da Tabela") DEVEM permitir aplicar filtros de recorte por coluna/valor (ex.: Campus, Status Ativo/Inativo, Ano, Classificação, Natureza, Curso, Empresa, etc.) que recalculam dinamicamente os KPIs e os gráficos do painel sobre o subconjunto filtrado.
- **FR-005**: O usuário DEVE poder clicar em qualquer barra ou fatia de categoria de um gráfico no Dashboard para aplicar (ou remover) rapidamente aquele valor como filtro ativo, além de contar com controles explícitos para adicionar e limpar filtros no topo do Dashboard.

**Criação de Novos Gráficos Personalizados a partir de Filtros (5 Abas)**

- **FR-006**: Em todas as 5 abas, o usuário DEVE poder criar **novos gráficos personalizados** no Dashboard informando: título do gráfico, tabela/entidade de origem da aba, coluna de agrupamento (dimensão), métrica de cálculo (`count` Contagem de Registros por padrão, ou `sum` Soma de Coluna Numérica / Quantidade de Vínculos sobre uma coluna-alvo), tipo visual de gráfico (`bar` Barras Horizontais, `donut` Rosca/Proporção, `columns` Colunas Verticais ou `financial_table` Ranking) e uma lista opcional de filtros por coluna com lógica `AND`/`OR`.
- **FR-007**: A barra de filtros de qualquer tabela nas 5 abas DEVE oferecer um atalho direto (**"Criar Gráfico com estes Filtros"**) que pré-carrega os filtros ativos da tabela no criador de gráfico, exibindo o novo gráfico personalizado tanto no painel **"Ver Gráficos da Tabela"** da página atual quanto no **Dashboard Geral** daquela aba.
- **FR-008**: Os gráficos personalizados criados pelo usuário DEVEM ser persistidos no banco local de forma isolada por aba/projeto (sem alterar nenhuma tabela exportada nos pacotes de dados) e permitir exclusão individual pelo usuário.
- **FR-009**: Toda a avaliação de filtros (em tabelas e dashboards) e o cálculo de agregação dos gráficos personalizados DEVEM ser executados no núcleo da aplicação desktop sobre o banco local, tratando adequadamente tanto colunas escalares quanto colunas estruturadas (extraindo nomes/títulos de listas de vínculos).

### Key Entities *(include if feature involves data)*

- **Regra de Filtro (`ColumnFilter`)**: Critério estruturado composto por coluna/campo (`field`), operador (`contains`, `equals`, `not_equals`, `is_empty`, `not_empty`) e valor (`value`), aplicável a listagens de tabelas, cálculos de Dashboard e gráficos personalizados.
- **Gráfico Personalizado (`CustomDashboardChart`)**: Configuração persistida localmente vinculada a uma aba/projeto (`scope`) e a uma tabela (`entity`), contendo identificador, título, tabela de origem, coluna de agrupamento (`group_by`), métrica (`metric`: `count` ou `sum` + `metric_field` opcional), tipo visual (`kind`), lógica de filtros (`filter_logic`: `and` ou `or`), lista de filtros (`filters`) e data de criação.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: O usuário consegue aplicar e combinar múltiplos filtros por coluna nas tabelas de **100% das 5 abas** (**Horizon**, **Horizon Legacy**, **SRC**, **Egressos** e **Egressos — Dados**), obtendo resultados paginados e contagens exatas em **menos de 500ms** em execução local.
- **SC-002**: O usuário consegue aplicar filtros de recorte nos Dashboards das **5 abas** e visualizar os indicadores (KPIs) e gráficos recalculados sobre o subconjunto filtrado em **menos de 2 segundos**.
- **SC-003**: O usuário consegue criar novos gráficos personalizados a partir de filtros (tanto pelo Dashboard Geral quanto diretamente pela barra de filtros de qualquer tabela nas 5 abas), visualizando o novo gráfico renderizado imediatamente e preservado ao reabrir a aba.
- **SC-004**: Criar, editar ou remover filtros e gráficos personalizados causa **0% de alteração** nos pacotes exportados pelas 5 abas (`exports_canonical.zip`, JSON do SRC e pastas JSON de Egressos).
- **SC-005**: **100% da suíte de testes automatizados** (testes unitários, de integração e de componentes) passa sem regressões.

## Assumptions

- Os operadores de filtro padrão (`contém`, `igual a`, `diferente de`, `está preenchido`, `está vazio`) cobrem tanto colunas textuais/categóricas (status, campus, curso, empresa, natureza) quanto anos, números e valores booleanos nas 5 abas.
- Quando uma coluna agrupada ou filtrada contém objetos ou listas de vínculos (ex.: `campus`, `knowledge_areas`, `research_groups`), o sistema extrai automaticamente o atributo `name` ou `title` dos elementos para permitir filtragem e agrupamento legíveis por nome.
- Os gráficos personalizados criados pelo usuário são armazenados em uma tabela interna dedicada no banco local (`custom_dashboard_charts`, com `exported: false`), garantindo persistência entre sessões do aplicativo sem interferir nos arquivos de exportação de domínio.
