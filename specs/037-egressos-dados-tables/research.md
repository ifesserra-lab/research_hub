# Research & Architecture Decisions: Tabelas, CRUD e Sincronização de `egressos-dados`

**Feature**: `037-egressos-dados-tables`  
**Date**: 2026-11-05  
**Related Spec**: [spec.md](file:///home/thiagodeps/research_hub/specs/037-egressos-dados-tables/spec.md)

## R1. Catálogo Canônico dos 39 Datasets JSON de `egressos-dados` (`EGRESSOS_DADOS_FILES`)

- **Decision**: Criar o módulo `src-tauri/src/egressos_dados_domain.rs` contendo a constante estática `EGRESSOS_DADOS_FILES: &[EgressosDadosFileDef; 39]` alinhada 1:1 com os 39 datasets declarados em `/home/thiagodeps/egressos-dados/packages/egressos-core/src/egressos_core/dados.py` (`CATALOGO`):
  - **12 datasets PII (`publication: "pii"`)**: `alunos` (`alunos.json` na raiz), `perfis_linkedin`, `perfis_conferidos`, `formandos_a_buscar`, `_novos_reais`, `genero_map`, `horizon_pesquisa`, `lattes_orientacoes`, `outros_labs`, `src_extensao`, `por_ano`, `consolidado_raw`.
  - **6 datasets Internos (`publication: "interno"`)**: `_empresas_lista`, `_novos_anon5`, `_novos_anon6`, `agg_pv`, `fontes_estado`, `mapa_mundi_detalhe`.
  - **21 datasets Publicáveis (`publication: "publicavel"`)**: `egressos_perfil`, `vitrine`, `panorama`, `analise`, `consolidado`, `impacto`, `trajetoria`, `recorte_cursos`, `fapes_fomento`, `renda_por_senioridade`, `cargos_ao_longo_do_tempo`, `codigofonte_2026`, `codigofonte_historico`, `so_benchmarks`, `salario_minimo`, `ibge_series`, `empresas_porte`, `empresas_aliases`, `empresas_linkedin_data`, `empresas_linkedin_urls`, `mapa_mundi`.
- **Rationale**: Expor tanto `role` (`"entrada"` ou `"saida"`) quanto `publication` (`"pii"`, `"interno"` ou `"publicavel"`) e `relative_path` (`"alunos.json"` vs `"data/<nome>.json"`) diretamente no catálogo estático permite filtrar abas por categoria no front-end e importar/exportar/sincronizar tanto a raiz quanto a subpasta `data/` sem lógica condicional espalhada.
- **Alternatives Considered**:
  - *Compartilhar a mesma tabela SQLite `egressos_files` do projeto público `Egressos`*: rejeitado porque violaria o isolamento entre projetos (`FR-002`) e misturaria os 21 arquivos públicos de `/egressos` com os 39 arquivos restritos da organização `ifesserra-lab`.

## R2. Suporte a 4 Formatos Estruturais de JSON (`object_sections`, `root_array`, `root_map`, `nested_map`)

- **Decision**: Estender a decomposição e reconstrução *round-trip* para suportar 4 tipos estruturais (`FileKind`):
  1. `ObjectSections`: objeto JSON na raiz com seções (`Array` ou `Object`) + metadados no envelope (ex.: `alunos.json`, `data/perfis_linkedin.json`, `data/formandos_a_buscar.json`, `data/src_extensao.json`).
  2. `RootArray`: array JSON `[]` diretamente na raiz do arquivo (`section_key == "_root"`), armazenando no `envelope_json` o marcador `{"__root_shape__": "array_objects" | "array_scalars"}` para reconstruir fielmente arrays de objetos (`_novos_reais.json`, `_novos_anon5.json`, `_novos_anon6.json`, `agg_pv.json`, `consolidado_raw.json`, `por_ano.json`) e arrays de escalares (`_empresas_lista.json`).
  3. `RootMap`: objeto JSON na raiz mapeando `_key` para objetos (`empresas_porte.json`, `empresas_aliases.json`, `empresas_linkedin_data.json`, `empresas_linkedin_urls.json`) ou para escalares (`genero_map.json`, onde cada entrada é `"id": "M" | "F"`), gravando no `envelope_json` `{"__root_shape__": "map_objects" | "map_scalars"}`.
  4. `NestedMap`: objeto JSON na raiz com metadados e sub-mapas indexados por `_key` (`codigofonte_historico.json`, `ibge_series.json`, `recorte_cursos.json`, `fontes_estado.json`).
- **Rationale**: No repositório `/home/thiagodeps/egressos-dados`, 7 arquivos possuem array na raiz (`RootArray`) e `data/genero_map.json` é um `RootMap` de escalares (`{"barbosa": "M", ...}`). Armazenar o shape exato no envelope permite 100% de equivalência *round-trip* (`SC-002`).
- **Alternatives Considered**:
  - *Converter arrays de raiz em objetos com chave artificial na exportação*: rejeitado porque quebraria o pipeline Python de `egressos-dados` (`egressos_core.dados`).

## R3. Preservação do Campo `"id"` do JSON vs Chave Primária Interna (`_row_id`)

- **Decision**: Em `egressos-dados`, diversos arquivos possuem um campo de domínio chamado `"id"` do tipo string no próprio objeto JSON (`alunos.json` com `"id": "barbosa"`, `_novos_reais.json` com `"id": "guilherme_gatti"`, `horizon_pesquisa.json` com `"id": "ana_rubia"`, `lattes_orientacoes.json`, `outros_labs.json`, `consolidado_raw.json`, `_novos_anon5.json`, `_novos_anon6.json`). Portanto:
  - `row_to_record` injeta a chave primária numérica do SQLite em `"_row_id"` (e só preenche `"id"` com o número do SQLite se o objeto JSON original não possuir a chave `"id"`).
  - `clean_payload_for_storage` remove apenas `"_row_id"` e `"_record_key"` (e remove `"id"` somente se seu valor for um inteiro igual ao `_row_id` em tabelas que não declaram `"id"` nas colunas), preservando integralmente qualquer `"id"` textual do domínio!
- **Rationale**: Garante que o `id` nominal do aluno (`"barbosa"`, `"gary"`, `"helen"`, etc.) apareça na coluna `id` da tabela `alunos`, possa ser editado ou criado no modal de CRUD e seja gravado sem perda em `alunos.json` e nos demais arquivos PII.
- **Alternatives Considered**:
  - *Sobrescrever `"id"` com o `rowid` numérico do SQLite*: rejeitado porque ocultaria e corromperia o campo `"id"` de `alunos.json` e outros 7 datasets de `egressos-dados`.

## R4. Resolução Automática da Raiz do Repositório (`alunos.json` + `data/*.json`)

- **Decision**: Na importação e exportação local (`import_dir` e `export_dir`), aceitar tanto o diretório raiz do repositório (`/home/thiagodeps/egressos-dados`) quanto a subpasta `data/` (`/home/thiagodeps/egressos-dados/data`):
  - Se o diretório informado terminar em `/data` e não contiver `alunos.json`, mas seu diretório pai contiver `alunos.json` (na importação) ou for um diretório válido (na exportação), normalizar automaticamente para o diretório raiz do repositório.
  - A partir da raiz `repo_root`, cada arquivo é lido/escrito em `repo_root.join(def.relative_path)` (`alunos.json` na raiz e `data/<nome>.json` na subpasta `data/`).
- **Rationale**: Evita erro de usuário caso o curador selecione `/home/thiagodeps/egressos-dados/data` em vez de `/home/thiagodeps/egressos-dados` no seletor de pastas, ao mesmo tempo em que preserva a localização exata de `alunos.json` na raiz e dos 38 arquivos em `data/`.
- **Alternatives Considered**:
  - *Exigir duas seleções de pasta separadas*: rejeitado por piorar a usabilidade e quebrar a atomicidade.

## R5. Sincronização GitHub com Reuso do Token em Memória (`EgressosDadosAuth`)

- **Decision**:
  - Criar os comandos IPC `egressos_dados_sync_get_config`, `egressos_dados_sync_set_config`, `egressos_dados_sync_download` e `egressos_dados_sync_upload`, todos protegidos por `state.require_egressos_dados_session()?`.
  - A configuração de repositório de `egressos-dados` nasce pré-configurada com `repo: "ifesserra-lab/egressos-dados"`, `branch: "main"`, `path: "."` (raiz do repositório, de onde saem `alunos.json` e `data/*.json`).
  - O token utilizado para download e upload é obtido diretamente da sessão ativa em memória (`EgressosDadosAuth.token`), sem exigir que o curador salve o token em `sync_config.json` no disco (Clarificação Q4).
  - No download e no upload via Git Data API, os 39 arquivos usam seus caminhos relativos à raiz do repositório (`alunos.json` e `data/<arquivo>.json`, ou prefixados por `path` caso o usuário configure um subdiretório diferente de `"."`).
- **Rationale**: Cumpre `FR-001`, `FR-010` e a Clarificação Q4: quem já destravou a aba com o token da `ifesserra-lab` sincroniza imediatamente os 39 arquivos em um único fluxo atômico.
- **Alternatives Considered**:
  - *Pedir um segundo token separado na tela de controle de dados*: rejeitado na Clarificação Q4.
