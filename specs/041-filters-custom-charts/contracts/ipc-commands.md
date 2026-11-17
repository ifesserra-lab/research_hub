# Contratos IPC (Tauri Commands & `api.js` Routes)

**Feature**: `041-filters-custom-charts`  
**Date**: 2026-11-16

Toda comunicação JS→Rust passa exclusivamente por `frontend/src/services/api.js` (Princípio VI da Constituição).

## 1. Parâmetro `filters` nas Rotas de Listagem de Tabelas (5 Abas)

Todas as rotas de listagem de tabelas passam a aceitar o query parameter opcional `filters` contendo um `FilterSpec` serializado em JSON (`{"logic":"and","rules":[{"field":"status","operator":"equals","value":"Active"}]}`):

1. **Horizon (`horizon-v2`)**:
   - `GET /horizon-v2/{entity}?limit=50&offset=0&search=...&sort=...&order=...&filters={json}`
   - → `invoke('hv2_list_entities', { entity, limit, offset, search, sort, order, filters })`
2. **Horizon Legacy (`horizon`)**:
   - `GET /{entity}?limit=50&offset=0&search=...&sort=...&order=...&filters={json}`
   - → `invoke('list_entities', { entity, limit, offset, search, sort, order, filters })`
3. **SRC (`src`)**:
   - `GET /src/acoes?limit=50&offset=0&search=...&sort=...&order=...&filters={json}`
   - → `invoke('src_list_acoes', { limit, offset, search, sort, order, filters })`
4. **Egressos (`egressos`)**:
   - `GET /egressos/files/{fileKey}/rows?section=...&limit=50&offset=0&search=...&sort=...&order=...&filters={json}`
   - → `invoke('egressos_list_rows', { fileKey, sectionKey, limit, offset, search, sort, order, filters })`
5. **Egressos — Dados (`egressos-dados`)**:
   - `GET /egressos-dados/files/{fileKey}/rows?section=...&limit=50&offset=0&search=...&sort=...&order=...&filters={json}`
   - → `invoke('egressos_dados_list_rows', { fileKey, sectionKey, limit, offset, search, sort, order, filters })`

---

## 2. Parâmetro `filters` nas Rotas de Dashboards Analíticos (5 Abas)

Todas as 5 rotas de analytics passam a aceitar o query parameter opcional `filters` (`FilterSpec` serializado em JSON), recalculando KPIs e gráficos sobre o subconjunto filtrado e incluindo os gráficos personalizados salvos:

- `GET /horizon-v2/analytics?entity={opt}&filters={opt}` → `invoke('hv2_get_analytics', { entity, filters })`
- `GET /analytics?entity={opt}&filters={opt}` → `invoke('get_horizon_analytics', { entity, filters })`
- `GET /src/analytics?filters={opt}` → `invoke('src_get_analytics', { filters })`
- `GET /egressos/analytics?file={opt}&filters={opt}` → `invoke('egressos_get_analytics', { fileKey, filters })`
- `GET /egressos-dados/analytics?file={opt}&filters={opt}` → `invoke('egressos_dados_get_analytics', { fileKey, filters })`

---

## 3. Comandos de Gráficos Personalizados (`/custom-charts`)

### 3.1 Criar Novo Gráfico Personalizado
- **Rota**: `POST /custom-charts`
- **Tauri Command**: `invoke('create_custom_chart', { payload })`
- **Request Body (`NewCustomChartInput`)**:
  ```json
  {
    "scope": "horizon-v2",
    "entity": "initiatives",
    "title": "Projetos de Serra por Status",
    "group_by": "status",
    "metric": "count",
    "metric_field": null,
    "kind": "donut",
    "filter_logic": "and",
    "filters": [
      { "field": "campus", "operator": "contains", "value": "Serra" }
    ]
  }
  ```
- **Response (`CustomDashboardChart`)**: Registro persistido com `id` atribuído.

### 3.2 Listar Gráficos Personalizados de uma Aba / Tabela
- **Rota**: `GET /custom-charts?scope={scope}&entity={opt}`
- **Tauri Command**: `invoke('list_custom_charts', { scope, entity })`
- **Response**: `Vec<CustomDashboardChart>`

### 3.3 Excluir Gráfico Personalizado
- **Rota**: `DELETE /custom-charts/{id}`
- **Tauri Command**: `invoke('delete_custom_chart', { id })`
- **Response**: `null`
