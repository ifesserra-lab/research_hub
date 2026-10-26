# Research: Espaço de Dados do Projeto Egressos com CRUD e Sincronização GitHub (`035-egressos-data-workspace`)

**Date**: 2026-10-22 | **Feature**: [spec.md](./spec.md)

## R1 — Armazenamento no SQLite para 21 arquivos JSON heterogêneos

- **Decision**: Criar a migração aditiva `003_egressos_tables.sql` no mesmo banco `hub.db` com duas tabelas dedicadas ao domínio Egressos:
  1. `egressos_files (file_key TEXT PRIMARY KEY, kind TEXT NOT NULL, primary_section TEXT NOT NULL, envelope_json TEXT NOT NULL, imported_at TEXT)` — armazena o envelope (metadados escalares, notas metodológicas, objetos auxiliares e descritores de seção com ordem original das chaves via `serde_json` + `preserve_order`).
  2. `egressos_rows (id INTEGER PRIMARY KEY AUTOINCREMENT, file_key TEXT NOT NULL REFERENCES egressos_files(file_key) ON DELETE CASCADE, section_key TEXT NOT NULL, ord INTEGER NOT NULL, record_key TEXT, search_text TEXT NOT NULL, raw_json TEXT NOT NULL)` — armazena cada registro de cada lista ou mapa dos 21 arquivos JSON.
- **Rationale**:
  - Os 21 arquivos em `/home/thiagodeps/egressos/dados` possuem estruturas distintas (14 objetos com uma ou várias listas + metadados, 4 mapas raiz indexados por nome de empresa e 3 objetos com mapas/séries aninhados). Criar 50+ tabelas físicas rígidas no SQLite acoplaria o schema a cada sub-lista e impediria preservar campos desconhecidos/futuros.
  - Com `egressos_files` + `egressos_rows`, a busca (`search_text LIKE`), ordenação, paginação e CRUD operam em SQLite com transações ACID e índices em `(file_key, section_key, ord)` e `(record_key)`, enquanto `raw_json` + `envelope_json` garantem reconstrução 100% sem perdas (*round-trip*, SC-003).
  - Isolamento total em relação às 15 tabelas do Horizon e às 3 tabelas `src_*` do SRC (`FR-002`, `FR-003`).
- **Alternatives considered**:
  - *Um arquivo SQLite separado para Egressos*: rejeitado pelo mesmo motivo de `033-src-data-tab` (R1) — `AppState` gerencia uma conexão principal para `hub.db` e o isolamento por prefixo de tabela `egressos_*` e comandos IPC dedicados `egressos_*` mantém a arquitetura simples.
  - *Uma tabela física por arquivo JSON (21+ tabelas)*: rejeitado porque vários arquivos (`vitrine.json`, `analise.json`, `impacto.json`, `trajetoria.json`) contêm de 4 a 13 listas diferentes cada um, além de metadados no topo.

## R2 — Catálogo canônico dos 21 arquivos de `/home/thiagodeps/egressos/dados` e navegação 1:1

- **Decision**: Declarar um catálogo estático `EGRESSOS_FILES: &[EgressosFileDef; 21]` em `src-tauri/src/egressos_domain.rs` mapeando 1 para 1 os 21 arquivos `.json` da pasta `dados/` (Clarification Q1):
  1. `analise.json` (`analise`) — Primária: `rows` (50); Sub-seções: `top_tech`, `funcoes`, `metodos`, `intl_timeline`, `trajetoria_destaque`, `hist_multiplicadores`.
  2. `cargos_ao_longo_do_tempo.json` (`cargos_ao_longo_do_tempo`) — Primária: `cargos` (28); Sub-seções: `edicoes`.
  3. `codigofonte_2026.json` (`codigofonte_2026`) — Primária: `por_senioridade` (5); Sub-seções: `por_contrato`, `por_regiao_top10`, `por_linguagem`, `recortes_indisponiveis_na_pagina`, `observacoes`.
  4. `codigofonte_historico.json` (`codigofonte_historico`) — Primária (mapa): `media_por_senioridade` (6 anos); Sub-seção (mapa): `ipca_fator_para_2026`.
  5. `consolidado.json` (`consolidado`) — Primária: `perfis` (49); Sub-seções: `agregado`, `pv`, `por_ano_sm`, `series_por_perfil` (mapa).
  6. `egressos_perfil.json` (`egressos_perfil`) — Primária: `egressos` (50); Sub-seções: `cursos` (mapa).
  7. `empresas_aliases.json` (`empresas_aliases`) — Primária (mapa raiz `_root`): 207 empresas.
  8. `empresas_linkedin_data.json` (`empresas_linkedin_data`) — Primária (mapa raiz `_root`): 46 empresas.
  9. `empresas_linkedin_urls.json` (`empresas_linkedin_urls`) — Primária (mapa raiz `_root`): 38 empresas.
  10. `empresas_porte.json` (`empresas_porte`) — Primária (mapa raiz `_root`): 204 empresas.
  11. `fapes_fomento.json` (`fapes_fomento`) — Primária: `projetos` (2).
  12. `ibge_series.json` (`ibge_series`) — Primária (mapa): `series` (`ipca`, `inpc`, `salario_minimo`, `cambio`).
  13. `impacto.json` (`impacto`) — Primária: `indicadores` (7); Sub-seções: `por_experiencia`, `por_experiencia_hoje`, `achados`, `ressalvas`, `dispersao`, `vias_de_formacao`, `internacional_no_tempo`, `funcoes`, `metodos`, `tecnologias`, `senioridade`, `perfis_agrupados`.
  14. `mapa_mundi.json` (`mapa_mundi`) — Primária: `paths` (113); Sub-seções: `viewBox`, `lat_range`.
  15. `panorama.json` (`panorama`) — Primária: `coorte` (50); Sub-seções: `salto`, `areas`, `familias`, `origem_comum`.
  16. `recorte_cursos.json` (`recorte_cursos`) — Primária (mapa): `cursos` (`bsi`, `eca`).
  17. `renda_por_senioridade.json` (`renda_por_senioridade`) — Primária: `cargos` (5); Sub-seções: `niveis_sem_referencia_do_modelo`.
  18. `salario_minimo.json` (`salario_minimo`) — Primária: `por_ano` (16); Sub-seções: `notas`.
  19. `so_benchmarks.json` (`so_benchmarks`) — Primária: `por_pais` (12); Sub-seções: `serie_anual`, `por_moeda_brasil`, `candidatas_a_referencia`, `notas`.
  20. `trajetoria.json` (`trajetoria`) — Primária: `serie` (15); Sub-seções: `trajetoria`, `trajetoria_coorte`, `sm_por_ano`, `regua`, `tabela_em_reais`, `por_origem_do_empregador`, `multiplicadores`, `corroboracao_por_edicao`, `moeda_por_faixa`.
  21. `vitrine.json` (`vitrine`) — Primária: `perfis` (50); Sub-seções: `empresas`, `empresas_verificadas`, `contratando`, `porte`, `setor`, `regiao_das_empresas`, `paises_no_exterior`, `trilhas`, `regioes`.
- **Rationale**: Atende diretamente à decisão Q1 da clarificação (`FR-007`, `FR-008`, `FR-009`), expondo exatamente 21 abas principais (uma por arquivo JSON) com sub-abas internas para arquivos que possuem mais de uma coleção, além de editor para o envelope de metadados do arquivo.
- **Alternatives considered**: Expor apenas um editor JSON bruto por arquivo — rejeitado porque viola o requisito de tabelas com busca, ordenação e CRUD de registros (`FR-008`).

## R3 — Propagação opcional entre tabelas relacionadas (Clarification Q2 / FR-008a)

- **Decision**: Suportar o parâmetro `propagate: bool` (default `false`) nas operações de atualização (`egressos_update_row`) e exclusão (`egressos_delete_row`). Quando `propagate == true`:
  - **Grupo Egresso (`rotulo` / `label` / `perfil`)**:
    - Identifica o egresso pela chave canônica (`label` em `egressos_perfil:egressos` e `analise:rows`; `rotulo` em `vitrine:perfis`, `panorama:coorte` e `panorama:salto`; `perfil` normalizado `"Perfil <rotulo>"` em `consolidado:perfis`).
    - Ao editar com `propagate = true`, sincroniza os atributos equivalentes presentes nos registros irmãos:
      - `nome` ↔ `egressos_perfil.nome`, `vitrine.nome`
      - `monograma` ↔ `egressos_perfil.monograma`, `vitrine.monograma`
      - `curso` ↔ `egressos_perfil.curso`, `vitrine.curso`, `analise.curso`
      - `cargo_atual` / `cargo` ↔ `egressos_perfil.cargo_atual`, `vitrine.cargo`, `panorama.cargo_atual`
      - `empresa_atual` / `empresa` / `empregador_atual` ↔ `egressos_perfil.empresa_atual`, `vitrine.empresa`, `panorama.empregador_atual`
      - `local_atual` / `lugar` ↔ `egressos_perfil.local_atual`, `vitrine.lugar`
      - `trilha` ↔ `analise.trilha`, `consolidado.trilha`, `panorama:salto.trilha`
    - Ao excluir com `propagate = true`, remove o registro correspondente nas coleções irmãs do mesmo `rotulo`.
  - **Grupo Empresa (`_key` / `nome` / `empresa`)**:
    - Identifica a empresa pelo nome canônico nas tabelas `empresas_porte:_root`, `empresas_aliases:_root`, `empresas_linkedin_data:_root`, `empresas_linkedin_urls:_root`, `vitrine:empresas`, `vitrine:empresas_verificadas` e `vitrine:contratando`.
    - Ao editar com `propagate = true`, sincroniza a renomeação da chave (`_key`/`empresa`/`nome`) e campos compartilhados (`porte`, `origem`, `setor`, `url`) nos arquivos de empresa onde a entidade existir.
    - Ao excluir com `propagate = true`, remove a empresa das tabelas de empresa relacionadas.
- **Rationale**: Cumpre exatamente a decisão da clarificação Q2 (`FR-008a`), permitindo que o curador escolha na interface (via caixa de seleção *"Propagar alteração para tabelas relacionadas do Egressos"*) entre edição isolada (A) ou propagada (B), dentro de uma única transação SQLite.

## R4 — Importação e Exportação Local de Pasta (`dados/`)

- **Decision**: Nos comandos `import_egressos_dir` e `export_egressos_dir`, utilizar o seletor nativo de diretório do `tauri_plugin_dialog` (`blocking_pick_folder`), sugerindo `/home/thiagodeps/egressos/dados` quando existir no sistema de arquivos local, além de aceitar `path: Option<String>` explícito (permitindo também acionar com 1 clique "Carregar de `/home/thiagodeps/egressos/dados`" na interface ou em testes automatizados).
- **Rationale**: Atende à clarificação Q4 (`FR-004`, `FR-010`) e mantém a regra arquitetural do Research Hub (Princípio VI: o front-end nunca acessa o sistema de arquivos diretamente; toda leitura/escrita de pasta acontece em Rust).
- **Snapshot**: Antes de substituir uma base Egressos não vazia, `import_egressos_dir` grava um backup SQLite com prefixo `egressos-pre-import-<timestamp>.db` no diretório `snapshots/` do `app_data_dir`, mantendo a política de retenção independente dos snapshots de Horizon e SRC (`FR-006`).

## R5 — Sincronização Direta da Pasta `dados/` com o GitHub via Token (Clarification Q3 / FR-011, FR-012)

- **Decision**: Estender `sync_domain.rs` e `sync_github.rs` para suportar o projeto `"egressos"` operando sobre **um diretório de 21 arquivos `.json`** no repositório GitHub:
  - `PROJECTS` passa a `["horizon", "src", "egressos"]`.
  - `check_upload_allowed("egressos")` retorna `Ok(())` (envio habilitado, conforme spec `FR-012`).
  - `validate_path("egressos", path)` aceita caminhos de diretório relativos (ex.: `"dados"` ou `"dados/"`, normalizados sem barra inicial/final, sem `".."`).
  - **Download (`github_download` para `project == "egressos"`)**:
    - Aceita como entrada na UI tanto uma URL de pasta do GitHub (ex.: `https://github.com/thiagodeps/egressos/tree/main/dados` ou `https://github.com/thiagodeps/egressos`) quanto a configuração salva `(repo, branch, path)`.
    - O `GitHubClient::download_egressos_dir` baixa os 21 arquivos `.json` canônicos de `{path}/{file_name}` na `branch`/`ref` via Contents API (`Accept: application/vnd.github.raw`), emite eventos `sync://progress` a cada arquivo baixado, grava-os em um subdiretório temporário `sync_tmp/egressos-<ts>/` e só invoca `egressos_domain::import_dir` após todos os 21 arquivos terem sido baixados com sucesso. Se qualquer arquivo falhar (404, 401/403, rede, JSON inválido), o diretório temporário é removido e a base local do Egressos permanece intacta (`FR-005`, `FR-011`).
  - **Upload (`github_upload` para `project == "egressos"`)**:
    - Exporta em memória os 21 arquivos `(file_name, bytes)` a partir do estado atual do SQLite (`egressos_domain::export_files_in_memory(&conn)`).
    - No `GitHubClient::upload_directory`, cria os 21 blobs na Git Data API (`POST /repos/{owner}/{repo}/git/blobs`), lê o `head_ref` da branch existente (falhando com erro claro se a branch não existir, sem criar branch automaticamente), monta uma única `tree` (`POST /repos/{owner}/{repo}/git/trees` com `base_tree` e as 21 entradas `{ path: "{dir}/{file_name}", mode: "100644", type: "blob", sha }`), cria um único commit (`POST /repos/{owner}/{repo}/git/commits`) e avança a ref da branch (`PATCH /repos/{owner}/{repo}/git/refs/heads/{branch}`).
- **Rationale**: Publica todas as 21 tabelas em **um único commit atômico** na branch do GitHub, garantindo que o repositório remoto nunca fique com metade dos arquivos JSON atualizados em caso de falha de rede no meio do envio (`Edge Cases`, `SC-005`).
- **Alternatives considered**: Fazer 21 `PUT /repos/{owner}/{repo}/contents/{path}` sequenciais — rejeitado porque geraria 21 commits separados no histórico do repositório e deixaria o repositório inconsistente se a conexão caísse no 10º arquivo.
