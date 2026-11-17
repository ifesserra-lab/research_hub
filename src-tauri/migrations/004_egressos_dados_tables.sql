-- SEP-037: tabelas do domínio Egressos-Dados (39 arquivos JSON: alunos.json na raiz + 38 em data/*.json).
-- Mantém as 15 tabelas do Horizon, as 3 tabelas do SRC e as 2 tabelas do Egressos intocadas (FR-002).

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
