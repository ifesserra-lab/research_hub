# Quickstart & Guia de Validação: `040-horizon-analytics-dashboard`

## Pré-requisitos

- Rust 1.77+ (`cargo`)
- Node.js 20+ (`npm`)
- Arquivo `exports_canonical.zip` atualizado na raiz do repositório (`/home/thiagodeps/research_hub/exports_canonical.zip`, 698 entradas, 70.545 registros nas 15 tabelas canônicas)

## 1. Executar Testes Automatizados do Núcleo Rust (`cargo test`)

```bash
cd src-tauri
cargo test
```

**O que é validado**:
1. Migração `006_person_linkedin_url.sql` e paridade entre `registry.rs` (`PERSON_COLUMNS` com 22 colunas incluindo `linkedin_url`) e o schema SQLite (`researchers`, `students`, `hv2_researchers`, `hv2_students`).
2. Importação e round-trip de exportação da versão final do `exports_canonical.zip` (`70.545` registros nas 15 tabelas canônicas, preservação das 22 colunas de pesquisadores/alunos e das `683` entradas não gerenciadas).
3. Comando `projects_status` reportando o status ativo/inativo e contagens dos 5 projetos.
4. Agregações analíticas (`analytics.rs`) para **Horizon**, **Horizon Legacy**, **SRC**, **Egressos** e **Egressos — Dados**, tanto na visão geral quanto por tabela individual (status de projetos ativos vs. inativos, valores de bolsas por programa/orientador/bolsista, instituições de atuação e formação sem ruído de carga horária, titulação e contagens por tabela).

## 2. Executar Testes Automatizados de Front-end (`npm test`)

```bash
cd frontend
npm test
```

**O que é validado**:
1. Mapeamento das novas rotas analíticas (`/projects/status`, `/horizon-v2/analytics`, `/analytics`, `/src/analytics`, `/egressos/analytics`, `/egressos-dados/analytics`) em [api.js](file:///home/thiagodeps/research_hub/frontend/src/services/api.js).
2. Exibição do selo semântico `ProjectStatusBadge` (Ativo / Inativo) nos cartões de `/projects`, nas linhas de `EntityTable` e nos formulários de `EntityForm`.
3. Renderização do `AnalyticsDashboard` nos 5 ambientes de projeto e do painel `TableAnalyticsPanel` ("Ver Gráficos da Tabela") nas abas de tabelas individuais.

## 3. Validação de Build Estático

```bash
cd frontend
npm run build
```

Verifica a compilação estática de todas as páginas Astro (`/projects`, `/horizon/*`, `/dashboard/*`, `/src/*`, `/egressos/*`, `/egressos-dados/*`).
