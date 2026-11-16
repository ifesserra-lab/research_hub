# Quickstart & Guia de Validação: `041-filters-custom-charts`

## Pré-requisitos

- Rust 1.77+ (`cargo`)
- Node.js 20+ (`npm`)

## 1. Executar Testes Automatizados do Núcleo Rust (`cargo test`)

```bash
cd src-tauri
cargo test
```

**O que é validado**:
1. Migração `007_custom_dashboard_charts.sql` criando a tabela `custom_dashboard_charts` sem impactar nenhuma das 15 tabelas canônicas exportadas.
2. Avaliação de `FilterSpec` (`AND` e `OR`) com todos os operadores (`contains`, `equals`, `not_equals`, `is_empty`, `not_empty`) sobre colunas escalares e colunas JSON estruturadas (`campus`, `knowledge_areas`, `academic_education`).
3. Listagens paginadas filtradas em `crud.rs` (Horizon e Horizon Legacy), `src_domain.rs` (SRC), `egressos_domain.rs` (Egressos) e `egressos_dados_domain.rs` (Egressos — Dados).
4. Recálculo dos Dashboards das 5 abas com `filters` aplicados.
5. Criação, listagem, cálculo (`count` e `sum`) e exclusão de gráficos personalizados (`CustomDashboardChart`) visíveis tanto na tabela de origem quanto no Dashboard Geral da aba.

## 2. Executar Testes Automatizados de Front-end (`npm test`)

```bash
cd frontend
npm test
```

**O que é validado**:
1. Mapeamento do query parameter `filters` nas rotas de listagem e analytics e das rotas `/custom-charts` em [api.js](file:///home/thiagodeps/research_hub/frontend/src/services/api.js).
2. Adição/remoção de filtros por coluna, alternância `E (AND)` / `OU (OR)` e botão **"Criar Gráfico com estes Filtros"** em `FilterBuilderBar.jsx`.
3. Criação e exclusão de gráficos personalizados e aplicação de filtros no Dashboard em `AnalyticsDashboard.jsx` e `CustomChartModal.jsx`.

## 3. Build Estático

```bash
cd frontend
npm run build
```
