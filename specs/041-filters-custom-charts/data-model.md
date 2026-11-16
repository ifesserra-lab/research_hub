# Data Model: Filtros Avançados e Gráficos Personalizados nas 5 Abas

**Feature**: `041-filters-custom-charts`  
**Date**: 2026-11-16

## 1. Migração SQLite (`007_custom_dashboard_charts.sql`)

Tabela interna (`exported: false`) para armazenar os gráficos personalizados criados pelo usuário em qualquer uma das 5 abas (`horizon-v2`, `horizon`, `src`, `egressos`, `egressos-dados`):

```sql
CREATE TABLE IF NOT EXISTS custom_dashboard_charts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scope TEXT NOT NULL,
    entity TEXT NOT NULL,
    title TEXT NOT NULL,
    group_by TEXT NOT NULL,
    metric TEXT NOT NULL DEFAULT 'count',
    metric_field TEXT NULL,
    kind TEXT NOT NULL DEFAULT 'bar',
    filter_logic TEXT NOT NULL DEFAULT 'and',
    filters_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT ''
);

CREATE INDEX IF NOT EXISTS idx_custom_charts_scope_entity
    ON custom_dashboard_charts(scope, entity);
```

---

## 2. Estruturas de Domínio em Rust (`src-tauri/src/analytics.rs`)

### 2.1 `ColumnFilter` e `FilterSpec`

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnFilter {
    pub field: String,
    /// Operadores suportados:
    /// - "contains": contém o texto (case-insensitive, inspeciona nomes em JSONs)
    /// - "equals": igual ao valor (case-insensitive ou correspondência exata de item em JSON)
    /// - "not_equals": diferente do valor
    /// - "is_empty": nulo, string vazia ou array JSON vazio `[]`
    /// - "not_empty": preenchido e diferente de `[]` / `null`
    pub operator: String,
    #[serde(default)]
    pub value: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilterSpec {
    /// "and" (padrão) ou "or" (Clarificação Q1)
    #[serde(default = "default_filter_logic")]
    pub logic: String,
    #[serde(default)]
    pub rules: Vec<ColumnFilter>,
}
```

### 2.2 `CustomDashboardChart` e `NewCustomChartInput`

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomDashboardChart {
    pub id: i64,
    pub scope: String,        // "horizon-v2" | "horizon" | "src" | "egressos" | "egressos-dados"
    pub entity: String,       // ex: "initiatives", "researchers", "src_acoes", "egressos_perfil", "alunos"
    pub title: String,
    pub group_by: String,     // coluna de agrupamento (eixo/categoria)
    pub metric: String,       // "count" | "sum"
    pub metric_field: Option<String>, // coluna somada quando metric == "sum"
    pub kind: String,         // "bar" | "donut" | "columns" | "financial_table"
    pub filter_logic: String, // "and" | "or"
    pub filters: Vec<ColumnFilter>,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewCustomChartInput {
    pub scope: String,
    pub entity: String,
    pub title: String,
    pub group_by: String,
    #[serde(default = "default_metric")]
    pub metric: String,
    pub metric_field: Option<String>,
    #[serde(default = "default_chart_kind")]
    pub kind: String,
    #[serde(default = "default_filter_logic")]
    pub filter_logic: String,
    #[serde(default)]
    pub filters: Vec<ColumnFilter>,
}
```

### 2.3 Extensão de `ChartSection` e `DashboardPayload`

`ChartSection` passa a incluir metadados opcionais para identificar gráficos personalizados e permitir sua exclusão direta no Dashboard:

```rust
pub struct ChartSection {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub kind: String,
    pub items: Vec<ChartItem>,
    #[serde(default)]
    pub is_custom: bool,
    #[serde(default)]
    pub custom_id: Option<i64>,
}
```

`DashboardPayload` passa a incluir os metadados de tabelas e colunas disponíveis no `scope` (`available_entities: Vec<EntityMetaOption>`) para popular os seletores de filtro e o construtor de gráficos personalizados em qualquer uma das 5 abas:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityMetaOption {
    pub entity: String,
    pub label: String,
    pub columns: Vec<String>,
}
```
