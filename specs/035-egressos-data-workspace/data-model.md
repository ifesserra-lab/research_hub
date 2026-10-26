# Data Model: Espaço de Dados do Projeto Egressos (`035-egressos-data-workspace`)

**Date**: 2026-10-22 | **Feature**: [spec.md](./spec.md) | **Research**: [research.md](./research.md)

## 1. Persistência SQLite (`003_egressos_tables.sql`)

A migração `003_egressos_tables.sql` adiciona duas tabelas ao banco único `hub.db` (elevando o total de tabelas de 19 para 21):

### 1.1. Tabela `egressos_files`

Armazena o envelope de cada um dos 21 arquivos JSON de `/home/thiagodeps/egressos/dados`.

| Coluna | Tipo SQLite | Restrições | Descrição |
|--------|-------------|------------|-----------|
| `file_key` | `TEXT` | `PRIMARY KEY` | Identificador canônico do arquivo sem extensão (ex.: `'egressos_perfil'`, `'vitrine'`, `'empresas_porte'`). |
| `kind` | `TEXT` | `NOT NULL` | Tipo estrutural do JSON raiz: `'object_sections'`, `'root_map'` ou `'nested_map'`. |
| `primary_section` | `TEXT` | `NOT NULL` | Chave da seção/tabela principal aberta por padrão na aba (ex.: `'egressos'`, `'perfis'`, `'_root'`). |
| `envelope_json` | `TEXT` | `NOT NULL` | Objeto JSON serializado (preservando ordem das chaves) com os metadados de nível de arquivo, campos escalares, objetos auxiliares e marcadores de posição das seções tabulares. |
| `imported_at` | `TEXT` | `NULL` | Timestamp ISO-8601 da última importação. |

### 1.2. Tabela `egressos_rows`

Armazena os registros individuais de cada seção/tabela de cada um dos 21 arquivos JSON.

| Coluna | Tipo SQLite | Restrições | Descrição |
|--------|-------------|------------|-----------|
| `id` | `INTEGER` | `PRIMARY KEY AUTOINCREMENT` | Identificador interno da linha no SQLite para CRUD e paginação. |
| `file_key` | `TEXT` | `NOT NULL REFERENCES egressos_files(file_key) ON DELETE CASCADE` | Arquivo JSON ao qual o registro pertence. |
| `section_key` | `TEXT` | `NOT NULL` | Nome da chave da lista/mapa dentro do arquivo JSON (ou `'_root'` quando o próprio arquivo é um mapa raiz como `empresas_porte.json`). |
| `ord` | `INTEGER` | `NOT NULL` | Ordem posicional do item na coleção para reconstrução determinística na exportação. |
| `record_key` | `TEXT` | `NULL` | Chave identificadora lógica do registro (ex.: `rotulo`/`label` do egresso, nome da empresa `_key`, `ano`, `projeto_id`), usada para unicidade em mapas e propagação opcional entre arquivos (`FR-008a`). |
| `search_text` | `TEXT` | `NOT NULL DEFAULT ''` | Texto concatenado dos campos visíveis da linha para busca rápida via `LIKE`. |
| `raw_json` | `TEXT` | `NOT NULL` | Objeto JSON completo do registro (ou `{ "_key": "...", ... }` em mapas e `{ "valor": ... }` em listas de escalares), preservando todos os campos escalares e aninhados. |

**Índices**:
- `idx_egressos_rows_file_section_ord ON egressos_rows(file_key, section_key, ord)`
- `idx_egressos_rows_record_key ON egressos_rows(record_key)`

---

## 2. Catálogo das 21 Abas / Arquivos JSON (`EGRESSOS_FILES`)

Cada entrada do catálogo define o `file_key`, nome do arquivo em `dados/`, título da aba, tipo estrutural (`kind`), seção principal (`primary_section`), sub-seções (`sections`), colunas principais exibidas na grade e grupo de vínculo (`entity_group`) para propagação opcional (`FR-008a`):

| # | `file_key` | Arquivo em `dados/` | `kind` | Seção Principal (`primary_section`) | Sub-seções Adicionais (`sections`) | Colunas Principais da Grade Primária | Grupo de Propagação (`FR-008a`) |
|---|------------|---------------------|--------|-------------------------------------|------------------------------------|--------------------------------------|---------------------------------|
| 1 | `egressos_perfil` | `egressos_perfil.json` | `object_sections` | `egressos` (50) | `cursos` (mapa: `bsi`, `eca`) | `label`, `nome`, `curso`, `cargo_atual`, `empresa_atual`, `local_atual`, `area_atual` | `Alumni` (chave: `label`) |
| 2 | `vitrine` | `vitrine.json` | `object_sections` | `perfis` (50) | `empresas` (45), `empresas_verificadas` (36), `contratando` (22), `porte` (6), `setor` (9), `regiao_das_empresas` (7), `paises_no_exterior` (5), `trilhas` (6), `regioes` (4) | `rotulo`, `nome`, `curso`, `cargo`, `empresa`, `lugar` | `Alumni` (`perfis` por `rotulo`) / `Company` (`empresas*` por `nome`) |
| 3 | `panorama` | `panorama.json` | `object_sections` | `coorte` (50) | `salto` (49), `areas` (11), `familias` (4), `origem_comum` (8) | `rotulo`, `cargo_atual`, `empregador_atual`, `origem`, `modalidade`, `area_atual_nome`, `inicio_carreira` | `Alumni` (`coorte` e `salto` por `rotulo`) |
| 4 | `analise` | `analise.json` | `object_sections` | `rows` (50) | `top_tech` (15), `funcoes` (6), `metodos` (10), `intl_timeline` (15), `trajetoria_destaque` (11), `hist_multiplicadores` (6) | `label`, `curso`, `anos`, `n_empresas`, `n_exp`, `trilha`, `em_tech`, `exterior` | `Alumni` (`rows` por `label`) |
| 5 | `consolidado` | `consolidado.json` | `object_sections` | `perfis` (49) | `agregado` (18), `pv` (18), `por_ano_sm` (16), `series_por_perfil` (mapa 49) | `perfil`, `trilha`, `em_tech`, `anos`, `bolsa`, `med_ini`, `med_atual`, `cresc` | `Alumni` (`perfis` por sufixo de `perfil`) |
| 6 | `empresas_porte` | `empresas_porte.json` | `root_map` | `_root` (204) | — | `_key`, `empresa`, `porte`, `origem`, `setor`, `funcionarios` | `Company` (chave: `_key`) |
| 7 | `empresas_aliases` | `empresas_aliases.json` | `root_map` | `_root` (207) | — | `_key`, `aliases`, `atual`, `n_egressos_atual` | `Company` (chave: `_key`) |
| 8 | `empresas_linkedin_data` | `empresas_linkedin_data.json` | `root_map` | `_root` (46) | — | `_key`, `nome_oficial`, `industry_linkedin`, `headcount_linkedin`, `hq_local`, `website`, `founded` | `Company` (chave: `_key`) |
| 9 | `empresas_linkedin_urls` | `empresas_linkedin_urls.json` | `root_map` | `_root` (38) | — | `_key`, `url`, `slug`, `confianca`, `via` | `Company` (chave: `_key`) |
| 10 | `fapes_fomento` | `fapes_fomento.json` | `object_sections` | `projetos` (2) | — | `projeto_id`, `titulo`, `situacao`, `instituicao`, `bolsa_sigla`, `bolsistas`, `valor_alocado` | — |
| 11 | `impacto` | `impacto.json` | `object_sections` | `indicadores` (7) | `por_experiencia` (18), `por_experiencia_hoje` (18), `achados` (4), `ressalvas` (5), `dispersao` (4), `vias_de_formacao` (4), `internacional_no_tempo` (15), `funcoes` (6), `metodos` (10), `tecnologias` (15), `senioridade` (3), `perfis_agrupados` (3) | `rotulo`, `valor` | — |
| 12 | `trajetoria` | `trajetoria.json` | `object_sections` | `serie` (15) | `trajetoria` (11), `trajetoria_coorte` (11), `sm_por_ano` (15), `regua` (16), `tabela_em_reais` (11), `por_origem_do_empregador` (3), `multiplicadores` (6), `corroboracao_por_edicao` (3), `moeda_por_faixa` (4) | `ano`, `n`, `med_real`, `med_nominal`, `sm_ano`, `em_sm` | — |
| 13 | `renda_por_senioridade` | `renda_por_senioridade.json` | `object_sections` | `cargos` (5) | `niveis_sem_referencia_do_modelo` (3) | `nivel`, `referencias`, `por_trilha_no_modelo` | — |
| 14 | `cargos_ao_longo_do_tempo` | `cargos_ao_longo_do_tempo.json` | `object_sections` | `cargos` (28) | `edicoes` (8) | `cargo`, `nome`, `n_edicoes`, `de`, `ate`, `variacao_pct` | — |
| 15 | `codigofonte_2026` | `codigofonte_2026.json` | `object_sections` | `por_senioridade` (5) | `por_contrato` (3), `por_regiao_top10` (10), `por_linguagem` (12), `recortes_indisponiveis_na_pagina` (3), `observacoes` (2) | `nivel`, `media` | — |
| 16 | `codigofonte_historico` | `codigofonte_historico.json` | `nested_map` | `media_por_senioridade` (6) | `ipca_fator_para_2026` (6) | `_key` (ano), `Estágio`, `Júnior`, `Pleno`, `Sênior`, `Especialista/Tech Lead` | — |
| 17 | `salario_minimo` | `salario_minimo.json` | `object_sections` | `por_ano` (16) | `notas` (4) | `ano`, `jan`, `dez`, `media_ponderada`, `reajuste_pct`, `ganho_real_pct` | — |
| 18 | `ibge_series` | `ibge_series.json` | `nested_map` | `series` (4) | — | `_key` (`ipca`, `inpc`, `salario_minimo`, `cambio`), `fonte`, `unidade` | — |
| 19 | `so_benchmarks` | `so_benchmarks.json` | `object_sections` | `por_pais` (12) | `serie_anual` (8), `por_moeda_brasil` (4), `candidatas_a_referencia` (3), `notas` (5) | `pais`, `usd_mes`, `n` | — |
| 20 | `recorte_cursos` | `recorte_cursos.json` | `nested_map` | `cursos` (2) | — | `_key` (`bsi`, `eca`), `nome`, `sigla`, `n` | — |
| 21 | `mapa_mundi` | `mapa_mundi.json` | `object_sections` | `paths` (113) | `viewBox` (4), `lat_range` (2) | `idx`, `value` | — |

---

## 3. Regras de Validação e Integridade

1. **Validação Atômica da Pasta `dados/` (`FR-005`)**:
   - Na importação local ou via GitHub, todos os 21 arquivos listados em `EGRESSOS_FILES` devem estar presentes e conter JSON válido compatível com o `kind` e as seções esperadas.
   - Se qualquer um dos 21 arquivos estiver ausente ou malformado, a transação não é iniciada e a base local permanece intacta.
2. **Unicidade de Chave em Mapas (`Edge Cases`)**:
   - Em seções representadas como mapa (ex.: `_root` em `empresas_porte`, `empresas_aliases`, `empresas_linkedin_data`, `empresas_linkedin_urls`, ou seções `nested_map`), o campo `_key` é obrigatório, não vazio e único dentro de `(file_key, section_key)`. Tentar criar ou renomear um registro para um `_key` já existente retorna `AppError::Conflict`.
3. **Propagação Opcional entre Tabelas Relacionadas (`FR-008a`)**:
   - `propagate = false` (Opção A): atualiza/remove exclusivamente a linha `id` na tabela `(file_key, section_key)` aberta.
   - `propagate = true` (Opção B): dentro da mesma transação SQLite, localiza registros irmãos do mesmo grupo (`Alumni` por código do egresso `A`..`AX`, ou `Company` pelo nome da empresa) nas demais tabelas do catálogo e atualiza os campos equivalentes (ou exclui os registros irmãos no caso de exclusão propagada).
4. **Fidelidade de Reconstrução (`SC-003`)**:
   - Para cada arquivo `file_key`, a exportação lê `envelope_json` e reconstrói cada seção a partir de `egressos_rows` ordenada por `ord ASC`:
     - Se a seção é uma lista de objetos: reconstrói o `Vec<Value>` com o `raw_json` de cada linha.
     - Se a seção é uma lista de escalares (ex.: `notas`, `trilhas`, `paths`): extrai o valor escalar de `{ "value": ... }` em `raw_json`.
     - Se a seção (ou raiz) é um mapa indexado por `_key`: reconstrói o `serde_json::Map<String, Value>` usando `_key` como chave de propriedade e o restante do objeto (ou valor `_value`) como valor associado, preservando a ordem `ord ASC`.
