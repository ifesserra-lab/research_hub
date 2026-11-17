# Phase 1 Data Model: Aba Horizon (Nova Versão Canônica) e Preservação do Horizon Legacy

**Feature**: `039-horizon-v2-canonical-tab`  
**Date**: 2026-11-12

## 1. Visão Geral do Armazenamento Local (`hub.db` e `app_data_dir`)

O aplicativo mantém um único arquivo SQLite (`hub.db`) no diretório de dados do sistema operacional, com separação física completa entre **Horizon Legacy** e o novo **Horizon**:

| Recurso | Horizon Legacy (`project = "horizon"`) | Novo Horizon (`project = "horizon_v2"`) |
| :--- | :--- | :--- |
| **Rota de UI base** | `/dashboard` e `/dashboard/<entidade>` | `/horizon` e `/horizon/<entidade>` |
| **Prefixo na ponte IPC (`api.js`)** | `/{entidade}`, `/merge/{entidade}`, `/link` | `/horizon-v2/{entidade}`, `/horizon-v2/merge/{entidade}`, `/horizon-v2/link` |
| **Catálogo no Registry (`registry.rs`)** | `ENTITIES` (`by_route`) | `HORIZON_V2_ENTITIES` (`by_horizon_v2_route`) |
| **Tabelas SQLite** | `researchers`, `students`, ..., `research_productions` (migração `001_init.sql`) | `hv2_researchers`, `hv2_students`, ..., `hv2_research_productions` (migração `005_horizon_v2_tables.sql`) |
| **Arquivo ZIP original salvo** | `original.zip` | `horizon_v2_original.zip` |
| **Prefixo de snapshots** | `snapshots/hub-<timestamp>.db` | `snapshots/horizon-v2-<timestamp>.db` |
| **Chave em `sync_config.json`** | `projects["horizon"]` | `projects["horizon_v2"]` |

## 2. Tabelas SQLite do Novo Horizon (`005_horizon_v2_tables.sql`)

Assim como na migração `001_init.sql`, todas as colunas (exceto `id INTEGER PRIMARY KEY`) são armazenadas como `TEXT` no SQLite, e os tipos originais do JSON (`Bool`, `Int`, `Float`, `Text`, `Structure`) são restaurados na exportação a partir de `horizon_v2_original.zip`.

| Rota (`route`) | Tabela SQLite (`table`) | Arquivo Canônico no ZIP | Colunas (`columns`) | Coluna de Busca (`search_column`) | Linhas no Novo ZIP |
| :--- | :--- | :--- | :--- | :--- | ---: |
| `researchers` | `hv2_researchers` | `researchers_canonical.json` | `id`, `name`, `identification_id`, `birthday`, `cnpq_url`, `google_scholar_url`, `resume`, `citation_names`, `initiatives`, `research_groups`, `knowledge_areas`, `academic_education`, `articles`, `advisorships`, `classification`, `classification_confidence`, `classification_note`, `role_evidence`, `was_student`, `was_staff`, `campus` | `name` | 18.429 |
| `students` | `hv2_students` | `students_canonical.json` | *(mesmas 21 colunas de `PERSON_COLUMNS`)* | `name` | 11.799 |
| `articles` | `hv2_articles` | `articles_canonical.json` | `id`, `title`, `doi`, `year`, `type`, `journal_conference`, `volume`, `pages`, `campus` | `title` | 8.057 |
| `groups` | `hv2_research_groups` | `research_groups_canonical.json` | `id`, `name`, `description`, `short_name`, `organization_id`, `campus_id`, `cnpq_url`, `site`, `organization`, `campus`, `knowledge_areas`, `members`, `leaders` | `name` | 353 |
| `initiatives` | `hv2_initiatives` | `initiatives_canonical.json` | `id`, `name`, `status`, `description`, `start_date`, `end_date`, `initiative_type_id`, `initiative_type`, `organization_id`, `organization`, `parent_id`, `team`, `demandante`, `campus`, `research_group`, `knowledge_areas`, `enrichment`, `external_partner`, `external_research_group` | `name` | 15.505 |
| `advisorships` | `hv2_advisorships` | `advisorships_canonical.json` | `id`, `name`, `status`, `description`, `start_date`, `end_date`, `campus`, `advisorships`, `team` | `name` | 190 |
| `awards` | `hv2_awards` | `awards_canonical.json` | `id`, `researcher_id`, `title`, `year`, `campus` | `title` | 183 |
| `campuses` | `hv2_campuses` | `campuses_canonical.json` | `id`, `name`, `description`, `short_name`, `organization_id`, `parent_id` | `name` | 23 |
| `organizations` | `hv2_organizations` | `organizations_canonical.json` | `id`, `name`, `description`, `short_name`, `campus` | `name` | 440 |
| `fellowships` | `hv2_fellowships` | `fellowships_canonical.json` | `id`, `name`, `description`, `value`, `campus` | `name` | 21 |
| `proficiencies` | `hv2_proficiencies` | `proficiencies_canonical.json` | `id`, `researcher_id`, `language_id`, `comprehension`, `speaking`, `reading`, `writing`, `campus` | `None` (busca por ID numérico) | 794 |
| `professional_activities` | `hv2_professional_activities` | `professional_activities_canonical.json` | `id`, `researcher_id`, `organization_id`, `institution`, `institution_name`, `institution_acronym`, `institution_country`, `period`, `start_year`, `end_year`, `bond`, `classification`, `work_regime`, `role_function`, `activity_type`, `current`, `campus` | `institution_name` | 9.557 |
| `knowledge_areas` | `hv2_knowledge_areas` | `knowledge_areas_canonical.json` | `id`, `name`, `campus` | `name` | 1.563 |
| `languages` | `hv2_languages` | `languages_canonical.json` | `id`, `name`, `campus` | `name` | 11 |
| `research_productions` | `hv2_research_productions` | `research_productions_canonical.json` | `id`, `title`, `year`, `production_type_id`, `publisher`, `isbn`, `edition`, `book_title`, `pages`, `version`, `platform`, `link`, `campus` | `title` | 3.729 |

## 3. Estrutura do Bloco `initiatives.enrichment` (Novo Pacote Canônico)

Em `initiatives_canonical.json`, o campo `enrichment` é `null` quando a iniciativa não possui cruzamento com arquivo SIGPesq ou um objeto JSON (`ColType::Structure`) com os seguintes atributos:

```json
{
  "source": "sigpesq_project_files",
  "source_file": "PJ_6903.json",
  "project_code": "6903",
  "match_strategy": "sigpesq_project_code",
  "origin": "matched_existing",
  "extracted_at": "2026-11-12T00:00:00Z",
  "extraction_model": "...",
  "needs_review": false,
  "reviewed_at": null,
  "reviewed_by": null,
  "area_conhecimento": "Ciências Exatas e da Terra",
  "linha_pesquisa": "Inteligência Artificial",
  "palavras_chave": ["IA", "NLP"],
  "objetivos": {
    "geral": "Texto do objetivo geral...",
    "especificos": ["Objetivo específico 1", "Objetivo específico 2"]
  },
  "cronograma": [
    { "atividade": "...", "inicio": "...", "fim": "..." }
  ]
}
```

- **Armazenamento**: serializado como JSON compacto na coluna `TEXT` `enrichment` de `hv2_initiatives`.
- **Exibição na UI (`Horizon`)**: renderizado como campo `json_readonly` no formulário de Iniciativas, com formatação legível de sub-objetos (`objetivos`) e listas (`palavras_chave`, `cronograma`), preservado integralmente ao salvar e reemitido como objeto JSON estruturado na exportação.

## 4. Entradas Não Gerenciadas Preservadas Byte a Byte (683 Entradas no Novo ZIP)

Durante a exportação (`export_horizon_v2_zip` e `export_canonical_zip`), todas as 683 entradas do pacote `exports_canonical.zip` que não correspondem aos 15 arquivos `{tabela}_canonical.json` gerenciados são copiadas sem recompressão (`raw_copy_file`), incluindo:
- `export_manifest.json`
- `horizon_ids_canonical.json` (`rule_version: 5`, 75.804 IDs)
- `horizon_id_aliases_canonical.json` (`rule_version: 5`, 9.545 aliases)
- `source_document_states_canonical.json` (`rule_version: 5`, 2.377 documentos)
- `initiatives_analytics_mart.json` e `knowledge_areas_mart.json`
- `project_sigpesq_files_json/PJ_*.json` (297 arquivos)
- `research_group_relationship_graphs/*.json` (353 arquivos)
- Demais tabelas de linhagem, grafos e trackings (`attribute_assertions_canonical.json`, `source_records_canonical.json`, `entity_matches_canonical.json`, `entity_change_logs_canonical.json`, `advisorships_tracking.json`, `initiatives_tracking.json`, `researchers_tracking.json`, etc.).
