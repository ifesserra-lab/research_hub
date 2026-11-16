-- SEP-041: Tabela interna para gráficos personalizados criados pelo usuário
-- nos Dashboards das 5 abas (Horizon, Horizon Legacy, SRC, Egressos e Egressos — Dados).
-- Estritamente local (`exported: false`) — jamais altera os pacotes exportados (SC-004).

CREATE TABLE IF NOT EXISTS custom_dashboard_charts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scope TEXT NOT NULL,
    entity TEXT NOT NULL,
    title TEXT NOT NULL,
    group_by TEXT NOT NULL,
    metric TEXT NOT NULL DEFAULT 'count',
    metric_field TEXT NULL,
    kind TEXT NOT NULL DEFAULT 'bar',
    filter_logic TEXT NOT NULL DEFAULT 'and',
    filters_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT ''
);

CREATE INDEX IF NOT EXISTS idx_custom_charts_scope_entity
    ON custom_dashboard_charts(scope, entity);
