# Frontend Routes & API Coupling Contract: `egressos-dados`

**Feature**: `037-egressos-dados-tables`  
**Date**: 2026-11-05  

## 1. Rotas em `frontend/src/services/api.js`

Todas as rotas do front-end para o núcleo Rust passam exclusivamente por `frontend/src/services/api.js`:

| Método e Rota (`apiFetch`) | Comando Tauri (`invoke`) |
|----------------------------|--------------------------|
| `GET /egressos-dados/auth/status` | `egressos_dados_status` |
| `POST /egressos-dados/auth/login` | `egressos_dados_login` |
| `DELETE /egressos-dados/auth/logout` | `egressos_dados_logout` |
| `GET /egressos-dados/workspace` | `egressos_dados_get_workspace` |
| `GET /egressos-dados/catalog` | `egressos_dados_get_catalog` |
| `GET /egressos-dados/files/:fileKey/rows` | `egressos_dados_list_rows` |
| `POST /egressos-dados/files/:fileKey/rows` | `egressos_dados_create_row` |
| `GET /egressos-dados/files/:fileKey/envelope` | `egressos_dados_get_envelope` |
| `PUT /egressos-dados/files/:fileKey/envelope` | `egressos_dados_update_envelope` |
| `GET /egressos-dados/rows/:id` | `egressos_dados_get_row` |
| `PUT /egressos-dados/rows/:id` | `egressos_dados_update_row` |
| `DELETE /egressos-dados/rows/:id` | `egressos_dados_delete_row` |
| `POST /egressos-dados/import` | `egressos_dados_import_dir` |
| `POST /egressos-dados/export` | `egressos_dados_export_dir` |
| `GET /egressos-dados/snapshots` | `egressos_dados_list_snapshots` |
| `GET /egressos-dados/sync/config` | `egressos_dados_sync_get_config` |
| `PUT /egressos-dados/sync/config` | `egressos_dados_sync_set_config` |
| `POST /egressos-dados/sync/download` | `egressos_dados_sync_download` |
| `POST /egressos-dados/sync/upload` | `egressos_dados_sync_upload` |

## 2. Páginas Astro e Componentes React

1. **Menu Lateral (`frontend/src/layouts/EgressosDadosDashboard.astro`)**:
   - Item 1: **Tabelas (39 JSONs)** → `/egressos-dados`
   - Item 2: **Controle de Dados** → `/egressos-dados/controle`
   - Ambos envoltos pela guarda do login especial da organização `ifesserra-lab` (`EgressosDadosWorkspace.jsx`), de modo que nenhuma tabela ou controle apareça sem autorização ativa.
2. **Página de Tabelas (`frontend/src/pages/egressos-dados/index.astro` + `frontend/src/components/EgressosDadosTablesPage.jsx`)**:
   - Filtros rápidos de categoria: `Todos (39)`, `PII / Coorte & Coleta (12)`, `Internos (6)` e `Publicáveis (21)` + busca rápida de arquivo.
   - Seletor das 39 abas exibindo título, caminho (`alunos.json` ou `data/<arquivo>.json`), badge de privacidade (`pii` / `interno` / `publicavel`), badge de papel (`entrada` / `saida`) e contagem de linhas.
   - Sub-abas de seção (ex.: `alunos`, `cursos`, `convencoes` em `alunos.json`), busca, ordenação por coluna, paginação e modais de CRUD e de Metadados do Arquivo.
3. **Página de Controle de Dados (`frontend/src/pages/egressos-dados/controle.astro` + `frontend/src/components/EgressosDadosDataControl.jsx`)**:
   - Importação local do diretório `/home/thiagodeps/egressos-dados` (com botão de preenchimento rápido quando o diretório existir na máquina).
   - Exportação local gravando `alunos.json` na raiz e os 38 arquivos em `data/*.json`.
   - Lista de snapshots `egressos-dados-*`.
   - Painel de Sincronização GitHub pré-configurado com `ifesserra-lab/egressos-dados` (`main`), reutilizando a credencial em memória da sessão ativa de `egressos-dados`.
