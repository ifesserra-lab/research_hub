# Contract: Rotas Front-end e Ponte `api.js` (`frontend-routes.md`)

**Date**: 2026-10-22 | **Feature**: [spec.md](../spec.md)

## 1. Páginas Astro (`frontend/src/pages/` e `ROUTES` em `src-tauri/src/lib.rs`)

Duas novas rotas de página estática são adicionadas para a área dedicada do projeto **Egressos** (`ROUTES` em `lib.rs` passa de 21 para 23 entradas):

| Rota | Arquivo Astro | Descrição |
|------|---------------|-----------|
| `/egressos` | `frontend/src/pages/egressos/index.astro` | Página inicial da área Egressos: resumo da base carregada (arquivos, total de registros, última importação), importação/exportação da pasta local `dados/` (`EgressosDataControl.jsx`) e sincronização de download/upload com o GitHub via token (`SyncPanel.jsx` com `project="egressos" allowUpload={true}`). |
| `/egressos/tabelas` | `frontend/src/pages/egressos/tabelas.astro` | Página de curadoria das **21 abas de tabelas** do Egressos (`EgressosTablesPage.jsx`), com navegação 1:1 para os 21 arquivos JSON (`?tab=<file_key>`), sub-abas para seções secundárias, busca, ordenação, CRUD completo, opção de propagação (`FR-008a`) e editor de metadados do arquivo (`FR-009`). |

Na página `/projects` (`frontend/src/pages/projects.astro`), a lista `PROJECTS` passa a incluir o terceiro cartão:
```js
{
  id: 'egressos',
  title: 'Egressos',
  description: 'Painel de carreira e renda dos egressos do IFES Campus Serra (21 tabelas JSON da pasta dados/)',
  href: '/egressos',
}
```

## 2. Rotas na Ponte Única `frontend/src/services/api.js`

Todas as rotas `/egressos/*` são declaradas antes dos padrões genéricos `/{entity}` em `api.js`:

| Método | Padrão de Endpoint | Comando Tauri Invocado |
|--------|--------------------|------------------------|
| `POST` | `/egressos/import` | `invoke('import_egressos_dir', { path: b?.path ?? null })` |
| `POST` | `/egressos/export` | `invoke('export_egressos_dir', { path: b?.path ?? null })` |
| `GET` | `/egressos/catalog` | `invoke('egressos_get_catalog')` |
| `GET` | `/egressos/files/:fileKey/envelope` | `invoke('egressos_get_envelope', { fileKey: m[1] })` |
| `PUT` | `/egressos/files/:fileKey/envelope` | `invoke('egressos_update_envelope', { fileKey: m[1], payload: b ?? {} })` |
| `GET` | `/egressos/files/:fileKey/rows` | `invoke('egressos_list_rows', { fileKey: m[1], sectionKey: q.get('section'), limit, offset, search, sort, order })` |
| `POST` | `/egressos/files/:fileKey/rows` | `invoke('egressos_create_row', { fileKey: m[1], sectionKey: b?.section_key ?? q.get('section'), payload: b?.payload ?? b ?? {} })` |
| `GET` | `/egressos/rows/:id` | `invoke('egressos_get_row', { id: Number(m[1]) })` |
| `PUT` | `/egressos/rows/:id` | `invoke('egressos_update_row', { id: Number(m[1]), payload: b?.payload ?? b ?? {}, propagate: Boolean(b?.propagate) })` |
| `DELETE` | `/egressos/rows/:id` | `invoke('egressos_delete_row', { id: Number(m[1]), propagate: q.get('propagate') === 'true' })` |
