# Contratos IPC (Tauri Commands & `api.js` Routes)

**Feature**: `040-horizon-analytics-dashboard`  
**Date**: 2026-11-16

Toda comunicação entre os componentes React e o processo Rust ocorre exclusivamente através de `apiFetch` em `frontend/src/services/api.js` (Princípio VI da Constituição).

## 1. Status Ativo/Inativo dos 5 Projetos (`/projects`)

### `GET /projects/status` → `invoke('projects_status')`

Retorna a lista de status dos 5 ambientes de projeto (`horizon-v2`, `horizon`, `src`, `egressos`, `egressos-dados`) para exibição dos selos de projeto **Ativo / Sem dados** nos cartões da tela `/projects`.

**Response (`Vec<ProjectWorkspaceStatus>`)**:
```json
[
  {
    "id": "horizon-v2",
    "active": true,
    "total_rows": 70545,
    "active_projects": 1943,
    "status_label": "Ativo • 70.545 registros (1.943 projetos ativos)"
  },
  {
    "id": "horizon",
    "active": false,
    "total_rows": 0,
    "active_projects": 0,
    "status_label": "Inativo • Sem dados carregados"
  },
  {
    "id": "src",
    "active": true,
    "total_rows": 120,
    "active_projects": null,
    "status_label": "Ativo • 120 registros"
  },
  {
    "id": "egressos",
    "active": true,
    "total_rows": 1450,
    "active_projects": null,
    "status_label": "Ativo • 1.450 registros (21 arquivos)"
  },
  {
    "id": "egressos-dados",
    "active": true,
    "total_rows": 3200,
    "active_projects": null,
    "status_label": "Ativo • 3.200 registros (39 datasets)"
  }
]
```

---

## 2. Dashboards Gerais e Gráficos por Tabela

### 2.1 Horizon (Nova Versão Canônica `hv2_*`)

- **Dashboard Geral**: `GET /horizon-v2/analytics` → `invoke('hv2_get_analytics', { entity: null })`
- **Gráficos de uma Tabela Específica**: `GET /horizon-v2/analytics?entity={route}` → `invoke('hv2_get_analytics', { entity: route })`
  - Onde `{route}` é uma das 15 rotas canônicas: `researchers`, `students`, `initiatives`, `advisorships`, `fellowships`, `articles`, `research_productions`, `groups`, `professional_activities`, `awards`, `organizations`, `campuses`, `knowledge_areas`, `proficiencies`, `languages`.

### 2.2 Horizon Legacy (Tabelas Canônicas Originais)

- **Dashboard Geral**: `GET /analytics` → `invoke('get_horizon_analytics', { entity: null })`
- **Gráficos de uma Tabela Específica**: `GET /analytics?entity={route}` → `invoke('get_horizon_analytics', { entity: route })`

### 2.3 SRC (`/src` e `/src/acoes`)

- **Dashboard Geral e Gráficos de Ações/Participações**: `GET /src/analytics` → `invoke('src_get_analytics')`

### 2.4 Egressos (`/egressos` e 21 tabelas de `dados/`)

- **Dashboard Geral**: `GET /egressos/analytics` → `invoke('egressos_get_analytics', { fileKey: null })`
- **Gráficos de uma Tabela Específica**: `GET /egressos/analytics?file={file_key}` → `invoke('egressos_get_analytics', { fileKey: file_key })`

### 2.5 Egressos — Dados (`/egressos-dados` e 39 datasets)

- **Dashboard Geral**: `GET /egressos-dados/analytics` → `invoke('egressos_dados_get_analytics', { fileKey: null })`
- **Gráficos de um Dataset Específico**: `GET /egressos-dados/analytics?file={file_key}` → `invoke('egressos_dados_get_analytics', { fileKey: file_key })`

---

## 3. Formato de Resposta Unificado (`DashboardPayload`)

Todos os 5 comandos analíticos retornam a estrutura `DashboardPayload`:

```json
{
  "scope": "horizon-v2",
  "entity": null,
  "title": "Dashboard Analítico — Horizon",
  "loaded": true,
  "total_rows": 70545,
  "kpis": [
    {
      "label": "Projetos Ativos",
      "value": "1.943",
      "sublabel": "12,5% de 15.546 iniciativas",
      "tone": "emerald"
    }
  ],
  "charts": [
    {
      "id": "project_status",
      "title": "Status dos Projetos (Iniciativas)",
      "subtitle": "Proporção de projetos ativos vs. inativos/concluídos",
      "kind": "donut",
      "items": [
        {
          "label": "Ativo (Em Execução)",
          "count": 1084,
          "secondary_value": null,
          "formatted_value": "1.084 projetos",
          "percentage": 6.97
        }
      ]
    }
  ]
}
```
