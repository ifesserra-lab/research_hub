# Quickstart & Validation Guide: Espaço de Dados do Projeto Egressos (`035-egressos-data-workspace`)

**Date**: 2026-10-22 | **Feature**: [spec.md](./spec.md)

## 1. Pré-requisitos

- Ambiente Rust (`cargo`) e Node.js (`npm`) configurados no repositório `/home/thiagodeps/research_hub`.
- Pasta de dados de referência disponível em `/home/thiagodeps/egressos/dados` contendo os 21 arquivos `.json`.

## 2. Execução da Suíte Automatizada (TDD — Princípios I e III)

### 2.1. Testes do Núcleo Rust (`cargo test`)

```bash
cd /home/thiagodeps/research_hub/src-tauri
cargo test
```

**O que os testes verificam**:
1. **Migração `003_egressos_tables.sql`**: criação das tabelas `egressos_files` e `egressos_rows` sem afetar as tabelas do Horizon ou do SRC.
2. **Importação e Validação da Pasta `dados/` (`FR-004`, `FR-005`, `FR-006`)**:
   - Importação completa dos 21 arquivos JSON de `/home/thiagodeps/egressos/dados` (ou fixture equivalente), verificando contagens de arquivos (21) e registros.
   - Falha atômica quando um dos 21 arquivos está ausente ou malformado, mantendo a base anterior intacta.
   - Geração de snapshot `egressos-pre-import-*.db` ao reimportar sobre base existente.
3. **Paridade Round-Trip (`SC-003`, `FR-010`)**:
   - Importar os 21 arquivos JSON → exportar para pasta temporária sem edições → comparar cada um dos 21 arquivos `serde_json::Value` com o original (100% idênticos).
4. **CRUD Completo e Propagação Opcional (`FR-007`..`FR-009`, `FR-008a`)**:
   - Listar, buscar, ordenar, criar, editar e excluir registros tanto em arquivos `object_sections` (ex.: `egressos_perfil`, `vitrine`) quanto em arquivos `root_map` (ex.: `empresas_porte`) e `nested_map` (ex.: `codigofonte_historico`, `ibge_series`).
   - Edição com `propagate = false` (altera apenas o arquivo atual) vs `propagate = true` (propaga campos equivalentes do egresso pelo `rotulo`/`label` ou da empresa pelo nome para os arquivos relacionados).
   - Edição do envelope de metadados (`egressos_update_envelope`) refletida na exportação.
5. **Sincronização GitHub de Diretório (`FR-011`..`FR-013` via `wiremock`)**:
   - Download dos 21 arquivos `.json` da pasta `dados/` via Contents API (com e sem token) e importação atômica.
   - Upload dos 21 arquivos `.json` via Git Data API (`blobs` → `tree` única com 21 entradas → `commit` único → atualização da `ref` da branch), validando bloqueio quando a branch não existe e ausência de vazamento do token em mensagens de erro.
6. **Resolução de Rotas (`lib.rs`)**:
   - Todas as 23 rotas em `ROUTES` (incluindo `/egressos` e `/egressos/tabelas`) resolvem no asset resolver do binário Tauri.

### 2.2. Testes do Front-end (`npm test`)

```bash
cd /home/thiagodeps/research_hub/frontend
npm test
```

**O que os testes verificam**:
- `ProjectSelector`: exibe os 3 projetos (**Horizon**, **SRC** e **Egressos**) e navega para `/egressos`.
- `api.js`: mapeia todas as rotas `/egressos/*` para os comandos Tauri correspondentes e suporta `project: "egressos"` nas rotas `/github/*`.
- `EgressosDataControl` & `EgressosTablesPage`: navegação entre as 21 abas, sub-abas de seção, CRUD, caixa de seleção de propagação (`FR-008a`), edição de metadados e integração com `SyncPanel` (`allowUpload={true}`).

## 3. Validação Manual Rápida no Aplicativo Desktop

1. Iniciar o aplicativo (`cargo tauri dev` ou binário compilado) e fazer login (`admin@admin.com` / `admin123`).
2. Na tela `/projects`, clicar no cartão **Egressos**.
3. Na página `/egressos` (Resumo), clicar em **Importar pasta `dados/`** (que sugere `/home/thiagodeps/egressos/dados`) e confirmar que os 21 arquivos são carregados.
4. Abrir **Tabelas (`/egressos/tabelas`)**, navegar pelas 21 abas, testar busca/ordenação e editar um registro testando a opção de propagação entre tabelas.
5. Configurar o token do GitHub e o repositório (`thiagodeps/egressos`, branch e pasta `dados`) no painel de sincronização para testar download e upload.
