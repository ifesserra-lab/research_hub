# Data Model: Adaptação Final do Pacote Canônico Horizon, Indicador de Status e Dashboards Analíticos

**Feature**: `040-horizon-analytics-dashboard`  
**Date**: 2026-11-16

## 1. Alteração de Esquema SQLite (`006_person_linkedin_url.sql`)

Para alinhar o banco local à versão final do pacote `exports_canonical.zip`, as tabelas de pessoas (`researchers` e `students` no Horizon Legacy e `hv2_researchers` e `hv2_students` no novo Horizon) passam de 21 para **22 colunas**:

```sql
ALTER TABLE researchers ADD COLUMN linkedin_url TEXT;
ALTER TABLE students ADD COLUMN linkedin_url TEXT;
ALTER TABLE hv2_researchers ADD COLUMN linkedin_url TEXT;
ALTER TABLE hv2_students ADD COLUMN linkedin_url TEXT;
```

### Catálogo `PERSON_COLUMNS` em `src-tauri/src/registry.rs` (22 colunas)

| # | Coluna | Tipo SQLite | Tipo JSON Canônico |
|---|---|---|---|
| 1 | `id` | `INTEGER PRIMARY KEY` | `integer` |
| 2 | `name` | `TEXT` | `string` |
| 3 | `identification_id` | `TEXT` | `string \| null` |
| 4 | `birthday` | `TEXT` | `string \| null` |
| 5 | `cnpq_url` | `TEXT` | `string \| null` |
| 6 | `google_scholar_url` | `TEXT` | `string \| null` |
| 7 | `resume` | `TEXT` | `string \| null` |
| 8 | `citation_names` | `TEXT` | `string \| null` |
| 9 | `linkedin_url` *(NOVO)* | `TEXT` | `string \| null` |
| 10 | `initiatives` | `TEXT` | `array` |
| 11 | `research_groups` | `TEXT` | `array` |
| 12 | `knowledge_areas` | `TEXT` | `array` |
| 13 | `academic_education` | `TEXT` | `array` |
| 14 | `articles` | `TEXT` | `array` |
| 15 | `advisorships` | `TEXT` | `array` |
| 16 | `classification` | `TEXT` | `string \| null` |
| 17 | `classification_confidence` | `TEXT` | `number \| null` |
| 18 | `classification_note` | `TEXT` | `string \| null` |
| 19 | `role_evidence` | `TEXT` | `array \| object \| null` |
| 20 | `was_student` | `TEXT` | `boolean` |
| 21 | `was_staff` | `TEXT` | `boolean` |
| 22 | `campus` | `TEXT` | `array \| object \| null` |

---

## 2. Contagens de Referência da Versão Final de `exports_canonical.zip`

- **Total de entradas no arquivo `.zip`**: `698` (`15` tabelas canônicas gerenciadas + `683` entradas preservadas byte a byte)
- **Total de registros nas 15 tabelas canônicas**: `70.545`
  - `researchers`: `18.334`
  - `initiatives`: `15.546`
  - `students`: `11.717`
  - `professional_activities`: `9.560`
  - `articles`: `8.079`
  - `research_productions`: `3.732`
  - `knowledge_areas`: `1.563`
  - `proficiencies`: `794`
  - `organizations`: `440`
  - `research_groups`: `353`
  - `advisorships`: `190`
  - `awards`: `183`
  - `campuses`: `23`
  - `fellowships`: `20`
  - `languages`: `11`

---

## 3. Regra de Classificação de Status de Projeto (`ProjectStatus`)

Aplicada aos registros de `initiatives` e `advisorships` (tanto em `Horizon` quanto em `Horizon Legacy`):

| Valor Canônico (`status`) | Macro-Status (`is_active`) | Rótulo Visual (`badge_label`) | Estilo Visual (`tone`) |
|---|---|---|---|
| `"Active"` | `true` (**Ativo**) | `Ativo (Em Execução)` | `emerald` (verde) |
| `"In Progress"` | `true` (**Ativo**) | `Ativo (Em Andamento)` | `teal` (verde-azulado) |
| `"Concluded"` | `false` (**Inativo**) | `Inativo (Concluído)` | `slate` (cinza-azulado) |
| `"Cancelled"` | `false` (**Inativo**) | `Inativo (Cancelado)` | `rose` (vermelho) |
| `"Unknown"`, `"N/A"`, `""`, `null` | `false` (**Inativo**) | `Inativo (Não Informado)` | `amber` (neutro/alerta) |

---

## 4. Estruturas de Dados Analíticas (Rust → Front-end IPC)

### 4.1 `ProjectWorkspaceStatus` (`GET /projects/status`)

Representa o estado de cada um dos 5 projetos na tela `/projects`:

```json
{
  "id": "horizon-v2",
  "active": true,
  "total_rows": 70545,
  "active_projects": 1943,
  "status_label": "Ativo • 70.545 registros (1.943 projetos ativos)"
}
```

### 4.2 Blocos Comuns de Visualização (`KpiCard`, `ChartItem`, `ChartSection`)

```rust
pub struct KpiCard {
    pub label: String,
    pub value: String,
    pub sublabel: Option<String>,
    pub tone: String, // "indigo" | "emerald" | "amber" | "rose" | "slate" | "sky"
}

pub struct ChartItem {
    pub label: String,
    pub count: usize,
    pub secondary_value: Option<f64>,
    pub formatted_value: Option<String>,
    pub percentage: f64,
}

pub struct ChartSection {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub kind: String, // "bar" | "donut" | "columns" | "financial_table"
    pub items: Vec<ChartItem>,
}
```

### 4.3 `DashboardPayload` (Retornado para os 5 Projetos e para cada Tabela Individual)

Estrutura unificada para renderização tanto no `AnalyticsDashboard.jsx` (visão consolidada de cada um dos 5 projetos) quanto no `TableAnalyticsPanel.jsx` (visão "Ver Gráficos da Tabela" em qualquer tabela):

```rust
pub struct DashboardPayload {
    pub scope: String,         // ex: "horizon-v2", "horizon", "src", "egressos", "egressos-dados"
    pub entity: Option<String>,// None para Dashboard Geral, Some(route/file_key) para tabela específica
    pub title: String,
    pub loaded: bool,
    pub total_rows: usize,
    pub kpis: Vec<KpiCard>,
    pub charts: Vec<ChartSection>,
}
```

### 4.4 Seções Obrigatórias no Dashboard Geral do Horizon / Horizon Legacy

1. **KPIs Principais**:
   - Total de Registros Canônicos (nas 15 tabelas)
   - Projetos / Iniciativas Ativas (`Active` + `In Progress`, com % sobre o total de iniciativas)
   - Projetos Concluídos / Inativos (`Concluded` + `Cancelled` + `Unknown`)
   - Total de Pesquisadores e Total de Alunos
   - Total de Grupos de Pesquisa e Produção Científica (`articles` + `research_productions`)
   - Investimento Mensal em Bolsas (`R$` somado das orientações remuneradas + total de bolsas remuneradas vs. voluntárias)
2. **Gráficos**:
   - `project_status`: Status dos Projetos / Iniciativas (Ativos em Execução, Ativos em Andamento, Concluídos, Cancelados, Não Informados) — `donut`
   - `advisorship_status`: Status dos Projetos de Orientação (Ativos vs. Concluídos) — `donut`
   - `table_counts`: Quantidade de Registros nas 15 Tabelas Canônicas — `bar`
   - `fellowship_programs`: Distribuição Financeira e Quantidade de Bolsas por Programa (`PIBIC`, `PIBITI`, `PIBIC-JR`, `PROPÓS`, `PIVIC`, etc.) — `financial_table`
   - `fellowship_catalog`: Valores Unitários das Modalidades de Bolsa (`fellowships`) — `bar`
   - `top_supervisors`: Quanto Movimentam/Supervisionam os Pesquisadores Orientadores (Top Orientadores por Valor Mensal de Bolsas e Nº de Bolsistas) — `financial_table`
   - `top_recipients`: Quanto Recebem os Bolsistas (Top Bolsistas por Valor Mensal e Bolsas Recebidas) — `financial_table`
   - `professional_institutions`: De Quais Instituições Vêm / Atuam os Pesquisadores (Vínculos em `professional_activities`) — `bar`
   - `education_institutions`: Instituições de Formação Acadêmica dos Pesquisadores (`academic_education`) — `bar`
   - `researcher_degrees`: Distribuição de Pesquisadores por Maior Titulação Acadêmica (`Doutorado`, `Mestrado`, `Especialização`, `Graduação`, etc.) — `donut`
   - `production_by_year`: Evolução Anual de Artigos e Produções Científicas — `columns`
   - `initiatives_by_campus`: Distribuição de Projetos e Grupos por Campus — `bar`
   - `top_knowledge_areas`: Principais Áreas do Conhecimento — `bar`
