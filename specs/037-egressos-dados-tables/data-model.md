# Data Model: Tabelas, CRUD e Sincronização de `egressos-dados`

**Feature**: `037-egressos-dados-tables`  
**Date**: 2026-11-05  
**Related Spec**: [spec.md](file:///home/thiagodeps/research_hub/specs/037-egressos-dados-tables/spec.md)

## 1. Esquema SQLite (`src-tauri/migrations/004_egressos_dados_tables.sql`)

Para manter isolamento estrito (`FR-002`) em relação às 15 tabelas do `Horizon`, às 3 tabelas do `SRC` e às 2 tabelas do projeto público `Egressos` (`egressos_files` e `egressos_rows`), a migração `004_egressos_dados_tables.sql` adiciona **2 tabelas exclusivas** para `egressos-dados` (totalizando 23 tabelas no banco):

```sql
CREATE TABLE IF NOT EXISTS egressos_dados_files (
    file_key TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    primary_section TEXT NOT NULL,
    envelope_json TEXT NOT NULL,
    imported_at TEXT NULL
);

CREATE TABLE IF NOT EXISTS egressos_dados_rows (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_key TEXT NOT NULL REFERENCES egressos_dados_files(file_key) ON DELETE CASCADE,
    section_key TEXT NOT NULL,
    ord INTEGER NOT NULL,
    record_key TEXT NULL,
    search_text TEXT NOT NULL DEFAULT '',
    raw_json TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_egressos_dados_rows_file_section_ord
    ON egressos_dados_rows(file_key, section_key, ord);

CREATE INDEX IF NOT EXISTS idx_egressos_dados_rows_record_key
    ON egressos_dados_rows(record_key);
```

## 2. Estruturas de Domínio (`src-tauri/src/egressos_dados_domain.rs`)

### 2.1 `FileKind`, `PublicationKind`, `DatasetRole` e `EntityGroup`

- `FileKind`:
  - `ObjectSections` (`"object_sections"`): objeto JSON na raiz com seções (`Array` ou `Object`) e metadados no envelope.
  - `RootArray` (`"root_array"`): array JSON `[]` diretamente na raiz (`section_key == "_root"`), podendo conter objetos (`"array_objects"`) ou escalares (`"array_scalars"`).
  - `RootMap` (`"root_map"`): objeto JSON na raiz mapeando `_key` para objetos (`"map_objects"`) ou escalares (`"map_scalars"`).
  - `NestedMap` (`"nested_map"`): objeto JSON na raiz com metadados e sub-mapas indexados por `_key`.
- `PublicationKind`:
  - `Pii` (`"pii"`): dados pessoais / nominais restritos à organização `ifesserra-lab`.
  - `Interno` (`"interno"`): insumos e estado interno do repositório privado.
  - `Publicavel` (`"publicavel"`): saídas agregadas ou anonimizadas.
- `DatasetRole`:
  - `Entrada` (`"entrada"`): curado manualmente por pessoa.
  - `Saida` (`"saida"`): gerado pelo pipeline (também editável via CRUD conforme Clarificação Q3).
- `EntityGroup`:
  - `Alumni` (`"alumni"`): registros de egressos/alunos identificados por `id`, `label`, `rotulo`, `perfil` ou `_key` (em `genero_map`).
  - `Company` (`"company"`): registros de empresas identificados por `_key`, `empresa` ou `nome`.
  - `None` (`"none"`): sem propagação entre arquivos.

### 2.2 Catálogo Canônico dos 39 Datasets (`EGRESSOS_DADOS_FILES`)

| # | `file_key` | `relative_path` | `title` | `kind` | `publication` | `role` | `primary_section` | `sections` |
|---|------------|-----------------|---------|--------|---------------|--------|-------------------|------------|
| 1 | `alunos` | `alunos.json` | `Cadastro de Alunos (Coorte)` | `object_sections` | `pii` | `entrada` | `alunos` | `["alunos", "cursos", "convencoes"]` |
| 2 | `perfis_linkedin` | `data/perfis_linkedin.json` | `Perfis LinkedIn (Coleta SIGA)` | `object_sections` | `pii` | `saida` | `perfis` | `["perfis"]` |
| 3 | `perfis_conferidos` | `data/perfis_conferidos.json` | `Perfis Conferidos Manualmente` | `object_sections` | `pii` | `entrada` | `conferidos` | `["conferidos"]` |
| 4 | `formandos_a_buscar` | `data/formandos_a_buscar.json` | `Fila de Formandos SIGA` | `object_sections` | `pii` | `saida` | `fila` | `["fila"]` |
| 5 | `_novos_reais` | `data/_novos_reais.json` | `Coorte Nova (Nomes Reais)` | `root_array` | `pii` | `entrada` | `_root` | `["_root"]` |
| 6 | `genero_map` | `data/genero_map.json` | `Mapa de Gênero por Aluno` | `root_map` | `pii` | `saida` | `_root` | `["_root"]` |
| 7 | `horizon_pesquisa` | `data/horizon_pesquisa.json` | `Cruzamento Horizon Pesquisa` | `object_sections` | `pii` | `entrada` | `encontrados` | `["encontrados", "nao_encontrados", "sem_vinculo_pesquisa", "papeis_agregado", "tabelas_usadas"]` |
| 8 | `lattes_orientacoes` | `data/lattes_orientacoes.json` | `Cruzamento Lattes Orientações` | `object_sections` | `pii` | `entrada` | `encontrados` | `["encontrados", "tipos_agregado"]` |
| 9 | `outros_labs` | `data/outros_labs.json` | `Outros Laboratórios (além do LEDS)` | `object_sections` | `pii` | `entrada` | `egressos` | `["egressos", "labs", "detalhe_fonte"]` |
| 10 | `src_extensao` | `data/src_extensao.json` | `Cruzamento SRC Extensão` | `object_sections` | `pii` | `saida` | `fapes_pesquisa` | `["fapes_pesquisa", "bolsa_documentada_oficial", "genero", "encontrados", "bolsistas_extensao", "funcoes", "projetos_por_egresso"]` |
| 11 | `por_ano` | `data/por_ano.json` | `Histórico Individual por Ano` | `root_array` | `pii` | `entrada` | `_root` | `["_root"]` |
| 12 | `consolidado_raw` | `data/consolidado_raw.json` | `Consolidado Bruto (Pré-Anonimização)` | `root_array` | `pii` | `entrada` | `_root` | `["_root"]` |
| 13 | `_empresas_lista` | `data/_empresas_lista.json` | `Lista de Empresas (Insumo)` | `root_array` | `interno` | `saida` | `_root` | `["_root"]` |
| 14 | `_novos_anon5` | `data/_novos_anon5.json` | `Coorte Nova Anonimizada (5 campos)` | `root_array` | `interno` | `entrada` | `_root` | `["_root"]` |
| 15 | `_novos_anon6` | `data/_novos_anon6.json` | `Coorte Nova Anonimizada (6 campos)` | `root_array` | `interno` | `entrada` | `_root` | `["_root"]` |
| 16 | `agg_pv` | `data/agg_pv.json` | `Agregado de Valor Presente` | `root_array` | `interno` | `entrada` | `_root` | `["_root"]` |
| 17 | `fontes_estado` | `data/fontes_estado.json` | `Estado do Vigia de Fontes` | `nested_map` | `interno` | `saida` | `fontes` | `["fontes"]` |
| 18 | `mapa_mundi_detalhe` | `data/mapa_mundi_detalhe.json` | `Mapa-Múndi Detalhado (50m)` | `object_sections` | `interno` | `saida` | `paths` | `["paths", "viewBox", "lat_range"]` |
| 19 | `egressos_perfil` | `data/egressos_perfil.json` | `Perfis de Egressos` | `object_sections` | `publicavel` | `saida` | `egressos` | `["egressos", "cursos"]` |
| 20 | `vitrine` | `data/vitrine.json` | `Vitrine de Egressos` | `object_sections` | `publicavel` | `saida` | `perfis` | `["perfis", "empresas", "empresas_verificadas", "contratando", "porte", "setor", "regiao_das_empresas", "paises_no_exterior", "trilhas", "regioes"]` |
| 21 | `panorama` | `data/panorama.json` | `Panorama do Coorte` | `object_sections` | `publicavel` | `saida` | `coorte` | `["coorte", "salto", "areas", "familias", "origem_comum"]` |
| 22 | `analise` | `data/analise.json` | `Análise do Grupo` | `object_sections` | `publicavel` | `saida` | `rows` | `["rows", "top_tech", "funcoes", "metodos", "intl_timeline", "trajetoria_destaque", "hist_multiplicadores"]` |
| 23 | `consolidado` | `data/consolidado.json` | `Consolidado Salarial` | `object_sections` | `publicavel` | `saida` | `perfis` | `["perfis", "agregado", "pv", "por_ano_sm", "series_por_perfil"]` |
| 24 | `impacto` | `data/impacto.json` | `Impacto na Carreira` | `object_sections` | `publicavel` | `saida` | `indicadores` | `["indicadores", "por_experiencia", "por_experiencia_hoje", "dispersao", "vias_de_formacao", "internacional_no_tempo", "funcoes", "metodos", "tecnologias", "senioridade", "perfis_agrupados", "achados", "ressalvas"]` |
| 25 | `trajetoria` | `data/trajetoria.json` | `Trajetória Salarial` | `object_sections` | `publicavel` | `saida` | `serie` | `["serie", "trajetoria", "sm_por_ano", "regua", "tabela_em_reais", "por_origem_do_empregador", "multiplicadores", "corroboracao_por_edicao", "moeda_por_faixa", "trajetoria_coorte"]` |
| 26 | `recorte_cursos` | `data/recorte_cursos.json` | `Recorte por Cursos` | `nested_map` | `publicavel` | `saida` | `cursos` | `["cursos"]` |
| 27 | `fapes_fomento` | `data/fapes_fomento.json` | `Fomento FAPES` | `object_sections` | `publicavel` | `saida` | `projetos` | `["projetos"]` |
| 28 | `renda_por_senioridade` | `data/renda_por_senioridade.json` | `Renda por Senioridade` | `object_sections` | `publicavel` | `saida` | `cargos` | `["cargos", "niveis_sem_referencia_do_modelo"]` |
| 29 | `cargos_ao_longo_do_tempo` | `data/cargos_ao_longo_do_tempo.json` | `Cargos ao Longo do Tempo` | `object_sections` | `publicavel` | `saida` | `cargos` | `["cargos", "edicoes"]` |
| 30 | `codigofonte_2026` | `data/codigofonte_2026.json` | `Pesquisa Código Fonte 2026` | `object_sections` | `publicavel` | `entrada` | `por_senioridade` | `["por_senioridade", "por_contrato", "por_regiao_top10", "por_linguagem", "recortes_indisponiveis_na_pagina", "observacoes"]` |
| 31 | `codigofonte_historico` | `data/codigofonte_historico.json` | `Código Fonte — Série Histórica` | `nested_map` | `publicavel` | `entrada` | `media_por_senioridade` | `["media_por_senioridade", "ipca_fator_para_2026"]` |
| 32 | `so_benchmarks` | `data/so_benchmarks.json` | `Benchmarks Stack Overflow` | `object_sections` | `publicavel` | `saida` | `por_pais` | `["por_pais", "por_moeda_brasil", "serie_anual", "candidatas_a_referencia", "notas"]` |
| 33 | `salario_minimo` | `data/salario_minimo.json` | `Série do Salário Mínimo` | `object_sections` | `publicavel` | `saida` | `por_ano` | `["por_ano", "notas", "sm_por_ano", "deflator_ipca_por_ano", "cambio_por_ano"]` |
| 34 | `ibge_series` | `data/ibge_series.json` | `Séries IBGE (IPCA/INPC)` | `nested_map` | `publicavel` | `saida` | `series` | `["series"]` |
| 35 | `empresas_porte` | `data/empresas_porte.json` | `Porte das Empresas` | `root_map` | `publicavel` | `saida` | `_root` | `["_root"]` |
| 36 | `empresas_aliases` | `data/empresas_aliases.json` | `Aliases de Empresas` | `root_map` | `publicavel` | `saida` | `_root` | `["_root"]` |
| 37 | `empresas_linkedin_data` | `data/empresas_linkedin_data.json` | `Dados LinkedIn das Empresas` | `root_map` | `publicavel` | `saida` | `_root` | `["_root"]` |
| 38 | `empresas_linkedin_urls` | `data/empresas_linkedin_urls.json` | `URLs LinkedIn das Empresas` | `root_map` | `publicavel` | `saida` | `_root` | `["_root"]` |
| 39 | `mapa_mundi` | `data/mapa_mundi.json` | `Mapa-Múndi ( Geometria SVG)` | `object_sections` | `publicavel` | `saida` | `paths` | `["paths", "viewBox", "lat_range"]` |

## 3. Regras de Identidade e Propagação

- **Preservação do campo `"id"` textual**:
  - Cada `Record` retornado por `list_rows` / `get_row` inclui `"_row_id": <sqlite_id>` (usado pelas rotas `GET/PUT/DELETE /egressos-dados/rows/:id`).
  - Se o JSON original do registro possuir a propriedade `"id"` (ex.: `"id": "barbosa"` em `alunos.json`), ela permanece intacta como string em `record["id"]`.
- **Propagação Opcional (`propagate: bool`)**:
  - Grupo `Alumni`: propaga entre seções nominais identificadas pelo mesmo `id` (`alunos/alunos`, `_novos_reais/_root`, `consolidado_raw/_root`, `horizon_pesquisa/encontrados`, `lattes_orientacoes/encontrados`, `outros_labs/egressos`, `genero_map/_root`) ou pelo mesmo rótulo (`egressos_perfil/egressos`, `vitrine/perfis`, `panorama/coorte`, `panorama/salto`, `analise/rows`, `consolidado/perfis`).
  - Grupo `Company`: propaga entre `empresas_porte/_root`, `empresas_aliases/_root`, `empresas_linkedin_data/_root`, `empresas_linkedin_urls/_root` e `vitrine/{empresas,empresas_verificadas,contratando}`.
