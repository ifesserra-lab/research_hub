# Phase 0 Research: Aba Horizon (Nova Versão Canônica) e Preservação do Horizon Legacy

**Feature**: `039-horizon-v2-canonical-tab`  
**Date**: 2026-11-12

## R1. Isolamento de Dados entre `Horizon Legacy` e o Novo `Horizon` no SQLite Único

- **Decision**: Manter as 15 tabelas originais (`researchers`, `students`, `articles`, `research_groups`, `initiatives`, `advisorships`, `awards`, `campuses`, `organizations`, `fellowships`, `proficiencies`, `professional_activities`, `knowledge_areas`, `languages`, `research_productions`), o arquivo `original.zip`, o prefixo de snapshot `hub-` e a chave de sincronização `"horizon"` vinculados ao **Horizon Legacy**. Criar na migração `005_horizon_v2_tables.sql` um conjunto paralelo de 15 tabelas prefixadas com `hv2_` (`hv2_researchers`, `hv2_students`, ..., `hv2_research_productions`), acompanhadas de um segundo catálogo `HORIZON_V2_ENTITIES` em `src-tauri/src/registry.rs`, arquivo de referência `horizon_v2_original.zip`, prefixo de snapshot `horizon-v2-` e chave de sincronização `"horizon_v2"`.
- **Rationale**:
  1. Atende diretamente à decisão de clarificação Q3: todos os dados locais, snapshots e configurações preexistentes permanecem imediatamente disponíveis no **Horizon Legacy** sem migração destrutiva.
  2. Preserva 100% de compatibilidade com a suíte de testes existente do legado (`db.rs`, `crud.rs`, `special.rs`, `import.rs`, `export.rs`).
  3. Cumpre a Constituição (Princípio V: SQLite em arquivo único `hub.db`), seguindo exatamente o padrão de separação por tabelas adotado em `SEP-033` (`src_*`), `SEP-035` (`egressos_*`) e `SEP-037` (`egressos_dados_*`).
- **Alternatives Considered**:
  - *Dois arquivos `.db` separados*: descartado porque toda a arquitetura de `AppState` e backup transacional do projeto opera sobre uma conexão central ao `hub.db` com migrações versionadas via `PRAGMA user_version`.
  - *Mover o legado para tabelas renomeadas e deixar as antigas para o novo Horizon*: descartado porque exigiria migração destrutiva de dados existentes e quebraria os testes legados.

## R2. Reuso Sem Duplicação da Lógica de CRUD, Fusão, Vínculo, Importação e Exportação

- **Decision**: Parametrizar as funções internas de `crud.rs`, `special.rs`, `import.rs` e `export.rs` pela fatia de definições de entidades (`&'static [EntityDef]`) e adicionar a `EntityDef` um método `canonical_table(&self) -> &'static str` (que remove o prefixo `"hv2_"` quando presente para obter o nome do arquivo `{tabela}_canonical.json` dentro do `.zip`).
- **Rationale**:
  - O arquivo `.zip` tanto no Horizon Legacy quanto no novo Horizon usa os mesmos nomes de entrada (`researchers_canonical.json`, `initiatives_canonical.json`, etc.), mas grava em tabelas SQLite distintas (`researchers` vs. `hv2_researchers`).
  - Ao adicionar `canonical_table(&self)` em `EntityDef`, o código de leitura/escrita do `.zip` em `import.rs` e `export.rs` busca `{def.canonical_table()}_canonical.json` no pacote e executa `DELETE`/`INSERT`/`SELECT` em `def.table`, reutilizando 100% da lógica de inferência de tipos, preservação byte a byte e transacionalidade.
- **Alternatives Considered**:
  - *Duplicar `import.rs`, `export.rs`, `crud.rs` e `special.rs` em novos módulos*: descartado por violar a manutenibilidade e duplicar regras de negócio em Rust.

## R3. Tratamento dos Novos Arquivos `*_canonical.json` em Formato de Dicionário (`rule_version: 5`)

- **Decision**: Manter a resolução de tabelas gerenciadas guiada exclusivamente pelas 15 entidades declaradas no registry (`ENTITIES` / `HORIZON_V2_ENTITIES`). Os 3 novos arquivos `horizon_ids_canonical.json`, `horizon_id_aliases_canonical.json` e `source_document_states_canonical.json` (cuja raiz é um objeto JSON `{"rule_version": 5, "generated_at": "...", ...}`) permanecem na categoria de entradas não gerenciadas e são copiados byte a byte via `raw_copy_file` durante o export, junto com `export_manifest.json`, `initiatives_analytics_mart.json`, `knowledge_areas_mart.json` e os 297 arquivos de `project_sigpesq_files_json/`.
- **Rationale**:
  - Como `import.rs` e `export.rs` iteram apenas sobre as 15 entidades exportadas do registry (e não sobre qualquer arquivo terminado em `_canonical.json`), `json_io::read_columns` e `json_io::infer_column_types` nunca tentam fazer parse de `horizon_ids_canonical.json` como array, evitando erros de tipo na raiz e garantindo preservação byte a byte (FR-008, Clarificação Q1).
- **Alternatives Considered**:
  - *Importar `horizon_ids`, `horizon_id_aliases` e `source_document_states` para tabelas SQLite*: descartado pela Clarificação Q1 (são artefatos de controle do pipeline ETL, não abas de curadoria humana).

## R4. Exposição Completa de Colunas Canônicas e Renderização de `initiatives.enrichment` na Nova Aba Horizon

- **Decision**:
  1. Nas 15 páginas de `frontend/src/pages/horizon/*.astro`, declarar em `fields` todas as colunas canônicas da respectiva entidade em `HORIZON_V2_ENTITIES`, mantendo as colunas escalares editáveis e as colunas estruturadas (listas de vínculos e o objeto `enrichment`) como `type: 'json_readonly'`.
  2. Atualizar `EntityForm.jsx` para formatar de maneira legível valores aninhados dentro de objetos `json_readonly` sem `id` (como `initiatives.enrichment`, que contém `objetivos: { geral, especificos }`, `cronograma: [...]` e `palavras_chave: [...]`), serializando sub-objetos/listas de forma limpa em vez de `[object Object]`, e aceitar uma prop opcional `basePath` (default `"/dashboard"`, e `"/horizon"` na nova aba) para que os links `openId` naveguem dentro do próprio ambiente (FR-010).
- **Rationale**:
  - Atende à Clarificação Q2 e aos requisitos FR-009 e FR-010 sem quebrar nenhum teste ou tela do `Horizon Legacy`.
- **Alternatives Considered**:
  - *Criar componentes `HorizonEntityPage`, `HorizonEntityForm` e `HorizonEntityTable` duplicados*: descartado; parametrizar `EntityPage`, `EntityForm` e `DataControlCenter` com props opcionais (`apiPrefix`, `basePath`) com defaults idênticos ao comportamento atual mantém os componentes únicos, testáveis e retrocompatíveis.

## R5. Atualização dos Testes de Referência do `exports_canonical.zip`

- **Decision**: Atualizar as expectativas de contagem nos testes que leem o arquivo `exports_canonical.zip` da raiz do repositório (`import::tests::reference_archive_matches_row_counts` e `export::tests::real_archive_round_trip_preserves_types_and_entries`) para refletir o novo pacote de referência já presente no workspace (70.615 registros nas 15 tabelas, 698 entradas no ZIP, 683 entradas preservadas) e adicionar testes equivalentes para o fluxo `horizon_v2`, incluindo verificação explícita do round-trip do objeto `enrichment` em `initiatives`.
- **Rationale**:
  - O arquivo `exports_canonical.zip` na raiz do repositório já foi substituído pelo novo pacote lançado pelo Horizon. Atualizar as asserções da fixture de referência e testar tanto o caminho legado quanto o novo caminho `horizon_v2` garante que `cargo test` valide o pacote real com 100% de precisão (FR-011, SC-002, SC-003).
