# Phase 0 Research: Filtros Avançados nas Tabelas e Dashboards e Criação de Novos Gráficos Personalizados nas 5 Abas

**Feature**: `041-filters-custom-charts`  
**Date**: 2026-11-16

## R1. Gramática Unificada de Filtros (`FilterSpec` e `ColumnFilter`)

- **Decision**: Representar qualquer conjunto de filtros ativos (em tabelas, dashboards ou gráficos personalizados) através da estrutura JSON serializada `FilterSpec`:
  ```json
  {
    "logic": "and",
    "rules": [
      { "field": "status", "operator": "equals", "value": "Active" },
      { "field": "campus", "operator": "contains", "value": "Serra" }
    ]
  }
  ```
  enviada como parâmetro opcional `filters` nas rotas GET de listagem e analytics, e armazenada em `filters_json` na tabela `custom_dashboard_charts`.
- **Rationale**:
  - Suporta 1..N regras combinadas por `"and"` (padrão) ou `"or"` (Clarificação Q1).
  - Permite que a mesma estrutura seja criada na barra de filtros da tabela (`FilterBuilderBar`), repassada para a listagem paginada, aplicada ao painel de gráficos da tabela e salva diretamente em um novo gráfico personalizado com zero retrabalho de conversão.
- **Alternatives considered**:
  - Passar um query param diferente por nome de coluna (`?status=Active&campus=Serra`): descartado porque não permite especificar o operador (`contains` vs `equals` vs `not_equals` vs `is_empty` vs `not_empty`), não suporta alternar entre `AND` e `OR`, e colidiria com parâmetros reservados como `limit`, `offset`, `search`, `sort` e `order`.

## R2. Avaliação de Filtros e Extração de Valores em Colunas Escalares e JSON Estruturado

- **Decision**: Implementar no módulo `src-tauri/src/analytics.rs` o avaliador universal `record_matches_filter_spec(record: &Record, spec: &FilterSpec) -> bool` e o extrator `extract_field_tokens(val: &Json) -> Vec<String>`:
  - Quando a coluna contém um escalar (`String`, `Number`, `Bool`), o extrator obtém sua representação textual (e, para booleanos e status de projetos, aceita também os sinônimos legíveis exibidos na interface, como `"Ativo"`/`"Sim"` para `true`/`"Active"`/`"In Progress"` e `"Inativo"`/`"Concluído"`/`"Não"` para `false`/`"Concluded"`).
  - Quando a coluna contém um JSON serializado (objeto ou array de vínculos, como `campus`, `knowledge_areas`, `research_groups`, `team`, `academic_education`), o extrator interpreta o JSON e extrai os campos `"name"`, `"title"`, `"institution"`, `"degree"` ou `"project_code"` de cada item.
  - Nas tabelas SQL colunares (`crud.rs` e `src_domain.rs`), `FilterSpec` pode ser avaliado via predicados SQL parametrizados (`LIKE` / `=` / `!=` / `IS NULL`) combinados por `AND` ou `OR` sobre colunas validadas pelo registry, garantindo performance em `<50ms` mesmo nas 18.334 linhas de `researchers`.
- **Rationale**:
  - Garante que filtrar ou agrupar por `campus = Serra` funcione perfeitamente mesmo quando a coluna `campus` armazena `[{"id": 1, "name": "Serra"}]` no SQLite.
- **Alternatives considered**:
  - Comparar apenas a string JSON bruta com ` equality `: descartado porque exigir que o usuário digite `[{"id": 1, "name": "Serra"}]` para um filtro `equals` inviabilizaria o uso em colunas de vínculos.

## R3. Persistência e Cálculo de Gráficos Personalizados (`custom_dashboard_charts`)

- **Decision**:
  1. Criar a migração `007_custom_dashboard_charts.sql` com a tabela interna `custom_dashboard_charts` (`id`, `scope`, `entity`, `title`, `group_by`, `metric`, `metric_field`, `kind`, `filter_logic`, `filters_json`, `created_at`).
  2. Ao carregar qualquer Dashboard (`horizon_analytics`, `src_analytics`, `egressos_analytics`, `egressos_dados_analytics`), consultar os registros de `custom_dashboard_charts` correspondentes ao `scope`:
     - Se for o **Dashboard Geral** da aba (`entity == None`), carregar **todos** os gráficos personalizados daquele `scope` (independentemente de terem sido criados no Dashboard Geral ou em uma tabela específica daquela aba — Clarificação Q2).
     - Se for o painel **"Ver Gráficos da Tabela"** (`entity == Some(e)`), carregar os gráficos personalizados daquele `scope` e `entity`.
  3. Para cada `CustomDashboardChart`, calcular dinamicamente os `ChartItem`s sobre os dados atuais da tabela `entity` aplicando tanto os filtros próprios do gráfico (`filters_json`) quanto eventuais filtros globais ativos no Dashboard, agrupando por `group_by` e calculando:
     - `metric == "count"`: contagem de registros por categoria;
     - `metric == "sum"`: soma numérica do campo `metric_field` (ou soma da quantidade de itens/vínculos quando `metric_field` é uma lista JSON como `articles`, `initiatives`, `team` ou `advisorships` — Clarificação Q3).
- **Rationale**:
  - Atende 100% às Clarificações Q1, Q2 e Q3, mantendo os gráficos personalizados sempre sincronizados com as edições feitas nas tabelas e totalmente isolados dos arquivos de exportação.
- **Alternatives considered**:
  - Salvar gráficos personalizados no `localStorage` do navegador: descartado porque violaria a centralização da persistência no SQLite local (Princípio V e VI) e dificultaria o cálculo agregado diretamente em Rust.
