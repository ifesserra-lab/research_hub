-- SEP-039. Isolated SQLite tables (`hv2_*`) for the new Horizon canonical workspace.
-- Every column TEXT except `id INTEGER PRIMARY KEY` (rowid alias): canonical JSON
-- types (including structured `enrichment` in `initiatives`) are restored on export
-- from `horizon_v2_original.zip`.

CREATE TABLE hv2_researchers (
  id INTEGER PRIMARY KEY, name TEXT, identification_id TEXT, birthday TEXT,
  cnpq_url TEXT, google_scholar_url TEXT, resume TEXT, citation_names TEXT,
  initiatives TEXT, research_groups TEXT, knowledge_areas TEXT,
  academic_education TEXT, articles TEXT, advisorships TEXT, classification TEXT,
  classification_confidence TEXT, classification_note TEXT, role_evidence TEXT,
  was_student TEXT, was_staff TEXT, campus TEXT
);

CREATE TABLE hv2_students (
  id INTEGER PRIMARY KEY, name TEXT, identification_id TEXT, birthday TEXT,
  cnpq_url TEXT, google_scholar_url TEXT, resume TEXT, citation_names TEXT,
  initiatives TEXT, research_groups TEXT, knowledge_areas TEXT,
  academic_education TEXT, articles TEXT, advisorships TEXT, classification TEXT,
  classification_confidence TEXT, classification_note TEXT, role_evidence TEXT,
  was_student TEXT, was_staff TEXT, campus TEXT
);

CREATE TABLE hv2_articles (
  id INTEGER PRIMARY KEY, title TEXT, doi TEXT, year TEXT, type TEXT,
  journal_conference TEXT, volume TEXT, pages TEXT, campus TEXT
);

CREATE TABLE hv2_research_groups (
  id INTEGER PRIMARY KEY, name TEXT, description TEXT, short_name TEXT,
  organization_id TEXT, campus_id TEXT, cnpq_url TEXT, site TEXT,
  organization TEXT, campus TEXT, knowledge_areas TEXT, members TEXT, leaders TEXT
);

CREATE TABLE hv2_initiatives (
  id INTEGER PRIMARY KEY, name TEXT, status TEXT, description TEXT,
  start_date TEXT, end_date TEXT, initiative_type_id TEXT, initiative_type TEXT,
  organization_id TEXT, organization TEXT, parent_id TEXT, team TEXT,
  demandante TEXT, campus TEXT, research_group TEXT, knowledge_areas TEXT,
  enrichment TEXT, external_partner TEXT, external_research_group TEXT
);

CREATE TABLE hv2_advisorships (
  id INTEGER PRIMARY KEY, name TEXT, status TEXT, description TEXT,
  start_date TEXT, end_date TEXT, campus TEXT, advisorships TEXT, team TEXT
);

CREATE TABLE hv2_awards (
  id INTEGER PRIMARY KEY, researcher_id TEXT, title TEXT, year TEXT, campus TEXT
);

CREATE TABLE hv2_campuses (
  id INTEGER PRIMARY KEY, name TEXT, description TEXT, short_name TEXT,
  organization_id TEXT, parent_id TEXT
);

CREATE TABLE hv2_organizations (
  id INTEGER PRIMARY KEY, name TEXT, description TEXT, short_name TEXT, campus TEXT
);

CREATE TABLE hv2_fellowships (
  id INTEGER PRIMARY KEY, name TEXT, description TEXT, value TEXT, campus TEXT
);

CREATE TABLE hv2_proficiencies (
  id INTEGER PRIMARY KEY, researcher_id TEXT, language_id TEXT, comprehension TEXT,
  speaking TEXT, reading TEXT, writing TEXT, campus TEXT
);

CREATE TABLE hv2_professional_activities (
  id INTEGER PRIMARY KEY, researcher_id TEXT, organization_id TEXT, institution TEXT,
  institution_name TEXT, institution_acronym TEXT, institution_country TEXT,
  period TEXT, start_year TEXT, end_year TEXT, bond TEXT, classification TEXT,
  work_regime TEXT, role_function TEXT, activity_type TEXT, current TEXT, campus TEXT
);

CREATE TABLE hv2_knowledge_areas (
  id INTEGER PRIMARY KEY, name TEXT, campus TEXT
);

CREATE TABLE hv2_languages (
  id INTEGER PRIMARY KEY, name TEXT, campus TEXT
);

CREATE TABLE hv2_research_productions (
  id INTEGER PRIMARY KEY, title TEXT, year TEXT, production_type_id TEXT,
  publisher TEXT, isbn TEXT, edition TEXT, book_title TEXT, pages TEXT,
  version TEXT, platform TEXT, link TEXT, campus TEXT
);
