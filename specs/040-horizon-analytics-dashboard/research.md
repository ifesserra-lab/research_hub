# Phase 0 Research: Adaptação Final do Pacote Canônico Horizon, Indicador de Status e Dashboards Analíticos

**Feature**: `040-horizon-analytics-dashboard`  
**Date**: 2026-11-16

## R1. Suporte à Nova Coluna `linkedin_url` em `researchers` e `students`

- **Decision**: Adicionar `"linkedin_url"` à constante `PERSON_COLUMNS` em `src-tauri/src/registry.rs` e criar a migração append-only `src-tauri/migrations/006_person_linkedin_url.sql` adicionando `linkedin_url TEXT` às tabelas `researchers`, `students`, `hv2_researchers` e `hv2_students`.
- **Rationale**:
  - Na versão final de `exports_canonical.zip`, `researchers_canonical.json` (18.334 linhas) e `students_canonical.json` (11.717 linhas) possuem 22 colunas (com a adição de `linkedin_url`).
  - Como `db.rs` exige migrações append-only (`PRAGMA user_version`), a migração `006` atualiza bancos existentes sem perda de dados, e o pareamento entre `PERSON_COLUMNS` e o schema SQLite mantém os testes `registry_and_schema_agree` e `horizon_v2_registry_and_schema_agree` íntegros, além de garantir que `real_archive_round_trip_preserves_types_and_entries` preserve todas as 22 colunas no export.
- **Alternatives considered**:
  - Editar `001_init.sql` e `005_horizon_v2_tables.sql` diretamente: descartado para `001_init.sql` porque bancos já inicializados na `user_version = 5` não executariam novamente migrações passadas.

## R2. Classificação Semântica de Projeto Ativo vs. Inativo (`ProjectStatusBadge`)

- **Decision**: Centralizar a regra de classificação de status de projeto (em Rust para agregações e no componente `ProjectStatusBadge.jsx` para renderização visual de células e formulários) mapeando:
  - **Ativo**: `"Active"` → `Ativo (Em Execução)` e `"In Progress"` → `Ativo (Em Andamento)` (selo verde/esmeralda).
  - **Inativo**: `"Concluded"` → `Inativo (Concluído)` (selo azul/ardósia), `"Cancelled"` → `Inativo (Cancelado)` (selo vermelho/rosa), e `"Unknown"` / `"N/A"` / vazio / nulo → `Inativo (Não Informado)` (selo cinza/âmbar).
  - Além disso, criar o comando `projects_status` (`GET /projects/status`) para informar nos 5 cartões de `/projects` se cada ambiente possui base ativa carregada no SQLite local, o total de registros carregados e (para Horizon e Horizon Legacy) a quantidade de iniciativas ativas.
- **Rationale**:
  - Na base final `exports_canonical.zip`, `initiatives` contém `Concluded` (13.485), `Active` (1.084), `In Progress` (859), `Cancelled` (113) e `Unknown` (5), enquanto `advisorships` contém `Concluded` (141), `Active` (48) e `N/A` (1).
  - Agrupar `Active` e `In Progress` como projetos **Ativos** (1.943 iniciativas no total) reflete com fidelidade os projetos em andamento, preservando entre parênteses o estado específico traduzido.
- **Alternatives considered**:
  - Traduzir ou sobrescrever o valor salvo no banco para português: descartado porque quebraria a paridade do arquivo `exports_canonical.zip` na exportação (Princípio II). O valor persistido permanece o canônico (`Active`, `In Progress`, `Concluded`, `Cancelled`), enquanto a camada visual exibe o badge semântico.

## R3. Cálculos de Bolsas ("Quanto Recebem os Pesquisadores / Bolsistas") e Instituições de Origem

- **Decision**: Calcular as métricas financeiras e institucionais em `src-tauri/src/analytics.rs` combinando as tabelas relacionais e os JSONs aninhados já armazenados no SQLite:
  1. **Modalidades de Bolsa (`fellowships`)**: ler `name`, `description` e `value` para listar o valor mensal de cada programa (ex.: `PROPÓS` R$ 1.008, `PIBITI`/`PIBIC` R$ 700–900, `PIBIC-JR` R$ 300, `PIVIC`/`PIVITI` R$ 0).
  2. **Distribuição por Programa, Orientador e Bolsista (`advisorships`)**: percorrer a lista JSON da coluna `advisorships` de cada projeto de orientação; cada item contém `status`, `supervisor_id`, `supervisor_name`, `person_id`, `person_name` e `fellowship` (`{ name, value }`). A partir disso, agregar:
     - Quantidade de bolsas e valor mensal por programa (`PIBIC`, `PIBITI`, `PIBIC-JR`, `PROPÓS`, etc.) vs. voluntários (`value == 0` ou `fellowship == null`);
     - **Top Pesquisadores Orientadores (`top_supervisors`)**: somatório mensal de bolsas supervisionadas, total de bolsistas remunerados e total de orientandos por pesquisador;
     - **Top Bolsistas / Recebedores (`top_recipients`)**: valor mensal recebido, quantidade de bolsas e programa de cada bolsista.
  3. **Instituições de Origem e Atuação**:
     - De `professional_activities`: agrupar por `institution_name` (não vazio), obtendo o ranking das instituições de atuação profissional;
     - De `researchers.academic_education` (JSON array): extrair `institution` e `degree`, filtrando strings vazias ou que comecem com `"(Carga Horária"` (ruído de importação do Lattes presente em 209 registros), obtendo o ranking real de universidades de formação (UFES, UFV, IFES, UENF, UFRRJ, UFLA, UFMG, etc.) e a distribuição por maior titulação (`Doutorado`, `Mestrado`, `Especialização`, `Graduação`, etc.).
- **Rationale**:
  - Cumpre integralmente o Princípio VI (toda regra de negócio e agregação em Rust) e a Clarificação Q3 (visão completa: orientadores + bolsistas + programas), sem depender de arquivos externos fora do SQLite após a importação.
- **Alternatives considered**:
  - Ler diretamente `advisorship_analytics.json` de dentro do `.zip` original: descartado porque não refletiria edições, criações ou exclusões feitas pelo curador no banco SQLite nem funcionaria quando a base fosse editada manualmente.

## R4. Arquitetura Unificada de Gráficos por Projeto e por Tabela (`AnalyticsDashboard` + `TableAnalyticsPanel`)

- **Decision**:
  1. Criar um contrato uniforme `ChartSection` (`{ id, title, subtitle, kind, items: Vec<ChartItem> }`) e `KpiCard` (`{ label, value, sublabel, tone }`) retornado pelos comandos Rust para alimentar tanto os **Dashboards Gerais** dos 5 projetos quanto o painel **"Ver Gráficos da Tabela"** de qualquer tabela individual:
     - **Horizon (`horizon-v2`) e Horizon Legacy (`horizon`)**: Dashboard Geral + gráficos específicos para cada uma das 15 rotas canônicas (`researchers`, `students`, `initiatives`, `advisorships`, `fellowships`, `articles`, `research_productions`, `groups`, `professional_activities`, `awards`, `organizations`, `campuses`, `knowledge_areas`, `proficiencies`, `languages`).
     - **SRC (`src`)**: Dashboard Geral na página inicial `/src` + painel de gráficos na página `/src/acoes` (ações por natureza, por tipo, por campus, top coordenadores, ações com mais participações e participações por tipo/atividade).
     - **Egressos (`egressos`)**: Dashboard Geral na página `/egressos` + painel de gráficos por tabela em `/egressos/tabelas` cobrindo as 21 tabelas (extraindo contagens por seção e agregações automáticas das colunas categóricas/numéricas da tabela selecionada, como `curso`, `empresa_atual`/`empresa`, `area_atual`, `cargo_atual`/`cargo`, `local_atual`/`lugar`, `porte`, `setor`, etc.).
     - **Egressos — Dados (`egressos-dados`)**: Dashboard Geral + painel de gráficos por tabela nas 39 tabelas (classificação `pii`/`interno`/`publicavel`, papel `entrada`/`saida`, coorte de alunos por `curso`, taxa de permanência `ainda_em_tech`, empresas, áreas, locais e distribuição dinâmica das colunas da tabela ativa).
  2. Renderizar os gráficos usando componentes React baseados em **SVG + Tailwind CSS** (gráficos de barras horizontais comparativas, gráficos de colunas verticais para séries temporais, gráficos de rosca/proporção para status/categorias e tabelas de ranking financeiro formatadas em `BRL`).
- **Rationale**:
  - Componentes SVG/Tailwind não exigem bibliotecas pesadas de terceiros, funcionam 100% offline, não sofrem com incompatibilidades de `Canvas`/`ResizeObserver` no `jsdom` (permitindo testes rápidos e determinísticos no `Vitest`) e mantêm a identidade visual limpa do ResearchHub.
- **Alternatives considered**:
  - Introduzir `chart.js` ou `recharts`: descartado por adicionar dependências extras no `package.json`, exigir mocks de `Canvas`/`ResizeObserver` em todos os testes unitários e aumentar o bundle estático sem necessidade.
