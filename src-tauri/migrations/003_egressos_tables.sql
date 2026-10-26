-- SEP-035: tabelas do domínio Egressos (pasta dados/ com 21 arquivos JSON).
-- Mantém as 15 tabelas do Horizon e as 3 tabelas do SRC intocadas (FR-002/FR-003).

CREATE TABLE IF NOT EXISTS egressos_files (
    file_key TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    primary_section TEXT NOT NULL,
    envelope_json TEXT NOT NULL,
    imported_at TEXT NULL
);

CREATE TABLE IF NOT EXISTS egressos_rows (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_key TEXT NOT NULL REFERENCES egressos_files(file_key) ON DELETE CASCADE,
    section_key TEXT NOT NULL,
    ord INTEGER NOT NULL,
    record_key TEXT NULL,
    search_text TEXT NOT NULL DEFAULT '',
    raw_json TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_egressos_rows_file_section_ord
    ON egressos_rows(file_key, section_key, ord);

CREATE INDEX IF NOT EXISTS idx_egressos_rows_record_key
    ON egressos_rows(record_key);
