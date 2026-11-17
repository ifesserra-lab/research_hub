# Quickstart & Validation Guide: Aba Horizon (Nova Versão Canônica) e Horizon Legacy

**Feature**: `039-horizon-v2-canonical-tab`  
**Date**: 2026-11-12

## Pré-requisitos

- Rust toolchain (`cargo` / `rustc`) e dependências de sistema Tauri 2.0 (Linux/Windows)
- Node.js 20+ e `npm`
- Pacote de referência `exports_canonical.zip` na raiz do repositório (`/home/thiagodeps/research_hub/exports_canonical.zip`)

## 1. Executar a Suíte de Testes do Backend (Rust)

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

**Resultados Esperados**:
- `db::tests::migration_creates_nineteen_tables` (atualizado para validar as 15 novas tabelas `hv2_*` da migração `005_horizon_v2_tables.sql`, totalizando 38 tabelas).
- `registry::tests::horizon_v2_routes_resolve_and_never_collide` e validação de sincronia entre as páginas `frontend/src/pages/dashboard/*.astro` (Horizon Legacy) e `frontend/src/pages/horizon/*.astro` (Novo Horizon).
- `import::tests::reference_archive_matches_row_counts` e `import::tests::horizon_v2_imports_isolated_from_legacy`: validam a importação das **70.615 linhas** distribuídas nas 15 tabelas canônicas do novo `exports_canonical.zip` e o isolamento total entre `Horizon Legacy` e `Horizon`.
- `export::tests::real_archive_round_trip_preserves_types_and_entries` e `export::tests::horizon_v2_round_trip_preserves_enrichment_and_entries`: validam as **698 entradas totais**, **683 entradas preservadas byte a byte** e a restauração do objeto `enrichment` em `initiatives_canonical.json`.
- `sync_domain::tests` e `sync_github`: validam a chave `"horizon_v2"` independente de `"horizon"`.

## 2. Executar a Suíte de Testes do Frontend (Vitest) e Build Estático

```bash
cd frontend
npm test -- --run
npm run build
```

**Resultados Esperados**:
- Testes unitários de `ProjectSelector.test.jsx` verificam a presença do novo card **Horizon** (`/horizon`) e a renomeação do card anterior para **Horizon Legacy** (`/dashboard`).
- Testes unitários de `api.test.js` verificam o mapeamento de todas as rotas `/horizon-v2/*` para os comandos `hv2_*`, `import_horizon_v2_zip` e `export_horizon_v2_zip`.
- Testes de `EntityForm` / `EntityPage` verificam a renderização legível do objeto `enrichment` e a navegação de vínculos com `basePath="/horizon"`.
- Após `npm run build`, `cargo test --manifest-path src-tauri/Cargo.toml todas_as_rotas_do_menu_resolvem_com_o_href_atual` comprova que todas as rotas de `/dashboard/*` e `/horizon/*` resolvem no binário Tauri.

## 3. Verificação Funcional Manual (Opcional)

1. Iniciar o aplicativo e autenticar-se.
2. Na tela `/projects`, confirmar a exibição de **Horizon** (novo) e **Horizon Legacy** (antigo).
3. Abrir **Horizon** (`/horizon`), importar `exports_canonical.zip` e conferir os 70.615 registros carregados em 15 tabelas.
4. Acessar `4. Iniciativas` (`/horizon/initiatives`), abrir uma iniciativa enriquecida pelo SIGPesq e verificar a exibição estruturada do campo `Enrichment (SIGPesq)` e de todos os demais atributos da tabela.
5. Abrir **Horizon Legacy** (`/dashboard`) e confirmar que sua base permaneceu isolada.
