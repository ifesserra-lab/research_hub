//! Motor de agregação analítica local, filtros estruturados e gráficos personalizados
//! (SEP-040 / SEP-041, Princípio VI).
//!
//! Calcula no processo Rust, diretamente sobre o SQLite local (`hub.db`),
//! todos os indicadores, selos de status de projetos, filtros por coluna (`AND`/`OR`)
//! e séries gráficas (padrão e personalizadas) para as 5 abas:
//! - Horizon (`hv2_*`) e Horizon Legacy (Dashboard Geral + 15 abas de entidades)
//! - SRC (`src_acoes` e `src_participacoes`)
//! - Egressos (21 tabelas JSON da pasta `dados/`)
//! - Egressos — Dados (39 datasets JSON de `ifesserra-lab/egressos-dados`)

use crate::crud::Record;
use crate::egressos_dados_domain::EGRESSOS_DADOS_FILES;
use crate::egressos_domain::EGRESSOS_FILES;
use crate::error::AppError;
use crate::registry;
use rusqlite::{params, types::Value as SqlValue, Connection};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value as Json};
use std::collections::{BTreeMap, HashMap};

fn default_filter_logic() -> String {
    "and".to_string()
}

fn default_metric() -> String {
    "count".to_string()
}

fn default_chart_kind() -> String {
    "bar".to_string()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnFilter {
    pub field: String,
    /// "contains" | "equals" | "not_equals" | "is_empty" | "not_empty"
    pub operator: String,
    #[serde(default)]
    pub value: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilterSpec {
    #[serde(default = "default_filter_logic")]
    pub logic: String,
    #[serde(default)]
    pub rules: Vec<ColumnFilter>,
}

impl FilterSpec {
    pub fn is_empty(&self) -> bool {
        self.rules
            .iter()
            .all(|r| r.field.trim().is_empty() || r.operator.trim().is_empty())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomDashboardChart {
    pub id: i64,
    pub scope: String,
    pub entity: String,
    pub title: String,
    pub group_by: String,
    pub metric: String,
    pub metric_field: Option<String>,
    pub kind: String,
    pub filter_logic: String,
    pub filters: Vec<ColumnFilter>,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewCustomChartInput {
    pub scope: String,
    pub entity: String,
    pub title: String,
    pub group_by: String,
    #[serde(default = "default_metric")]
    pub metric: String,
    pub metric_field: Option<String>,
    #[serde(default = "default_chart_kind")]
    pub kind: String,
    #[serde(default = "default_filter_logic")]
    pub filter_logic: String,
    #[serde(default)]
    pub filters: Vec<ColumnFilter>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityMetaOption {
    pub entity: String,
    pub label: String,
    pub columns: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectStatusInfo {
    pub raw_status: String,
    pub is_active: bool,
    pub macro_label: &'static str,
    pub badge_label: &'static str,
    pub tone: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectWorkspaceStatus {
    pub id: String,
    pub active: bool,
    pub total_rows: usize,
    pub active_projects: Option<usize>,
    pub status_label: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KpiCard {
    pub label: String,
    pub value: String,
    pub sublabel: Option<String>,
    pub tone: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChartItem {
    pub label: String,
    pub count: usize,
    pub secondary_value: Option<f64>,
    pub formatted_value: Option<String>,
    pub percentage: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChartSection {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub kind: String, // "donut" | "bar" | "columns" | "financial_table"
    pub items: Vec<ChartItem>,
    #[serde(default)]
    pub is_custom: bool,
    #[serde(default)]
    pub custom_id: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DashboardPayload {
    pub scope: String,
    pub entity: Option<String>,
    pub title: String,
    pub loaded: bool,
    pub total_rows: usize,
    pub kpis: Vec<KpiCard>,
    pub charts: Vec<ChartSection>,
    #[serde(default)]
    pub available_entities: Vec<EntityMetaOption>,
}

/// Faz o parse de um `FilterSpec` serializado em JSON (aceita também array direto de `ColumnFilter`).
pub fn parse_filter_spec(raw: Option<&str>) -> FilterSpec {
    let Some(s) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
        return FilterSpec::default();
    };
    if let Ok(spec) = serde_json::from_str::<FilterSpec>(s) {
        return spec;
    }
    if let Ok(rules) = serde_json::from_str::<Vec<ColumnFilter>>(s) {
        return FilterSpec {
            logic: "and".to_string(),
            rules,
        };
    }
    FilterSpec::default()
}

/// Extrai os valores legíveis de um campo (escalar ou JSON estruturado com `name`/`title`/`institution`/`degree`).
pub fn extract_field_tokens(val: Option<&Json>) -> Vec<String> {
    let Some(v) = val else {
        return Vec::new();
    };
    match v {
        Json::Null => Vec::new(),
        Json::Bool(b) => {
            if *b {
                vec!["true".to_string(), "sim".to_string(), "ativo".to_string(), "1".to_string()]
            } else {
                vec!["false".to_string(), "não".to_string(), "nao".to_string(), "inativo".to_string(), "0".to_string()]
            }
        }
        Json::Number(n) => vec![n.to_string()],
        Json::String(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() || trimmed == "null" || trimmed == "[]" {
                return Vec::new();
            }
            if (trimmed.starts_with('[') && trimmed.ends_with(']'))
                || (trimmed.starts_with('{') && trimmed.ends_with('}'))
            {
                if let Ok(parsed) = serde_json::from_str::<Json>(trimmed) {
                    let sub = extract_field_tokens(Some(&parsed));
                    if !sub.is_empty() {
                        return sub;
                    }
                }
            }
            let mut out = vec![trimmed.to_string()];
            let lower = trimmed.to_ascii_lowercase();
            if lower == "true" {
                out.push("sim".to_string());
                out.push("ativo".to_string());
            } else if lower == "false" {
                out.push("não".to_string());
                out.push("nao".to_string());
                out.push("inativo".to_string());
            } else if lower == "active" || lower == "in progress" {
                out.push("ativo".to_string());
                out.push(classify_project_status(Some(trimmed)).badge_label.to_string());
            } else if lower == "concluded" || lower == "cancelled" || lower == "unknown" {
                out.push("inativo".to_string());
                out.push(classify_project_status(Some(trimmed)).badge_label.to_string());
            }
            out
        }
        Json::Array(arr) => {
            let mut out = Vec::new();
            for item in arr {
                out.extend(extract_field_tokens(Some(item)));
            }
            out
        }
        Json::Object(obj) => {
            let mut out = Vec::new();
            for key in [
                "name",
                "title",
                "institution",
                "institution_name",
                "degree",
                "project_code",
                "short_name",
                "supervisor_name",
                "person_name",
            ] {
                if let Some(sub) = obj.get(key).and_then(|x| x.as_str()).map(str::trim) {
                    if !sub.is_empty() {
                        out.push(sub.to_string());
                    }
                }
            }
            if out.is_empty() {
                if let Ok(raw) = serde_json::to_string(obj) {
                    out.push(raw);
                }
            }
            out
        }
    }
}

/// Extrai as categorias primárias de agrupamento (`group_by`) de um valor de campo.
pub fn extract_group_labels(field: &str, val: Option<&Json>) -> Vec<String> {
    let Some(v) = val else {
        return vec!["Não informado".to_string()];
    };
    if field == "status" {
        if let Some(s) = v.as_str() {
            return vec![classify_project_status(Some(s)).badge_label.to_string()];
        }
    }
    match v {
        Json::Null => vec!["Não informado".to_string()],
        Json::Bool(b) => vec![if *b { "Sim".to_string() } else { "Não".to_string() }],
        Json::Number(n) => vec![n.to_string()],
        Json::String(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() || trimmed == "null" || trimmed == "[]" {
                return vec!["Não informado".to_string()];
            }
            if (trimmed.starts_with('[') && trimmed.ends_with(']'))
                || (trimmed.starts_with('{') && trimmed.ends_with('}'))
            {
                if let Ok(parsed) = serde_json::from_str::<Json>(trimmed) {
                    return extract_group_labels(field, Some(&parsed));
                }
            }
            vec![trimmed.to_string()]
        }
        Json::Array(arr) => {
            if arr.is_empty() {
                return vec!["Sem vínculo".to_string()];
            }
            let mut out = Vec::new();
            for item in arr {
                match item {
                    Json::Object(obj) => {
                        if let Some(n) = obj
                            .get("name")
                            .or_else(|| obj.get("title"))
                            .or_else(|| obj.get("institution"))
                            .or_else(|| obj.get("degree"))
                            .and_then(|x| x.as_str())
                            .map(str::trim)
                            .filter(|s| !s.is_empty())
                        {
                            if field == "academic_education" {
                                if let Some(clean) = clean_education_institution(Some(n)) {
                                    out.push(clean);
                                }
                            } else {
                                out.push(n.to_string());
                            }
                        }
                    }
                    Json::String(s) if !s.trim().is_empty() => out.push(s.trim().to_string()),
                    _ => {}
                }
            }
            if out.is_empty() {
                vec![format!("{} vínculos", arr.len())]
            } else {
                out
            }
        }
        Json::Object(obj) => {
            if let Some(n) = obj
                .get("name")
                .or_else(|| obj.get("title"))
                .or_else(|| obj.get("project_code"))
                .and_then(|x| x.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                vec![n.to_string()]
            } else {
                vec!["Objeto estruturado".to_string()]
            }
        }
    }
}

/// Extrai valor numérico para a métrica `sum` (soma valor numérico ou quantidade de elementos em array JSON de vínculos).
pub fn extract_numeric_or_link_value(val: Option<&Json>) -> f64 {
    let Some(v) = val else {
        return 0.0;
    };
    match v {
        Json::Number(n) => n.as_f64().unwrap_or(0.0),
        Json::Bool(b) => if *b { 1.0 } else { 0.0 },
        Json::Array(arr) => arr.len() as f64,
        Json::Object(_) => 1.0,
        Json::String(s) => {
            let trimmed = s.trim();
            if let Ok(num) = trimmed.replace(',', ".").parse::<f64>() {
                return num;
            }
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                if let Ok(Json::Array(arr)) = serde_json::from_str::<Json>(trimmed) {
                    return arr.len() as f64;
                }
            }
            if trimmed.starts_with('{') && trimmed.ends_with('}') {
                return 1.0;
            }
            0.0
        }
        Json::Null => 0.0,
    }
}

fn rule_matches_record(record: &Record, rule: &ColumnFilter) -> bool {
    let field = rule.field.trim();
    if field.is_empty() {
        return true;
    }
    let op = rule.operator.trim().to_ascii_lowercase();
    let target = rule.value.trim().to_lowercase();
    let tokens = extract_field_tokens(record.get(field));
    let non_empty = !tokens.is_empty();

    match op.as_str() {
        "is_empty" | "empty" => !non_empty,
        "not_empty" => non_empty,
        "equals" | "eq" | "=" => {
            if target.is_empty() {
                !non_empty
            } else {
                tokens.iter().any(|t| t.to_lowercase() == target)
            }
        }
        "not_equals" | "neq" | "!=" => {
            if target.is_empty() {
                non_empty
            } else {
                !tokens.iter().any(|t| t.to_lowercase() == target)
            }
        }
        _ => {
            // default: "contains"
            if target.is_empty() {
                true
            } else {
                tokens.iter().any(|t| t.to_lowercase().contains(&target))
            }
        }
    }
}

/// Avalia se um `Record` satisfaz todas (`logic == "and"`) ou qualquer uma (`logic == "or"`) das regras de `FilterSpec`.
pub fn record_matches_filter_spec(record: &Record, spec: &FilterSpec) -> bool {
    let active_rules: Vec<&ColumnFilter> = spec
        .rules
        .iter()
        .filter(|r| !r.field.trim().is_empty() && !r.operator.trim().is_empty())
        .collect();
    if active_rules.is_empty() {
        return true;
    }
    if spec.logic.trim().eq_ignore_ascii_case("or") {
        active_rules.iter().any(|r| rule_matches_record(record, r))
    } else {
        active_rules.iter().all(|r| rule_matches_record(record, r))
    }
}

/// Regra canônica de classificação de status de projeto (FR-004, FR-005, R2).
pub fn classify_project_status(raw: Option<&str>) -> ProjectStatusInfo {
    let trimmed = raw.unwrap_or("").trim();
    match trimmed.to_ascii_lowercase().as_str() {
        "active" | "ativo" | "em execução" | "em execucao" => ProjectStatusInfo {
            raw_status: trimmed.to_string(),
            is_active: true,
            macro_label: "Ativo",
            badge_label: "Ativo (Em Execução)",
            tone: "emerald",
        },
        "in progress" | "em andamento" | "andamento" => ProjectStatusInfo {
            raw_status: trimmed.to_string(),
            is_active: true,
            macro_label: "Ativo",
            badge_label: "Ativo (Em Andamento)",
            tone: "teal",
        },
        "concluded" | "concluído" | "concluido" | "encerrado" => ProjectStatusInfo {
            raw_status: trimmed.to_string(),
            is_active: false,
            macro_label: "Inativo",
            badge_label: "Inativo (Concluído)",
            tone: "slate",
        },
        "cancelled" | "canceled" | "cancelado" | "suspenso" => ProjectStatusInfo {
            raw_status: trimmed.to_string(),
            is_active: false,
            macro_label: "Inativo",
            badge_label: "Inativo (Cancelado)",
            tone: "rose",
        },
        _ => ProjectStatusInfo {
            raw_status: if trimmed.is_empty() {
                "Unknown".to_string()
            } else {
                trimmed.to_string()
            },
            is_active: false,
            macro_label: "Inativo",
            badge_label: "Inativo (Não Informado)",
            tone: "amber",
        },
    }
}

/// Filtra ruídos de importação em `academic_education.institution` (ex.: `"(Carga Horária: 360h)"`).
pub fn clean_education_institution(raw: Option<&str>) -> Option<String> {
    let s = raw?.trim();
    if s.is_empty() {
        return None;
    }
    let lower = s.to_lowercase();
    if lower.starts_with("(carga hor")
        || lower.starts_with("carga hor")
        || lower == "n/a"
        || lower == "null"
        || lower == "none"
        || lower == "-"
    {
        return None;
    }
    Some(s.to_string())
}

pub fn format_pt_int(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push('.');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

pub fn format_brl(value: f64) -> String {
    let cents = (value.max(0.0) * 100.0).round() as u64;
    let integer_part = (cents / 100) as usize;
    let frac_part = cents % 100;
    format!("R$ {},{:02}", format_pt_int(integer_part), frac_part)
}

fn pct(part: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        ((part as f64 / total as f64) * 1000.0).round() / 10.0
    }
}

fn table_row_count(conn: &Connection, table: &str) -> Result<usize, AppError> {
    let n: i64 = conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))?;
    Ok(n.max(0) as usize)
}

fn active_initiatives_count(conn: &Connection, table: &str) -> Result<usize, AppError> {
    let n: i64 = conn.query_row(
        &format!(
            "SELECT COUNT(*) FROM {table} WHERE LOWER(TRIM(COALESCE(status, ''))) IN ('active', 'in progress', 'ativo', 'em andamento')"
        ),
        [],
        |r| r.get(0),
    )?;
    Ok(n.max(0) as usize)
}

/// Retorna o status de ativação (base carregada + contagens) dos 5 ambientes para `/projects` (FR-004).
pub fn compute_projects_status(conn: &Connection) -> Result<Vec<ProjectWorkspaceStatus>, AppError> {
    let mut hv2_total = 0usize;
    for def in registry::HORIZON_V2_ENTITIES {
        hv2_total += table_row_count(conn, def.table)?;
    }
    let hv2_active_init = active_initiatives_count(conn, "hv2_initiatives")?;

    let mut legacy_total = 0usize;
    for def in registry::exported() {
        legacy_total += table_row_count(conn, def.table)?;
    }
    let legacy_active_init = active_initiatives_count(conn, "initiatives")?;

    let src_acoes = table_row_count(conn, "src_acoes")?;
    let src_part = table_row_count(conn, "src_participacoes")?;
    let src_total = src_acoes + src_part;

    let egressos_rows = table_row_count(conn, "egressos_rows")?;
    let egressos_files = table_row_count(conn, "egressos_files")?;

    let ed_rows = table_row_count(conn, "egressos_dados_rows")?;
    let ed_files = table_row_count(conn, "egressos_dados_files")?;

    Ok(vec![
        ProjectWorkspaceStatus {
            id: "horizon-v2".to_string(),
            active: hv2_total > 0,
            total_rows: hv2_total,
            active_projects: Some(hv2_active_init),
            status_label: if hv2_total > 0 {
                format!(
                    "Ativo • {} registros ({} projetos ativos)",
                    format_pt_int(hv2_total),
                    format_pt_int(hv2_active_init)
                )
            } else {
                "Inativo • Sem dados carregados".to_string()
            },
        },
        ProjectWorkspaceStatus {
            id: "horizon".to_string(),
            active: legacy_total > 0,
            total_rows: legacy_total,
            active_projects: Some(legacy_active_init),
            status_label: if legacy_total > 0 {
                format!(
                    "Ativo • {} registros ({} projetos ativos)",
                    format_pt_int(legacy_total),
                    format_pt_int(legacy_active_init)
                )
            } else {
                "Inativo • Sem dados carregados".to_string()
            },
        },
        ProjectWorkspaceStatus {
            id: "src".to_string(),
            active: src_total > 0,
            total_rows: src_total,
            active_projects: Some(src_acoes),
            status_label: if src_total > 0 {
                format!(
                    "Ativo • {} ações ({} participações)",
                    format_pt_int(src_acoes),
                    format_pt_int(src_part)
                )
            } else {
                "Inativo • Sem dados carregados".to_string()
            },
        },
        ProjectWorkspaceStatus {
            id: "egressos".to_string(),
            active: egressos_rows > 0,
            total_rows: egressos_rows,
            active_projects: None,
            status_label: if egressos_rows > 0 {
                format!(
                    "Ativo • {} registros ({} arquivos)",
                    format_pt_int(egressos_rows),
                    egressos_files
                )
            } else {
                "Inativo • Sem dados carregados".to_string()
            },
        },
        ProjectWorkspaceStatus {
            id: "egressos-dados".to_string(),
            active: ed_rows > 0,
            total_rows: ed_rows,
            active_projects: None,
            status_label: if ed_rows > 0 {
                format!(
                    "Ativo • {} registros ({} datasets)",
                    format_pt_int(ed_rows),
                    ed_files
                )
            } else {
                "Inativo • Sem dados carregados".to_string()
            },
        },
    ])
}

fn entity_pt_label(route: &str) -> &'static str {
    match route {
        "researchers" => "Pesquisadores",
        "students" => "Alunos",
        "initiatives" => "Iniciativas (Projetos)",
        "professional_activities" => "Atividades Profissionais",
        "articles" => "Artigos",
        "research_productions" => "Produções Científicas",
        "knowledge_areas" => "Áreas de Conhecimento",
        "proficiencies" => "Proficiências",
        "organizations" => "Organizações",
        "groups" => "Grupos de Pesquisa",
        "advisorships" => "Orientações",
        "awards" => "Premiações",
        "campuses" => "Campus",
        "fellowships" => "Bolsas",
        "languages" => "Idiomas",
        _ => "Registros",
    }
}

fn load_table_records_filtered(
    conn: &Connection,
    def: &registry::EntityDef,
    spec: &FilterSpec,
) -> Result<Vec<Record>, AppError> {
    let cols = def.columns.join(", ");
    let mut stmt = conn.prepare(&format!("SELECT {cols} FROM {}", def.table))?;
    let rows = stmt.query_map([], |r| {
        let mut rec = Map::with_capacity(def.columns.len());
        for (i, name) in def.columns.iter().enumerate() {
            let v: SqlValue = r.get(i)?;
            let jv = match v {
                SqlValue::Null => Json::Null,
                SqlValue::Integer(n) => Json::from(n),
                SqlValue::Real(f) => Json::from(f),
                SqlValue::Text(s) => Json::from(s),
                SqlValue::Blob(_) => Json::Null,
            };
            rec.insert((*name).to_string(), jv);
        }
        Ok(rec)
    })?;

    let mut out = Vec::new();
    for r in rows {
        let rec = r?;
        if record_matches_filter_spec(&rec, spec) {
            out.push(rec);
        }
    }
    Ok(out)
}

fn load_records_for_scope_entity(
    conn: &Connection,
    scope: &str,
    entity: &str,
    spec: &FilterSpec,
) -> Result<Vec<Record>, AppError> {
    match scope {
        "horizon-v2" => {
            let def = registry::by_horizon_v2_route(entity)?;
            load_table_records_filtered(conn, def, spec)
        }
        "horizon" => {
            let def = registry::by_route(entity)?;
            load_table_records_filtered(conn, def, spec)
        }
        "src" => {
            let def = registry::by_src_route("src_acoes")?;
            load_table_records_filtered(conn, def, spec)
        }
        "egressos" => {
            let mut stmt = conn.prepare(
                "SELECT id, record_key, raw_json FROM egressos_rows WHERE file_key = ?1",
            )?;
            let rows = stmt.query_map([entity], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })?;
            let mut out = Vec::new();
            for row in rows.flatten() {
                if let Ok(Json::Object(mut obj)) = serde_json::from_str::<Json>(&row.2) {
                    obj.insert("id".to_string(), Json::from(row.0));
                    if let Some(rk) = row.1 {
                        obj.insert("_record_key".to_string(), Json::String(rk));
                    }
                    if record_matches_filter_spec(&obj, spec) {
                        out.push(obj);
                    }
                }
            }
            Ok(out)
        }
        "egressos-dados" => {
            let mut stmt = conn.prepare(
                "SELECT id, record_key, raw_json FROM egressos_dados_rows WHERE file_key = ?1",
            )?;
            let rows = stmt.query_map([entity], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })?;
            let mut out = Vec::new();
            for row in rows.flatten() {
                if let Ok(Json::Object(mut obj)) = serde_json::from_str::<Json>(&row.2) {
                    obj.insert("_row_id".to_string(), Json::from(row.0));
                    if let Some(rk) = row.1 {
                        obj.insert("_record_key".to_string(), Json::String(rk));
                    }
                    if record_matches_filter_spec(&obj, spec) {
                        out.push(obj);
                    }
                }
            }
            Ok(out)
        }
        other => Err(AppError::Validation(format!("Escopo desconhecido: {other}"))),
    }
}

pub fn available_entities_for_scope(scope: &str) -> Vec<EntityMetaOption> {
    match scope {
        "horizon-v2" => registry::HORIZON_V2_ENTITIES
            .iter()
            .map(|d| EntityMetaOption {
                entity: d.route.to_string(),
                label: format!("{} ({})", entity_pt_label(d.route), d.canonical_table()),
                columns: d.columns.iter().map(|c| (*c).to_string()).collect(),
            })
            .collect(),
        "horizon" => registry::ENTITIES
            .iter()
            .filter(|d| d.exported)
            .map(|d| EntityMetaOption {
                entity: d.route.to_string(),
                label: format!("{} ({})", entity_pt_label(d.route), d.table),
                columns: d.columns.iter().map(|c| (*c).to_string()).collect(),
            })
            .collect(),
        "src" => vec![EntityMetaOption {
            entity: "src_acoes".to_string(),
            label: "Ações do SRC (src_acoes)".to_string(),
            columns: vec![
                "id",
                "acao_id",
                "processo",
                "titulo",
                "natureza",
                "tipo",
                "coordenador",
                "acao_vinculante",
                "campus",
                "total_participacoes",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        }],
        "egressos" => EGRESSOS_FILES
            .iter()
            .map(|f| EntityMetaOption {
                entity: f.file_key.to_string(),
                label: format!("{} ({})", f.title, f.file_name),
                columns: f.columns.iter().map(|c| (*c).to_string()).collect(),
            })
            .collect(),
        "egressos-dados" => EGRESSOS_DADOS_FILES
            .iter()
            .map(|f| EntityMetaOption {
                entity: f.file_key.to_string(),
                label: format!("{} ({})", f.title, f.file_name),
                columns: f.columns.iter().map(|c| (*c).to_string()).collect(),
            })
            .collect(),
        _ => Vec::new(),
    }
}

// ------------------------------------------------ Gráficos Personalizados (SEP-041)

pub fn create_custom_chart(
    conn: &Connection,
    input: &NewCustomChartInput,
) -> Result<CustomDashboardChart, AppError> {
    let scope = input.scope.trim();
    let entity = input.entity.trim();
    let group_by = input.group_by.trim();
    if scope.is_empty() || entity.is_empty() || group_by.is_empty() {
        return Err(AppError::Validation(
            "Escopo, tabela e coluna de agrupamento são obrigatórios.".to_string(),
        ));
    }
    let metric = if input.metric.trim().eq_ignore_ascii_case("sum") {
        "sum"
    } else {
        "count"
    };
    let kind = match input.kind.trim() {
        "donut" | "columns" | "financial_table" => input.kind.trim(),
        _ => "bar",
    };
    let filter_logic = if input.filter_logic.trim().eq_ignore_ascii_case("or") {
        "or"
    } else {
        "and"
    };
    let title = if input.title.trim().is_empty() {
        format!("{entity} por {group_by}")
    } else {
        input.title.trim().to_string()
    };
    let filters_json = serde_json::to_string(&input.filters)
        .map_err(|e| AppError::Internal(format!("falha ao serializar filtros: {e}")))?;
    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_default();

    conn.execute(
        "INSERT INTO custom_dashboard_charts
         (scope, entity, title, group_by, metric, metric_field, kind, filter_logic, filters_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            scope,
            entity,
            title,
            group_by,
            metric,
            input.metric_field.as_deref(),
            kind,
            filter_logic,
            filters_json,
            created_at,
        ],
    )?;
    let id = conn.last_insert_rowid();

    Ok(CustomDashboardChart {
        id,
        scope: scope.to_string(),
        entity: entity.to_string(),
        title,
        group_by: group_by.to_string(),
        metric: metric.to_string(),
        metric_field: input.metric_field.clone(),
        kind: kind.to_string(),
        filter_logic: filter_logic.to_string(),
        filters: input.filters.clone(),
        created_at,
    })
}

pub fn list_custom_charts(
    conn: &Connection,
    scope: &str,
    entity: Option<&str>,
) -> Result<Vec<CustomDashboardChart>, AppError> {
    let clean_entity = entity.map(str::trim).filter(|s| !s.is_empty());
    let mut out = Vec::new();

    if let Some(ent) = clean_entity {
        let mut stmt = conn.prepare(
            "SELECT id, scope, entity, title, group_by, metric, metric_field, kind, filter_logic, filters_json, created_at
             FROM custom_dashboard_charts
             WHERE scope = ?1 AND entity = ?2
             ORDER BY id DESC",
        )?;
        let rows = stmt.query_map(params![scope.trim(), ent], row_to_custom_chart)?;
        for r in rows {
            out.push(r?);
        }
    } else {
        let mut stmt = conn.prepare(
            "SELECT id, scope, entity, title, group_by, metric, metric_field, kind, filter_logic, filters_json, created_at
             FROM custom_dashboard_charts
             WHERE scope = ?1
             ORDER BY id DESC",
        )?;
        let rows = stmt.query_map(params![scope.trim()], row_to_custom_chart)?;
        for r in rows {
            out.push(r?);
        }
    }
    Ok(out)
}

fn row_to_custom_chart(r: &rusqlite::Row) -> rusqlite::Result<CustomDashboardChart> {
    let filters_str: String = r.get(9)?;
    let filters: Vec<ColumnFilter> = serde_json::from_str(&filters_str).unwrap_or_default();
    Ok(CustomDashboardChart {
        id: r.get(0)?,
        scope: r.get(1)?,
        entity: r.get(2)?,
        title: r.get(3)?,
        group_by: r.get(4)?,
        metric: r.get(5)?,
        metric_field: r.get(6)?,
        kind: r.get(7)?,
        filter_logic: r.get(8)?,
        filters,
        created_at: r.get(10)?,
    })
}

pub fn delete_custom_chart(conn: &Connection, id: i64) -> Result<(), AppError> {
    conn.execute("DELETE FROM custom_dashboard_charts WHERE id = ?1", [id])?;
    Ok(())
}

pub fn compute_custom_chart_section(
    conn: &Connection,
    cfg: &CustomDashboardChart,
    global_spec: &FilterSpec,
) -> Result<ChartSection, AppError> {
    let chart_spec = FilterSpec {
        logic: cfg.filter_logic.clone(),
        rules: cfg.filters.clone(),
    };
    let records = load_records_for_scope_entity(conn, &cfg.scope, &cfg.entity, &chart_spec)?;
    let filtered_records: Vec<&Record> = records
        .iter()
        .filter(|rec| {
            global_spec.is_empty()
                || !rec.contains_key(&global_spec.rules.first().map(|r| r.field.clone()).unwrap_or_default())
                || record_matches_filter_spec(rec, global_spec)
        })
        .collect();

    let is_sum = cfg.metric.eq_ignore_ascii_case("sum") && cfg.metric_field.is_some();
    let metric_col = cfg.metric_field.as_deref().unwrap_or("");
    let mut agg_map: HashMap<String, (usize, f64)> = HashMap::new();

    for rec in &filtered_records {
        let labels = extract_group_labels(&cfg.group_by, rec.get(&cfg.group_by));
        let add_val = if is_sum {
            extract_numeric_or_link_value(rec.get(metric_col))
        } else {
            1.0
        };
        for lb in labels {
            let entry = agg_map.entry(lb).or_insert((0, 0.0));
            entry.0 += 1;
            entry.1 += add_val;
        }
    }

    let mut vec: Vec<(String, usize, f64)> = agg_map
        .into_iter()
        .map(|(k, (c, s))| (k, c, s))
        .collect();
    if is_sum {
        vec.sort_by(|a, b| {
            b.2.partial_cmp(&a.2)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.1.cmp(&a.1))
        });
    } else {
        vec.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    }

    let total_count: usize = vec.iter().map(|(_, c, _)| *c).sum();
    let max_sum = vec.first().map(|x| x.2).unwrap_or(1.0).max(1.0);

    let items: Vec<ChartItem> = vec
        .into_iter()
        .take(15)
        .map(|(label, count, sum_val)| {
            if is_sum {
                let rounded = sum_val.round() as usize;
                ChartItem {
                    label,
                    count: rounded,
                    secondary_value: Some(sum_val),
                    formatted_value: Some(format!(
                        "Soma: {} ({} reg.)",
                        format_pt_int(rounded),
                        format_pt_int(count)
                    )),
                    percentage: ((sum_val / max_sum) * 1000.0).round() / 10.0,
                }
            } else {
                ChartItem {
                    label,
                    count,
                    secondary_value: None,
                    formatted_value: Some(format!("{} registros", format_pt_int(count))),
                    percentage: pct(count, total_count),
                }
            }
        })
        .collect();

    let filter_desc = if cfg.filters.is_empty() {
        format!("Tabela: {} • Agrupado por: {}", cfg.entity, cfg.group_by)
    } else {
        let rules_txt: Vec<String> = cfg
            .filters
            .iter()
            .map(|r| format!("{} {} '{}'", r.field, r.operator, r.value))
            .collect();
        format!(
            "Tabela: {} • Agrupado por: {} • Filtro ({}): {}",
            cfg.entity,
            cfg.group_by,
            cfg.filter_logic.to_uppercase(),
            rules_txt.join(", ")
        )
    };

    Ok(ChartSection {
        id: format!("custom_chart_{}", cfg.id),
        title: cfg.title.clone(),
        subtitle: Some(filter_desc),
        kind: cfg.kind.clone(),
        items,
        is_custom: true,
        custom_id: Some(cfg.id),
    })
}

fn append_custom_charts(
    conn: &Connection,
    scope: &str,
    entity: Option<&str>,
    global_spec: &FilterSpec,
    charts: &mut Vec<ChartSection>,
) -> Result<(), AppError> {
    let custom_list = list_custom_charts(conn, scope, entity)?;
    let mut custom_sections = Vec::with_capacity(custom_list.len());
    for cfg in &custom_list {
        if let Ok(sec) = compute_custom_chart_section(conn, cfg, global_spec) {
            custom_sections.push(sec);
        }
    }
    // Coloca os gráficos personalizados no topo para destaque imediato
    if !custom_sections.is_empty() {
        custom_sections.append(charts);
        *charts = custom_sections;
    }
    Ok(())
}

// ------------------------------------------------ Helpers de Agregação Padrão

fn top_items_from_counts(
    counts: HashMap<String, usize>,
    total_base: usize,
    limit: usize,
    unit: &str,
) -> Vec<ChartItem> {
    let mut vec: Vec<(String, usize)> = counts
        .into_iter()
        .filter(|(k, c)| !k.trim().is_empty() && *c > 0)
        .collect();
    vec.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let denom = if total_base > 0 {
        total_base
    } else {
        vec.iter().map(|(_, c)| *c).sum()
    };
    vec.into_iter()
        .take(limit)
        .map(|(label, count)| ChartItem {
            label,
            count,
            secondary_value: None,
            formatted_value: Some(format!("{} {}", format_pt_int(count), unit)),
            percentage: pct(count, denom),
        })
        .collect()
}

fn extract_named_entities_from_records(
    records: &[Record],
    col: &str,
) -> (HashMap<String, usize>, usize) {
    let mut counts: HashMap<String, usize> = HashMap::new();
    let mut total_links = 0usize;
    for rec in records {
        for lb in extract_group_labels(col, rec.get(col)) {
            if lb != "Não informado" && lb != "Sem vínculo" {
                *counts.entry(lb).or_insert(0) += 1;
                total_links += 1;
            }
        }
    }
    (counts, total_links)
}

#[derive(Default)]
struct AdvisorshipFinancialSummary {
    total_advisorships: usize,
    active_advisorships: usize,
    paid_count: usize,
    volunteer_count: usize,
    total_monthly_investment: f64,
    program_counts: HashMap<String, (usize, f64)>,
    supervisors: HashMap<String, (usize, usize, f64)>,
    recipients: HashMap<String, (usize, f64, String)>,
}

fn compute_advisorship_financials_from_records(records: &[Record]) -> AdvisorshipFinancialSummary {
    let mut summary = AdvisorshipFinancialSummary::default();
    for rec in records {
        let Some(raw) = rec.get("advisorships").and_then(|v| v.as_str()) else {
            continue;
        };
        let Ok(Json::Array(items)) = serde_json::from_str::<Json>(raw) else {
            continue;
        };
        for item in items {
            summary.total_advisorships += 1;
            let st = item.get("status").and_then(|v| v.as_str());
            if classify_project_status(st).is_active {
                summary.active_advisorships += 1;
            }
            let sup_name = item
                .get("supervisor_name")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or("Orientador não informado")
                .to_string();
            let person_name = item
                .get("person_name")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .unwrap_or("Estudante não informado")
                .to_string();

            let (prog_name, val) = match item.get("fellowship") {
                Some(Json::Object(f)) => {
                    let name = f
                        .get("name")
                        .and_then(|v| v.as_str())
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .unwrap_or("Sem Programa")
                        .to_string();
                    let value = f
                        .get("value")
                        .and_then(|v| v.as_f64().or_else(|| v.as_str()?.parse::<f64>().ok()))
                        .unwrap_or(0.0);
                    (Some(name), value)
                }
                _ => (None, 0.0),
            };

            let sup_entry = summary.supervisors.entry(sup_name).or_insert((0, 0, 0.0));
            sup_entry.0 += 1;

            if let Some(prog) = prog_name {
                let p_entry = summary.program_counts.entry(prog.clone()).or_insert((0, 0.0));
                p_entry.0 += 1;
                p_entry.1 += val;
                if val > 0.0 {
                    summary.paid_count += 1;
                    summary.total_monthly_investment += val;
                    sup_entry.1 += 1;
                    sup_entry.2 += val;

                    let rec_entry = summary
                        .recipients
                        .entry(person_name)
                        .or_insert((0, 0.0, prog.clone()));
                    rec_entry.0 += 1;
                    rec_entry.1 += val;
                    rec_entry.2 = prog;
                } else {
                    summary.volunteer_count += 1;
                }
            } else {
                summary.volunteer_count += 1;
            }
        }
    }
    summary
}

fn degree_rank(deg: &str) -> u8 {
    let lower = deg.to_lowercase();
    if lower.contains("pós-doutorado") || lower.contains("pos-doutorado") {
        6
    } else if lower.contains("doutorado") {
        5
    } else if lower.contains("mestrado") {
        4
    } else if lower.contains("especializa") || lower.contains("resid") {
        3
    } else if lower.contains("gradua") {
        2
    } else if lower.contains("técnico") || lower.contains("tecnico") || lower.contains("aperfei") {
        1
    } else {
        0
    }
}

fn normalize_degree_label(deg: &str) -> String {
    let lower = deg.to_lowercase();
    if lower.contains("pós-doutorado") || lower.contains("pos-doutorado") {
        "Pós-Doutorado".to_string()
    } else if lower.contains("doutorado") {
        "Doutorado".to_string()
    } else if lower.contains("mestrado") {
        "Mestrado".to_string()
    } else if lower.contains("especializa") {
        "Especialização".to_string()
    } else if lower.contains("gradua") {
        "Graduação".to_string()
    } else if lower.contains("técnico") || lower.contains("tecnico") {
        "Curso Técnico".to_string()
    } else {
        deg.trim().to_string()
    }
}

fn compute_education_and_degrees_from_records(
    records: &[Record],
) -> (HashMap<String, usize>, HashMap<String, usize>) {
    let mut inst_counts: HashMap<String, usize> = HashMap::new();
    let mut highest_degrees: HashMap<String, usize> = HashMap::new();

    for rec in records {
        let Some(raw) = rec.get("academic_education").and_then(|v| v.as_str()) else {
            continue;
        };
        let Ok(Json::Array(items)) = serde_json::from_str::<Json>(raw) else {
            continue;
        };
        let mut best_deg: Option<(u8, String)> = None;
        let mut seen_inst_for_person = std::collections::HashSet::new();
        for item in items {
            if let Some(inst) =
                clean_education_institution(item.get("institution").and_then(|v| v.as_str()))
            {
                if seen_inst_for_person.insert(inst.clone()) {
                    *inst_counts.entry(inst).or_insert(0) += 1;
                }
            }
            if let Some(deg) = item
                .get("degree")
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                let rank = degree_rank(deg);
                let label = normalize_degree_label(deg);
                if best_deg.as_ref().map(|(r, _)| rank > *r).unwrap_or(true) {
                    best_deg = Some((rank, label));
                }
            }
        }
        if let Some((_, label)) = best_deg {
            *highest_degrees.entry(label).or_insert(0) += 1;
        }
    }
    (inst_counts, highest_degrees)
}

fn compute_status_breakdown_from_records(
    records: &[Record],
) -> (usize, usize, usize, usize, Vec<ChartItem>) {
    let mut active_exec = 0usize;
    let mut active_prog = 0usize;
    let mut concluded = 0usize;
    let mut cancelled = 0usize;
    let mut unknown = 0usize;
    let total = records.len();

    for rec in records {
        let st = rec.get("status").and_then(|v| v.as_str());
        let info = classify_project_status(st);
        match info.badge_label {
            "Ativo (Em Execução)" => active_exec += 1,
            "Ativo (Em Andamento)" => active_prog += 1,
            "Inativo (Concluído)" => concluded += 1,
            "Inativo (Cancelado)" => cancelled += 1,
            _ => unknown += 1,
        }
    }

    let mut items = Vec::new();
    for (label, count) in [
        ("Ativo (Em Execução)", active_exec),
        ("Ativo (Em Andamento)", active_prog),
        ("Inativo (Concluído)", concluded),
        ("Inativo (Cancelado)", cancelled),
        ("Inativo (Não Informado)", unknown),
    ] {
        if count > 0 {
            items.push(ChartItem {
                label: label.to_string(),
                count,
                secondary_value: None,
                formatted_value: Some(format!(
                    "{} projetos ({:.1}%)",
                    format_pt_int(count),
                    pct(count, total)
                )),
                percentage: pct(count, total),
            });
        }
    }

    (active_exec + active_prog, concluded, cancelled, total, items)
}

fn build_fellowship_charts(
    fellowship_records: &[Record],
    adv_fin: &AdvisorshipFinancialSummary,
) -> Vec<ChartSection> {
    let mut sections = Vec::new();

    if !adv_fin.program_counts.is_empty() {
        let mut prog_vec: Vec<(String, usize, f64)> = adv_fin
            .program_counts
            .iter()
            .map(|(k, (c, v))| (k.clone(), *c, *v))
            .collect();
        prog_vec.sort_by(|a, b| {
            b.2.partial_cmp(&a.2)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.1.cmp(&a.1))
        });
        let total_prog_count: usize = prog_vec.iter().map(|(_, c, _)| *c).sum();
        let items = prog_vec
            .into_iter()
            .map(|(label, count, val)| ChartItem {
                label,
                count,
                secondary_value: Some(val),
                formatted_value: Some(format!(
                    "{} bolsas • {}/mês",
                    format_pt_int(count),
                    format_brl(val)
                )),
                percentage: pct(count, total_prog_count),
            })
            .collect();
        sections.push(ChartSection {
            id: "fellowship_programs".to_string(),
            title: "Bolsas e Investimento Mensal por Programa de Fomento".to_string(),
            subtitle: Some(
                "Quantidade de orientações vinculadas e volume financeiro mensal por programa"
                    .to_string(),
            ),
            kind: "financial_table".to_string(),
            items,
            is_custom: false,
            custom_id: None,
        });
    }

    if !adv_fin.supervisors.is_empty() {
        let mut sup_vec: Vec<(String, usize, usize, f64)> = adv_fin
            .supervisors
            .iter()
            .map(|(name, (tot, paid, brl))| (name.clone(), *tot, *paid, *brl))
            .collect();
        sup_vec.sort_by(|a, b| {
            b.3.partial_cmp(&a.3)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.2.cmp(&a.2))
                .then_with(|| b.1.cmp(&a.1))
        });
        let max_brl = sup_vec.first().map(|x| x.3).unwrap_or(1.0).max(1.0);
        let items = sup_vec
            .into_iter()
            .take(12)
            .map(|(name, tot, paid, brl)| ChartItem {
                label: name,
                count: paid,
                secondary_value: Some(brl),
                formatted_value: Some(format!(
                    "{}/mês ({} bolsas remuneradas / {} orientações)",
                    format_brl(brl),
                    paid,
                    tot
                )),
                percentage: ((brl / max_brl) * 1000.0).round() / 10.0,
            })
            .collect();
        sections.push(ChartSection {
            id: "top_supervisors".to_string(),
            title: "Quanto Supervisionam os Pesquisadores Orientadores (Top Bolsas)".to_string(),
            subtitle: Some(
                "Ranking de pesquisadores pelo montante mensal de bolsas e número de bolsistas sob sua orientação"
                    .to_string(),
            ),
            kind: "financial_table".to_string(),
            items,
            is_custom: false,
            custom_id: None,
        });
    }

    if !adv_fin.recipients.is_empty() {
        let mut rec_vec: Vec<(String, usize, f64, String)> = adv_fin
            .recipients
            .iter()
            .map(|(name, (cnt, brl, prog))| (name.clone(), *cnt, *brl, prog.clone()))
            .collect();
        rec_vec.sort_by(|a, b| {
            b.2.partial_cmp(&a.2)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.1.cmp(&a.1))
        });
        let max_brl = rec_vec.first().map(|x| x.2).unwrap_or(1.0).max(1.0);
        let items = rec_vec
            .into_iter()
            .take(12)
            .map(|(name, cnt, brl, prog)| ChartItem {
                label: format!("{name} ({prog})"),
                count: cnt,
                secondary_value: Some(brl),
                formatted_value: Some(format!("{}/mês ({} cota(s))", format_brl(brl), cnt)),
                percentage: ((brl / max_brl) * 1000.0).round() / 10.0,
            })
            .collect();
        sections.push(ChartSection {
            id: "top_recipients".to_string(),
            title: "Quanto Recebem os Bolsistas (Top Bolsistas Remunerados)".to_string(),
            subtitle: Some(
                "Valor mensal acumulado e programa de fomento dos bolsistas vinculados".to_string(),
            ),
            kind: "financial_table".to_string(),
            items,
            is_custom: false,
            custom_id: None,
        });
    }

    let mut catalog_map: BTreeMap<String, f64> = BTreeMap::new();
    for rec in fellowship_records {
        let name = rec
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .trim();
        let val = extract_numeric_or_link_value(rec.get("value"));
        let key = if val > 0.0 {
            format!("{} ({})", name, format_brl(val))
        } else {
            format!("{} (Voluntário / R$ 0,00)", name)
        };
        let entry = catalog_map.entry(key).or_insert(val);
        if val > *entry {
            *entry = val;
        }
    }
    if !catalog_map.is_empty() {
        let mut cat_vec: Vec<(String, f64)> = catalog_map.into_iter().collect();
        cat_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let max_val = cat_vec.first().map(|x| x.1).unwrap_or(1.0).max(1.0);
        let items = cat_vec
            .into_iter()
            .map(|(label, val)| ChartItem {
                label,
                count: val.round() as usize,
                secondary_value: Some(val),
                formatted_value: Some(format!("{}/mês", format_brl(val))),
                percentage: if max_val > 0.0 {
                    ((val / max_val) * 1000.0).round() / 10.0
                } else {
                    0.0
                },
            })
            .collect();
        sections.push(ChartSection {
            id: "fellowship_catalog".to_string(),
            title: "Valores Unitários das Modalidades de Bolsa (Tabela de Bolsas)".to_string(),
            subtitle: Some("Valor unitário mensal cadastrado no catálogo de bolsas".to_string()),
            kind: "bar".to_string(),
            items,
            is_custom: false,
            custom_id: None,
        });
    }

    sections
}

fn scalar_distribution_from_records(
    records: &[Record],
    col: &str,
    limit: usize,
    unit: &str,
) -> Vec<ChartItem> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for rec in records {
        for lb in extract_group_labels(col, rec.get(col)) {
            *counts.entry(lb).or_insert(0) += 1;
        }
    }
    top_items_from_counts(counts, records.len(), limit, unit)
}

/// Wrapper sem filtro para compatibilidade com chamadas legadas.
pub fn horizon_analytics(
    conn: &Connection,
    v2: bool,
    entity: Option<&str>,
) -> Result<DashboardPayload, AppError> {
    horizon_analytics_filtered(conn, v2, entity, None)
}

/// Gera o Dashboard Geral ou os Gráficos por Tabela para Horizon (`v2 = true`) ou Horizon Legacy (`v2 = false`) com suporte a `filters`.
pub fn horizon_analytics_filtered(
    conn: &Connection,
    v2: bool,
    entity: Option<&str>,
    filters: Option<&str>,
) -> Result<DashboardPayload, AppError> {
    let scope = if v2 { "horizon-v2" } else { "horizon" }.to_string();
    let spec = parse_filter_spec(filters);
    let entities = if v2 {
        registry::HORIZON_V2_ENTITIES
    } else {
        registry::ENTITIES
    };

    let clean_entity = entity.map(str::trim).filter(|s| !s.is_empty());
    if let Some(route) = clean_entity {
        return horizon_entity_analytics_filtered(conn, v2, route, &spec);
    }

    let mut table_counts_items = Vec::new();
    let mut total_rows = 0usize;
    let mut count_by_route: HashMap<&str, usize> = HashMap::new();
    let mut records_by_route: HashMap<&str, Vec<Record>> = HashMap::new();

    for def in entities.iter().filter(|e| e.exported) {
        // Se um filtro global foi aplicado, filtramos apenas as tabelas que possuem as colunas referenciadas
        let applicable_rules: Vec<ColumnFilter> = spec
            .rules
            .iter()
            .filter(|r| def.has_column(r.field.trim()))
            .cloned()
            .collect();
        let table_spec = FilterSpec {
            logic: spec.logic.clone(),
            rules: applicable_rules,
        };
        let recs = load_table_records_filtered(conn, def, &table_spec)?;
        let c = recs.len();
        total_rows += c;
        count_by_route.insert(def.route, c);
        table_counts_items.push((
            format!("{} ({})", entity_pt_label(def.route), def.canonical_table()),
            c,
        ));
        records_by_route.insert(def.route, recs);
    }

    table_counts_items.sort_by(|a, b| b.1.cmp(&a.1));
    let max_table_count = table_counts_items.first().map(|x| x.1).unwrap_or(1).max(1);
    let table_chart_items: Vec<ChartItem> = table_counts_items
        .into_iter()
        .map(|(label, count)| ChartItem {
            label,
            count,
            secondary_value: None,
            formatted_value: Some(format!("{} registros", format_pt_int(count))),
            percentage: pct(count, max_table_count),
        })
        .collect();

    let empty_vec = Vec::new();
    let init_recs = records_by_route.get("initiatives").unwrap_or(&empty_vec);
    let adv_recs = records_by_route.get("advisorships").unwrap_or(&empty_vec);
    let fell_recs = records_by_route.get("fellowships").unwrap_or(&empty_vec);
    let res_recs = records_by_route.get("researchers").unwrap_or(&empty_vec);
    let pa_recs = records_by_route
        .get("professional_activities")
        .unwrap_or(&empty_vec);
    let art_recs = records_by_route.get("articles").unwrap_or(&empty_vec);
    let prod_recs = records_by_route
        .get("research_productions")
        .unwrap_or(&empty_vec);

    let (active_init, concluded_init, cancelled_init, total_init, init_status_items) =
        compute_status_breakdown_from_records(init_recs);
    let (active_adv, _concluded_adv, _cancelled_adv, total_adv_proj, adv_status_items) =
        compute_status_breakdown_from_records(adv_recs);

    let adv_fin = compute_advisorship_financials_from_records(adv_recs);
    let (edu_insts, highest_degrees) = compute_education_and_degrees_from_records(res_recs);

    let mut prof_insts: HashMap<String, usize> = HashMap::new();
    for rec in pa_recs {
        if let Some(name) = rec
            .get("institution_name")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            *prof_insts.entry(name.to_string()).or_insert(0) += 1;
        }
    }

    let mut class_counts: HashMap<String, usize> = HashMap::new();
    for rec in res_recs {
        let cl = rec
            .get("classification")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("Não Classificado");
        let label = match cl {
            "researcher" => "Pesquisador (IFES)".to_string(),
            "student" => "Estudante".to_string(),
            "outside_ifes" => "Externo ao IFES".to_string(),
            other => other.to_string(),
        };
        *class_counts.entry(label).or_insert(0) += 1;
    }

    let mut prod_by_year: BTreeMap<String, usize> = BTreeMap::new();
    for rec in art_recs.iter().chain(prod_recs.iter()) {
        if let Some(y) = rec.get("year").and_then(|v| v.as_str()) {
            let year_clean = y.split('.').next().unwrap_or(y).trim().to_string();
            if year_clean.len() == 4 && year_clean.chars().all(|c| c.is_ascii_digit()) {
                *prod_by_year.entry(year_clean).or_insert(0) += 1;
            }
        }
    }

    let (init_campuses, _) = extract_named_entities_from_records(init_recs, "campus");
    let (init_areas, _) = extract_named_entities_from_records(init_recs, "knowledge_areas");

    let researchers_count = *count_by_route.get("researchers").unwrap_or(&0);
    let students_count = *count_by_route.get("students").unwrap_or(&0);
    let groups_count = *count_by_route.get("groups").unwrap_or(&0);
    let articles_count = *count_by_route.get("articles").unwrap_or(&0);
    let productions_count = *count_by_route.get("research_productions").unwrap_or(&0);

    let kpis = vec![
        KpiCard {
            label: "Projetos Ativos".to_string(),
            value: format_pt_int(active_init),
            sublabel: Some(format!(
                "{:.1}% de {} iniciativas ({} orientações ativas)",
                pct(active_init, total_init),
                format_pt_int(total_init),
                format_pt_int(active_adv)
            )),
            tone: "emerald".to_string(),
        },
        KpiCard {
            label: "Projetos Inativos / Concluídos".to_string(),
            value: format_pt_int(concluded_init + cancelled_init),
            sublabel: Some(format!(
                "{} concluídos • {} cancelados",
                format_pt_int(concluded_init),
                format_pt_int(cancelled_init)
            )),
            tone: "slate".to_string(),
        },
        KpiCard {
            label: "Registros nas 15 Tabelas".to_string(),
            value: format_pt_int(total_rows),
            sublabel: Some("Total consolidado no banco local".to_string()),
            tone: "indigo".to_string(),
        },
        KpiCard {
            label: "Pesquisadores & Alunos".to_string(),
            value: format_pt_int(researchers_count),
            sublabel: Some(format!(
                "{} na tabela de alunos • {} grupos de pesquisa",
                format_pt_int(students_count),
                format_pt_int(groups_count)
            )),
            tone: "sky".to_string(),
        },
        KpiCard {
            label: "Investimento Mensal em Bolsas".to_string(),
            value: format_brl(adv_fin.total_monthly_investment),
            sublabel: Some(format!(
                "{} bolsas remuneradas • {} voluntárias",
                format_pt_int(adv_fin.paid_count),
                format_pt_int(adv_fin.volunteer_count)
            )),
            tone: "amber".to_string(),
        },
        KpiCard {
            label: "Produção Científica".to_string(),
            value: format_pt_int(articles_count + productions_count),
            sublabel: Some(format!(
                "{} artigos • {} produções técnicas/livros",
                format_pt_int(articles_count),
                format_pt_int(productions_count)
            )),
            tone: "indigo".to_string(),
        },
    ];

    let mut charts = Vec::new();

    if !init_status_items.is_empty() {
        charts.push(ChartSection {
            id: "project_status".to_string(),
            title: "Status dos Projetos (Iniciativas: Ativos vs. Inativos)".to_string(),
            subtitle: Some(format!(
                "Total de {} iniciativas: {} ativas ({:.1}%) vs. {} inativas/concluídas",
                format_pt_int(total_init),
                format_pt_int(active_init),
                pct(active_init, total_init),
                format_pt_int(total_init.saturating_sub(active_init))
            )),
            kind: "donut".to_string(),
            items: init_status_items,
            is_custom: false,
            custom_id: None,
        });
    }

    if !adv_status_items.is_empty() {
        charts.push(ChartSection {
            id: "advisorship_status".to_string(),
            title: "Status dos Projetos de Orientação".to_string(),
            subtitle: Some(format!(
                "Total de {} projetos de orientação ({} orientações individuais vinculadas)",
                format_pt_int(total_adv_proj),
                format_pt_int(adv_fin.total_advisorships)
            )),
            kind: "donut".to_string(),
            items: adv_status_items,
            is_custom: false,
            custom_id: None,
        });
    }

    charts.push(ChartSection {
        id: "table_counts".to_string(),
        title: "Quantidade de Registros por Tabela Canônica (15 Tabelas)".to_string(),
        subtitle: Some("Volume de registros carregados em cada entidade do domínio".to_string()),
        kind: "bar".to_string(),
        items: table_chart_items,
        is_custom: false,
        custom_id: None,
    });

    charts.extend(build_fellowship_charts(fell_recs, &adv_fin));

    if !prof_insts.is_empty() {
        charts.push(ChartSection {
            id: "professional_institutions".to_string(),
            title: "De Quais Instituições Vêm / Atuam os Pesquisadores (Atividades Profissionais)"
                .to_string(),
            subtitle: Some("Principais instituições de vínculo profissional registradas".to_string()),
            kind: "bar".to_string(),
            items: top_items_from_counts(prof_insts, 0, 12, "vínculos"),
            is_custom: false,
            custom_id: None,
        });
    }

    if !edu_insts.is_empty() {
        charts.push(ChartSection {
            id: "education_institutions".to_string(),
            title: "Instituições de Formação Acadêmica dos Pesquisadores".to_string(),
            subtitle: Some(
                "Universidades e instituições onde os pesquisadores obtiveram suas titulações"
                    .to_string(),
            ),
            kind: "bar".to_string(),
            items: top_items_from_counts(edu_insts, 0, 12, "pesquisadores"),
            is_custom: false,
            custom_id: None,
        });
    }

    if !highest_degrees.is_empty() {
        charts.push(ChartSection {
            id: "researcher_degrees".to_string(),
            title: "Distribuição por Maior Titulação Acadêmica".to_string(),
            subtitle: Some(
                "Maior grau acadêmico identificado no currículo dos pesquisadores".to_string(),
            ),
            kind: "donut".to_string(),
            items: top_items_from_counts(highest_degrees, 0, 8, "pesquisadores"),
            is_custom: false,
            custom_id: None,
        });
    }

    if !class_counts.is_empty() {
        charts.push(ChartSection {
            id: "person_classifications".to_string(),
            title: "Classificação Canônica na Base de Pesquisadores".to_string(),
            subtitle: Some("Divisão por perfil identificado pelo pipeline Horizon".to_string()),
            kind: "donut".to_string(),
            items: top_items_from_counts(class_counts, researchers_count, 6, "pessoas"),
            is_custom: false,
            custom_id: None,
        });
    }

    if !prod_by_year.is_empty() {
        let recent: Vec<(String, usize)> = prod_by_year
            .into_iter()
            .rev()
            .take(12)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        let max_y = recent.iter().map(|(_, c)| *c).max().unwrap_or(1).max(1);
        charts.push(ChartSection {
            id: "production_by_year".to_string(),
            title: "Evolução Anual da Produção Científica (Artigos + Produções)".to_string(),
            subtitle: Some(
                "Volume de publicações por ano nos últimos períodos registrados".to_string(),
            ),
            kind: "columns".to_string(),
            items: recent
                .into_iter()
                .map(|(year, count)| ChartItem {
                    label: year,
                    count,
                    secondary_value: None,
                    formatted_value: Some(format!("{} publicações", format_pt_int(count))),
                    percentage: pct(count, max_y),
                })
                .collect(),
            is_custom: false,
            custom_id: None,
        });
    }

    if !init_campuses.is_empty() {
        charts.push(ChartSection {
            id: "initiatives_by_campus".to_string(),
            title: "Distribuição de Projetos / Iniciativas por Campus".to_string(),
            subtitle: Some("Participação dos campi nos projetos cadastrados".to_string()),
            kind: "bar".to_string(),
            items: top_items_from_counts(init_campuses, 0, 12, "projetos"),
            is_custom: false,
            custom_id: None,
        });
    }

    if !init_areas.is_empty() {
        charts.push(ChartSection {
            id: "top_knowledge_areas".to_string(),
            title: "Principais Áreas do Conhecimento nos Projetos".to_string(),
            subtitle: Some("Áreas de conhecimento com maior número de iniciativas associadas".to_string()),
            kind: "bar".to_string(),
            items: top_items_from_counts(init_areas, 0, 12, "projetos"),
            is_custom: false,
            custom_id: None,
        });
    }

    append_custom_charts(conn, &scope, None, &spec, &mut charts)?;

    let db_has_any_rows = total_rows > 0 || !spec.is_empty();

    Ok(DashboardPayload {
        available_entities: available_entities_for_scope(&scope),
        scope,
        entity: None,
        title: if v2 {
            "Dashboard Analítico — Horizon".to_string()
        } else {
            "Dashboard Analítico — Horizon Legacy".to_string()
        },
        loaded: db_has_any_rows && (total_rows > 0 || !charts.is_empty()),
        total_rows,
        kpis,
        charts,
    })
}

fn horizon_entity_analytics_filtered(
    conn: &Connection,
    v2: bool,
    route: &str,
    spec: &FilterSpec,
) -> Result<DashboardPayload, AppError> {
    let def = if v2 {
        registry::by_horizon_v2_route(route)?
    } else {
        registry::by_route(route)?
    };
    let scope = if v2 { "horizon-v2" } else { "horizon" }.to_string();
    let raw_table_count = table_row_count(conn, def.table)?;
    let records = load_table_records_filtered(conn, def, spec)?;
    let total_rows = records.len();
    let pt_label = entity_pt_label(route);

    let mut kpis = vec![KpiCard {
        label: format!("Total em {pt_label}"),
        value: format_pt_int(total_rows),
        sublabel: Some(if spec.is_empty() {
            format!("Tabela canônica: {}", def.canonical_table())
        } else {
            format!("Filtrado de {} registros totais", format_pt_int(raw_table_count))
        }),
        tone: "indigo".to_string(),
    }];
    let mut charts = Vec::new();

    match route {
        "initiatives" => {
            let (active, concluded, cancelled, total, status_items) =
                compute_status_breakdown_from_records(&records);
            let enriched = records
                .iter()
                .filter(|r| {
                    r.get("enrichment")
                        .and_then(|v| v.as_str())
                        .map(|s| !s.trim().is_empty() && s.trim() != "null")
                        .unwrap_or(false)
                })
                .count();
            kpis.push(KpiCard {
                label: "Projetos Ativos".to_string(),
                value: format_pt_int(active),
                sublabel: Some(format!("{:.1}% do recorte atual", pct(active, total))),
                tone: "emerald".to_string(),
            });
            kpis.push(KpiCard {
                label: "Inativos / Concluídos".to_string(),
                value: format_pt_int(concluded + cancelled),
                sublabel: Some(format!(
                    "{} concluídos • {} cancelados",
                    format_pt_int(concluded),
                    format_pt_int(cancelled)
                )),
                tone: "slate".to_string(),
            });
            kpis.push(KpiCard {
                label: "Enriquecidos SIGPesq".to_string(),
                value: format_pt_int(enriched),
                sublabel: Some("Projetos com metadados SIGPesq vinculados".to_string()),
                tone: "sky".to_string(),
            });
            if !status_items.is_empty() {
                charts.push(ChartSection {
                    id: "initiatives_status".to_string(),
                    title: "Status das Iniciativas (Ativos vs. Inativos)".to_string(),
                    subtitle: Some("Distribuição das iniciativas por situação atual".to_string()),
                    kind: "donut".to_string(),
                    items: status_items,
                    is_custom: false,
                    custom_id: None,
                });
            }
            let (types_map, _) = extract_named_entities_from_records(&records, "initiative_type");
            if !types_map.is_empty() {
                charts.push(ChartSection {
                    id: "initiative_types".to_string(),
                    title: "Iniciativas por Tipo de Projeto".to_string(),
                    subtitle: None,
                    kind: "donut".to_string(),
                    items: top_items_from_counts(types_map, total_rows, 8, "projetos"),
                    is_custom: false,
                    custom_id: None,
                });
            }
            let (campuses_map, _) = extract_named_entities_from_records(&records, "campus");
            if !campuses_map.is_empty() {
                charts.push(ChartSection {
                    id: "initiatives_campus".to_string(),
                    title: "Iniciativas por Campus".to_string(),
                    subtitle: None,
                    kind: "bar".to_string(),
                    items: top_items_from_counts(campuses_map, 0, 12, "projetos"),
                    is_custom: false,
                    custom_id: None,
                });
            }
            let (areas_map, _) = extract_named_entities_from_records(&records, "knowledge_areas");
            if !areas_map.is_empty() {
                charts.push(ChartSection {
                    id: "initiatives_areas".to_string(),
                    title: "Principais Áreas de Conhecimento das Iniciativas".to_string(),
                    subtitle: None,
                    kind: "bar".to_string(),
                    items: top_items_from_counts(areas_map, 0, 12, "projetos"),
                    is_custom: false,
                    custom_id: None,
                });
            }
        }
        "advisorships" | "fellowships" => {
            let adv_def = if v2 {
                registry::by_horizon_v2_route("advisorships")?
            } else {
                registry::by_route("advisorships")?
            };
            let fell_def = if v2 {
                registry::by_horizon_v2_route("fellowships")?
            } else {
                registry::by_route("fellowships")?
            };
            let adv_records = if route == "advisorships" {
                records.clone()
            } else {
                load_table_records_filtered(conn, adv_def, &FilterSpec::default())?
            };
            let fell_records = if route == "fellowships" {
                records.clone()
            } else {
                load_table_records_filtered(conn, fell_def, &FilterSpec::default())?
            };
            let (active, concluded, _cancelled, total_proj, status_items) =
                compute_status_breakdown_from_records(&adv_records);
            let adv_fin = compute_advisorship_financials_from_records(&adv_records);
            kpis.push(KpiCard {
                label: "Projetos de Orientação Ativos".to_string(),
                value: format_pt_int(active),
                sublabel: Some(format!(
                    "{:.1}% de {} projetos ({} concluídos)",
                    pct(active, total_proj),
                    format_pt_int(total_proj),
                    format_pt_int(concluded)
                )),
                tone: "emerald".to_string(),
            });
            kpis.push(KpiCard {
                label: "Investimento Mensal em Bolsas".to_string(),
                value: format_brl(adv_fin.total_monthly_investment),
                sublabel: Some(format!(
                    "{} bolsas remuneradas • {} voluntárias",
                    format_pt_int(adv_fin.paid_count),
                    format_pt_int(adv_fin.volunteer_count)
                )),
                tone: "amber".to_string(),
            });
            if !status_items.is_empty() {
                charts.push(ChartSection {
                    id: "advisorships_status".to_string(),
                    title: "Status dos Projetos de Orientação (Ativos vs. Inativos)".to_string(),
                    subtitle: None,
                    kind: "donut".to_string(),
                    items: status_items,
                    is_custom: false,
                    custom_id: None,
                });
            }
            charts.extend(build_fellowship_charts(&fell_records, &adv_fin));
        }
        "researchers" | "students" => {
            let (edu_insts, degrees) = compute_education_and_degrees_from_records(&records);
            let with_articles = records
                .iter()
                .filter(|r| extract_numeric_or_link_value(r.get("articles")) > 0.0)
                .count();
            let with_initiatives = records
                .iter()
                .filter(|r| extract_numeric_or_link_value(r.get("initiatives")) > 0.0)
                .count();
            kpis.push(KpiCard {
                label: "Com Projetos / Iniciativas".to_string(),
                value: format_pt_int(with_initiatives),
                sublabel: Some(format!("{:.1}% do recorte", pct(with_initiatives, total_rows))),
                tone: "emerald".to_string(),
            });
            kpis.push(KpiCard {
                label: "Com Artigos Publicados".to_string(),
                value: format_pt_int(with_articles),
                sublabel: Some(format!("{:.1}% do recorte", pct(with_articles, total_rows))),
                tone: "sky".to_string(),
            });
            let class_items =
                scalar_distribution_from_records(&records, "classification", 8, "registros");
            if !class_items.is_empty() {
                charts.push(ChartSection {
                    id: "person_classification".to_string(),
                    title: format!("Classificação em {pt_label}"),
                    subtitle: None,
                    kind: "donut".to_string(),
                    items: class_items,
                    is_custom: false,
                    custom_id: None,
                });
            }
            if !degrees.is_empty() {
                charts.push(ChartSection {
                    id: "person_degrees".to_string(),
                    title: format!("Maior Titulação Acadêmica em {pt_label}"),
                    subtitle: None,
                    kind: "donut".to_string(),
                    items: top_items_from_counts(degrees, 0, 8, "pessoas"),
                    is_custom: false,
                    custom_id: None,
                });
            }
            if !edu_insts.is_empty() {
                charts.push(ChartSection {
                    id: "person_edu_institutions".to_string(),
                    title: format!("Instituições de Formação Acadêmica ({pt_label})"),
                    subtitle: None,
                    kind: "bar".to_string(),
                    items: top_items_from_counts(edu_insts, 0, 12, "pessoas"),
                    is_custom: false,
                    custom_id: None,
                });
            }
            let (campuses_map, _) = extract_named_entities_from_records(&records, "campus");
            if !campuses_map.is_empty() {
                charts.push(ChartSection {
                    id: "person_campus".to_string(),
                    title: format!("Distribuição por Campus ({pt_label})"),
                    subtitle: None,
                    kind: "bar".to_string(),
                    items: top_items_from_counts(campuses_map, 0, 12, "pessoas"),
                    is_custom: false,
                    custom_id: None,
                });
            }
        }
        "professional_activities" => {
            let current_count = records
                .iter()
                .filter(|r| {
                    r.get("current")
                        .and_then(|v| v.as_str())
                        .map(|s| matches!(s.trim().to_ascii_lowercase().as_str(), "true" | "1"))
                        .unwrap_or(false)
                })
                .count();
            kpis.push(KpiCard {
                label: "Vínculos Atuais (Ativos)".to_string(),
                value: format_pt_int(current_count),
                sublabel: Some(format!(
                    "{:.1}% das atividades profissionais",
                    pct(current_count, total_rows)
                )),
                tone: "emerald".to_string(),
            });
            let inst_items =
                scalar_distribution_from_records(&records, "institution_name", 12, "vínculos");
            if !inst_items.is_empty() {
                charts.push(ChartSection {
                    id: "pa_institutions".to_string(),
                    title: "Principais Instituições de Vínculo Profissional".to_string(),
                    subtitle: None,
                    kind: "bar".to_string(),
                    items: inst_items,
                    is_custom: false,
                    custom_id: None,
                });
            }
            let bond_items = scalar_distribution_from_records(&records, "bond", 10, "vínculos");
            if !bond_items.is_empty() {
                charts.push(ChartSection {
                    id: "pa_bonds".to_string(),
                    title: "Distribuição por Tipo de Vínculo / Regime".to_string(),
                    subtitle: None,
                    kind: "donut".to_string(),
                    items: bond_items,
                    is_custom: false,
                    custom_id: None,
                });
            }
        }
        "articles" | "research_productions" | "awards" => {
            let year_items = scalar_distribution_from_records(&records, "year", 12, "registros");
            if !year_items.is_empty() {
                charts.push(ChartSection {
                    id: "by_year".to_string(),
                    title: format!("{pt_label} por Ano"),
                    subtitle: None,
                    kind: "columns".to_string(),
                    items: year_items,
                    is_custom: false,
                    custom_id: None,
                });
            }
            if def.has_column("type") {
                let type_items = scalar_distribution_from_records(&records, "type", 10, "artigos");
                if !type_items.is_empty() {
                    charts.push(ChartSection {
                        id: "article_types".to_string(),
                        title: "Artigos por Tipo de Publicação".to_string(),
                        subtitle: None,
                        kind: "donut".to_string(),
                        items: type_items,
                        is_custom: false,
                        custom_id: None,
                    });
                }
                let venue_items =
                    scalar_distribution_from_records(&records, "journal_conference", 10, "artigos");
                if !venue_items.is_empty() {
                    charts.push(ChartSection {
                        id: "article_venues".to_string(),
                        title: "Principais Periódicos e Conferências".to_string(),
                        subtitle: None,
                        kind: "bar".to_string(),
                        items: venue_items,
                        is_custom: false,
                        custom_id: None,
                    });
                }
            }
            if def.has_column("publisher") {
                let pub_items =
                    scalar_distribution_from_records(&records, "publisher", 10, "produções");
                if !pub_items.is_empty() {
                    charts.push(ChartSection {
                        id: "prod_publishers".to_string(),
                        title: "Principais Editoras / Plataformas".to_string(),
                        subtitle: None,
                        kind: "bar".to_string(),
                        items: pub_items,
                        is_custom: false,
                        custom_id: None,
                    });
                }
            }
        }
        "groups" => {
            let (campuses_map, _) = extract_named_entities_from_records(&records, "campus");
            if !campuses_map.is_empty() {
                charts.push(ChartSection {
                    id: "groups_campus".to_string(),
                    title: "Grupos de Pesquisa por Campus".to_string(),
                    subtitle: None,
                    kind: "bar".to_string(),
                    items: top_items_from_counts(campuses_map, 0, 12, "grupos"),
                    is_custom: false,
                    custom_id: None,
                });
            }
            let (areas_map, _) = extract_named_entities_from_records(&records, "knowledge_areas");
            if !areas_map.is_empty() {
                charts.push(ChartSection {
                    id: "groups_areas".to_string(),
                    title: "Áreas de Conhecimento dos Grupos de Pesquisa".to_string(),
                    subtitle: None,
                    kind: "bar".to_string(),
                    items: top_items_from_counts(areas_map, 0, 12, "grupos"),
                    is_custom: false,
                    custom_id: None,
                });
            }
        }
        "proficiencies" => {
            let comp_items =
                scalar_distribution_from_records(&records, "comprehension", 8, "registros");
            if !comp_items.is_empty() {
                charts.push(ChartSection {
                    id: "prof_comprehension".to_string(),
                    title: "Nível de Compreensão de Idiomas".to_string(),
                    subtitle: None,
                    kind: "donut".to_string(),
                    items: comp_items,
                    is_custom: false,
                    custom_id: None,
                });
            }
            let read_items = scalar_distribution_from_records(&records, "reading", 8, "registros");
            if !read_items.is_empty() {
                charts.push(ChartSection {
                    id: "prof_reading".to_string(),
                    title: "Nível de Leitura de Idiomas".to_string(),
                    subtitle: None,
                    kind: "bar".to_string(),
                    items: read_items,
                    is_custom: false,
                    custom_id: None,
                });
            }
        }
        _ => {
            if def.has_column("name") {
                let name_items =
                    scalar_distribution_from_records(&records, "name", 12, "registros");
                if !name_items.is_empty() {
                    charts.push(ChartSection {
                        id: "entity_names".to_string(),
                        title: format!("Distribuição de Registros em {pt_label}"),
                        subtitle: None,
                        kind: "bar".to_string(),
                        items: name_items,
                        is_custom: false,
                        custom_id: None,
                    });
                }
            }
            if def.has_column("campus") {
                let (campuses_map, _) = extract_named_entities_from_records(&records, "campus");
                if !campuses_map.is_empty() {
                    charts.push(ChartSection {
                        id: "entity_campus".to_string(),
                        title: format!("{pt_label} por Campus"),
                        subtitle: None,
                        kind: "bar".to_string(),
                        items: top_items_from_counts(campuses_map, 0, 12, "registros"),
                        is_custom: false,
                        custom_id: None,
                    });
                }
            }
        }
    }

    append_custom_charts(conn, &scope, Some(route), spec, &mut charts)?;

    Ok(DashboardPayload {
        available_entities: available_entities_for_scope(&scope),
        scope,
        entity: Some(route.to_string()),
        title: format!("Gráficos da Tabela — {pt_label}"),
        loaded: raw_table_count > 0 || !charts.is_empty(),
        total_rows,
        kpis,
        charts,
    })
}

/// Wrapper sem filtro para SRC.
pub fn src_analytics(conn: &Connection) -> Result<DashboardPayload, AppError> {
    src_analytics_filtered(conn, None)
}

/// Gera o Dashboard Analítico para a aba SRC (`src_acoes` e `src_participacoes`) com suporte a `filters`.
pub fn src_analytics_filtered(
    conn: &Connection,
    filters: Option<&str>,
) -> Result<DashboardPayload, AppError> {
    let spec = parse_filter_spec(filters);
    let raw_acoes = table_row_count(conn, "src_acoes")?;
    let total_part = table_row_count(conn, "src_participacoes")?;
    let def = registry::by_src_route("src_acoes")?;
    let acoes_records = load_table_records_filtered(conn, def, &spec)?;
    let total_acoes = acoes_records.len();

    let acoes_com_part = acoes_records
        .iter()
        .filter(|r| extract_numeric_or_link_value(r.get("total_participacoes")) > 0.0)
        .count();
    let acoes_vinculadas = acoes_records
        .iter()
        .filter(|r| {
            r.get("acao_vinculante")
                .and_then(|v| v.as_str())
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false)
        })
        .count();

    let kpis = vec![
        KpiCard {
            label: "Total de Ações SRC".to_string(),
            value: format_pt_int(total_acoes),
            sublabel: Some(format!("{} com ação vinculante", format_pt_int(acoes_vinculadas))),
            tone: "indigo".to_string(),
        },
        KpiCard {
            label: "Ações Ativas c/ Participações".to_string(),
            value: format_pt_int(acoes_com_part),
            sublabel: Some(format!(
                "{:.1}% das ações possuem equipe/participantes",
                pct(acoes_com_part, total_acoes)
            )),
            tone: "emerald".to_string(),
        },
        KpiCard {
            label: "Total de Participações".to_string(),
            value: format_pt_int(total_part),
            sublabel: Some("Membros e público-alvo vinculados".to_string()),
            tone: "sky".to_string(),
        },
    ];

    let mut charts = Vec::new();
    let nat_items = scalar_distribution_from_records(&acoes_records, "natureza", 10, "ações");
    if !nat_items.is_empty() {
        charts.push(ChartSection {
            id: "src_natureza".to_string(),
            title: "Ações por Natureza".to_string(),
            subtitle: Some("Distribuição entre extensão, ensino e pesquisa".to_string()),
            kind: "donut".to_string(),
            items: nat_items,
            is_custom: false,
            custom_id: None,
        });
    }
    let tipo_items = scalar_distribution_from_records(&acoes_records, "tipo", 10, "ações");
    if !tipo_items.is_empty() {
        charts.push(ChartSection {
            id: "src_tipo".to_string(),
            title: "Ações por Tipo".to_string(),
            subtitle: None,
            kind: "bar".to_string(),
            items: tipo_items,
            is_custom: false,
            custom_id: None,
        });
    }
    let coord_items = scalar_distribution_from_records(&acoes_records, "coordenador", 10, "ações");
    if !coord_items.is_empty() {
        charts.push(ChartSection {
            id: "src_coordenadores".to_string(),
            title: "Principais Coordenadores de Ações".to_string(),
            subtitle: None,
            kind: "bar".to_string(),
            items: coord_items,
            is_custom: false,
            custom_id: None,
        });
    }
    let campus_items = scalar_distribution_from_records(&acoes_records, "campus", 10, "ações");
    if !campus_items.is_empty() {
        charts.push(ChartSection {
            id: "src_campus".to_string(),
            title: "Ações por Campus".to_string(),
            subtitle: None,
            kind: "bar".to_string(),
            items: campus_items,
            is_custom: false,
            custom_id: None,
        });
    }

    append_custom_charts(conn, "src", None, &spec, &mut charts)?;

    Ok(DashboardPayload {
        available_entities: available_entities_for_scope("src"),
        scope: "src".to_string(),
        entity: Some("src_acoes".to_string()),
        title: "Dashboard Analítico — SRC (Ações de Extensão e Ensino)".to_string(),
        loaded: (raw_acoes + total_part) > 0 || !charts.is_empty(),
        total_rows: total_acoes,
        kpis,
        charts,
    })
}

fn aggregate_records_by_fields(
    records: &[Record],
    fields: &[(&str, &str, &str)],
) -> Vec<ChartSection> {
    let mut maps: Vec<HashMap<String, usize>> = vec![HashMap::new(); fields.len()];
    let total = records.len();

    for obj in records {
        for (idx, (field_name, _, _)) in fields.iter().enumerate() {
            if let Some(val) = obj.get(*field_name) {
                let s = match val {
                    Json::String(st) => st.trim().to_string(),
                    Json::Bool(b) => {
                        if *b {
                            "Sim (Ativo)".to_string()
                        } else {
                            "Não (Inativo)".to_string()
                        }
                    }
                    Json::Number(n) => n.to_string(),
                    _ => String::new(),
                };
                if !s.is_empty() && s != "—" && s != "-" {
                    *maps[idx].entry(s).or_insert(0) += 1;
                }
            }
        }
    }

    let mut sections = Vec::new();
    for ((_field_name, chart_id, chart_title), map) in fields.iter().zip(maps.into_iter()) {
        if !map.is_empty() {
            let kind = if map.len() <= 5 { "donut" } else { "bar" };
            sections.push(ChartSection {
                id: (*chart_id).to_string(),
                title: (*chart_title).to_string(),
                subtitle: None,
                kind: kind.to_string(),
                items: top_items_from_counts(map, total, 12, "registros"),
                is_custom: false,
                custom_id: None,
            });
        }
    }
    sections
}

/// Wrapper sem filtro para Egressos.
pub fn egressos_analytics(
    conn: &Connection,
    file_key: Option<&str>,
) -> Result<DashboardPayload, AppError> {
    egressos_analytics_filtered(conn, file_key, None)
}

/// Gera o Dashboard Geral ou os Gráficos por Tabela para o projeto Egressos com suporte a `filters`.
pub fn egressos_analytics_filtered(
    conn: &Connection,
    file_key: Option<&str>,
    filters: Option<&str>,
) -> Result<DashboardPayload, AppError> {
    let spec = parse_filter_spec(filters);
    let clean_fk = file_key.map(str::trim).filter(|s| !s.is_empty());
    let total_files = table_row_count(conn, "egressos_files")?;
    let raw_total_rows = table_row_count(conn, "egressos_rows")?;

    if let Some(fk) = clean_fk {
        let file_def = EGRESSOS_FILES.iter().find(|f| f.file_key == fk);
        let title_label = file_def.map(|f| f.title).unwrap_or(fk);
        let records = load_records_for_scope_entity(conn, "egressos", fk, &spec)?;
        let file_rows = records.len();

        let mut stmt = conn.prepare(
            "SELECT section_key, COUNT(*) FROM egressos_rows WHERE file_key = ?1 GROUP BY section_key ORDER BY 2 DESC",
        )?;
        let sec_rows = stmt.query_map([fk], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? as usize))
        })?;
        let sec_vec: Vec<(String, usize)> = sec_rows.flatten().collect();

        let kpis = vec![
            KpiCard {
                label: format!("Registros em {title_label}"),
                value: format_pt_int(file_rows),
                sublabel: Some(format!("Arquivo: {}.json", fk)),
                tone: "indigo".to_string(),
            },
            KpiCard {
                label: "Seções no Arquivo".to_string(),
                value: format_pt_int(sec_vec.len()),
                sublabel: Some("Seções estruturadas carregadas".to_string()),
                tone: "emerald".to_string(),
            },
        ];

        let mut charts = Vec::new();
        if !sec_vec.is_empty() && spec.is_empty() {
            charts.push(ChartSection {
                id: "egressos_file_sections".to_string(),
                title: format!("Registros por Seção ({title_label})"),
                subtitle: None,
                kind: "bar".to_string(),
                items: sec_vec
                    .into_iter()
                    .map(|(label, count)| ChartItem {
                        label,
                        count,
                        secondary_value: None,
                        formatted_value: Some(format!("{} registros", format_pt_int(count))),
                        percentage: pct(count, file_rows),
                    })
                    .collect(),
                is_custom: false,
                custom_id: None,
            });
        }

        if let Some(def) = file_def {
            let dynamic_fields: Vec<(&str, &str, &str)> =
                def.columns.iter().take(4).map(|&col| (col, col, col)).collect();
            for mut sec in aggregate_records_by_fields(&records, &dynamic_fields) {
                sec.id = format!("egressos_{fk}_{}", sec.id);
                sec.title = format!("Distribuição por '{}' em {}", sec.title, title_label);
                charts.push(sec);
            }
        }

        append_custom_charts(conn, "egressos", Some(fk), &spec, &mut charts)?;

        return Ok(DashboardPayload {
            available_entities: available_entities_for_scope("egressos"),
            scope: "egressos".to_string(),
            entity: Some(fk.to_string()),
            title: format!("Gráficos da Tabela — {title_label}"),
            loaded: raw_total_rows > 0 || !charts.is_empty(),
            total_rows: file_rows,
            kpis,
            charts,
        });
    }

    let mut stmt = conn.prepare(
        "SELECT file_key, COUNT(*) FROM egressos_rows GROUP BY file_key ORDER BY 2 DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? as usize))
    })?;
    let mut file_counts = Vec::new();
    for (fk, cnt) in rows.flatten() {
        let label = EGRESSOS_FILES
            .iter()
            .find(|f| f.file_key == fk)
            .map(|f| format!("{} ({})", f.title, f.file_name))
            .unwrap_or(fk);
        file_counts.push((label, cnt));
    }
    let max_fc = file_counts.first().map(|x| x.1).unwrap_or(1).max(1);

    let perfil_records = load_records_for_scope_entity(conn, "egressos", "egressos_perfil", &spec)?;
    let displayed_total = if spec.is_empty() {
        raw_total_rows
    } else {
        perfil_records.len()
    };

    let kpis = vec![
        KpiCard {
            label: "Arquivos Carregados".to_string(),
            value: format!("{total_files} / 21"),
            sublabel: Some(if total_files > 0 {
                "Projeto Egressos Ativo".to_string()
            } else {
                "Sem dados carregados".to_string()
            }),
            tone: "emerald".to_string(),
        },
        KpiCard {
            label: "Total de Registros".to_string(),
            value: format_pt_int(displayed_total),
            sublabel: Some("Registros considerados na análise".to_string()),
            tone: "indigo".to_string(),
        },
    ];

    let mut charts = Vec::new();
    if !file_counts.is_empty() && spec.is_empty() {
        charts.push(ChartSection {
            id: "egressos_files_volume".to_string(),
            title: "Volume de Registros nas 21 Tabelas de Egressos".to_string(),
            subtitle: None,
            kind: "bar".to_string(),
            items: file_counts
                .into_iter()
                .map(|(label, count)| ChartItem {
                    label,
                    count,
                    secondary_value: None,
                    formatted_value: Some(format!("{} registros", format_pt_int(count))),
                    percentage: pct(count, max_fc),
                })
                .collect(),
            is_custom: false,
            custom_id: None,
        });
    }

    charts.extend(aggregate_records_by_fields(
        &perfil_records,
        &[
            ("curso", "egressos_curso", "Egressos por Curso (Perfis)"),
            (
                "empresa_atual",
                "egressos_empresa",
                "Principais Empresas Empregadoras",
            ),
            ("area_atual", "egressos_area", "Áreas de Atuação dos Egressos"),
            (
                "local_atual",
                "egressos_local",
                "Localização Atual dos Egressos",
            ),
        ],
    ));

    append_custom_charts(conn, "egressos", None, &spec, &mut charts)?;

    Ok(DashboardPayload {
        available_entities: available_entities_for_scope("egressos"),
        scope: "egressos".to_string(),
        entity: None,
        title: "Dashboard Analítico — Egressos (IFES Campus Serra)".to_string(),
        loaded: raw_total_rows > 0 || !charts.is_empty(),
        total_rows: displayed_total,
        kpis,
        charts,
    })
}

/// Wrapper sem filtro para Egressos — Dados.
pub fn egressos_dados_analytics(
    conn: &Connection,
    file_key: Option<&str>,
) -> Result<DashboardPayload, AppError> {
    egressos_dados_analytics_filtered(conn, file_key, None)
}

/// Gera o Dashboard Geral ou os Gráficos por Tabela para Egressos — Dados (39 datasets) com suporte a `filters`.
pub fn egressos_dados_analytics_filtered(
    conn: &Connection,
    file_key: Option<&str>,
    filters: Option<&str>,
) -> Result<DashboardPayload, AppError> {
    let spec = parse_filter_spec(filters);
    let clean_fk = file_key.map(str::trim).filter(|s| !s.is_empty());
    let total_files = table_row_count(conn, "egressos_dados_files")?;
    let raw_total_rows = table_row_count(conn, "egressos_dados_rows")?;

    if let Some(fk) = clean_fk {
        let file_def = EGRESSOS_DADOS_FILES.iter().find(|f| f.file_key == fk);
        let title_label = file_def.map(|f| f.title).unwrap_or(fk);
        let records = load_records_for_scope_entity(conn, "egressos-dados", fk, &spec)?;
        let file_rows = records.len();

        let mut stmt = conn.prepare(
            "SELECT section_key, COUNT(*) FROM egressos_dados_rows WHERE file_key = ?1 GROUP BY section_key ORDER BY 2 DESC",
        )?;
        let sec_rows = stmt.query_map([fk], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? as usize))
        })?;
        let sec_vec: Vec<(String, usize)> = sec_rows.flatten().collect();

        let kpis = vec![
            KpiCard {
                label: format!("Registros em {title_label}"),
                value: format_pt_int(file_rows),
                sublabel: file_def.map(|d| {
                    format!(
                        "{} • {}",
                        d.relative_path,
                        d.publication.as_str().to_uppercase()
                    )
                }),
                tone: "indigo".to_string(),
            },
            KpiCard {
                label: "Seções no Dataset".to_string(),
                value: format_pt_int(sec_vec.len()),
                sublabel: file_def.map(|d| format!("Papel: {}", d.role.as_str())),
                tone: "emerald".to_string(),
            },
        ];

        let mut charts = Vec::new();
        if !sec_vec.is_empty() && spec.is_empty() {
            charts.push(ChartSection {
                id: "ed_file_sections".to_string(),
                title: format!("Registros por Seção ({title_label})"),
                subtitle: None,
                kind: "bar".to_string(),
                items: sec_vec
                    .into_iter()
                    .map(|(label, count)| ChartItem {
                        label,
                        count,
                        secondary_value: None,
                        formatted_value: Some(format!("{} registros", format_pt_int(count))),
                        percentage: pct(count, file_rows),
                    })
                    .collect(),
                is_custom: false,
                custom_id: None,
            });
        }

        if let Some(def) = file_def {
            let dynamic_fields: Vec<(&str, &str, &str)> = def
                .columns
                .iter()
                .filter(|&&c| c != "id" && c != "nome")
                .take(4)
                .map(|&col| (col, col, col))
                .collect();
            for mut sec in aggregate_records_by_fields(&records, &dynamic_fields) {
                sec.id = format!("ed_{fk}_{}", sec.id);
                sec.title = format!("Distribuição por '{}' em {}", sec.title, title_label);
                charts.push(sec);
            }
        }

        append_custom_charts(conn, "egressos-dados", Some(fk), &spec, &mut charts)?;

        return Ok(DashboardPayload {
            available_entities: available_entities_for_scope("egressos-dados"),
            scope: "egressos-dados".to_string(),
            entity: Some(fk.to_string()),
            title: format!("Gráficos do Dataset — {title_label}"),
            loaded: raw_total_rows > 0 || !charts.is_empty(),
            total_rows: file_rows,
            kpis,
            charts,
        });
    }

    let mut stmt = conn.prepare(
        "SELECT file_key, COUNT(*) FROM egressos_dados_rows GROUP BY file_key ORDER BY 2 DESC",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? as usize))
    })?;
    let mut file_counts = Vec::new();
    let mut pub_counts: HashMap<String, usize> = HashMap::new();
    for (fk, cnt) in rows.flatten() {
        if let Some(def) = EGRESSOS_DADOS_FILES.iter().find(|f| f.file_key == fk) {
            let pub_label = match def.publication.as_str() {
                "pii" => "PII (Sensível)",
                "interno" => "Interno",
                _ => "Publicável",
            };
            *pub_counts.entry(pub_label.to_string()).or_insert(0) += cnt;
            file_counts.push((format!("{} ({})", def.title, def.file_name), cnt));
        } else {
            file_counts.push((fk, cnt));
        }
    }
    let max_fc = file_counts.first().map(|x| x.1).unwrap_or(1).max(1);

    let alunos_records =
        load_records_for_scope_entity(conn, "egressos-dados", "alunos", &spec)?;
    let displayed_total = if spec.is_empty() {
        raw_total_rows
    } else {
        alunos_records.len()
    };

    let kpis = vec![
        KpiCard {
            label: "Datasets Carregados".to_string(),
            value: format!("{total_files} / 39"),
            sublabel: Some(if total_files > 0 {
                "Base Egressos — Dados Ativa".to_string()
            } else {
                "Sem dados carregados".to_string()
            }),
            tone: "emerald".to_string(),
        },
        KpiCard {
            label: "Total de Registros".to_string(),
            value: format_pt_int(displayed_total),
            sublabel: Some("Repositório ifesserra-lab/egressos-dados".to_string()),
            tone: "indigo".to_string(),
        },
    ];

    let mut charts = Vec::new();
    if !pub_counts.is_empty() && spec.is_empty() {
        charts.push(ChartSection {
            id: "ed_publication_breakdown".to_string(),
            title: "Registros por Classificação de Privacidade (PII / Interno / Publicável)"
                .to_string(),
            subtitle: None,
            kind: "donut".to_string(),
            items: top_items_from_counts(pub_counts, raw_total_rows, 5, "registros"),
            is_custom: false,
            custom_id: None,
        });
    }
    if !file_counts.is_empty() && spec.is_empty() {
        charts.push(ChartSection {
            id: "ed_top_datasets".to_string(),
            title: "Datasets com Maior Volume de Registros".to_string(),
            subtitle: None,
            kind: "bar".to_string(),
            items: file_counts
                .into_iter()
                .take(15)
                .map(|(label, count)| ChartItem {
                    label,
                    count,
                    secondary_value: None,
                    formatted_value: Some(format!("{} registros", format_pt_int(count))),
                    percentage: pct(count, max_fc),
                })
                .collect(),
            is_custom: false,
            custom_id: None,
        });
    }

    charts.extend(aggregate_records_by_fields(
        &alunos_records,
        &[
            (
                "ainda_em_tech",
                "ed_ainda_em_tech",
                "Permanência na Área de Tecnologia (Ativo em Tech)",
            ),
            ("curso", "ed_curso", "Coorte de Alunos por Curso"),
            (
                "empresa_atual",
                "ed_empresa",
                "Principais Empresas Empregadoras (Coorte)",
            ),
            ("area_atual", "ed_area", "Áreas de Atuação (Coorte)"),
        ],
    ));

    append_custom_charts(conn, "egressos-dados", None, &spec, &mut charts)?;

    Ok(DashboardPayload {
        available_entities: available_entities_for_scope("egressos-dados"),
        scope: "egressos-dados".to_string(),
        entity: None,
        title: "Dashboard Analítico — Egressos — Dados (39 Datasets)".to_string(),
        loaded: raw_total_rows > 0 || !charts.is_empty(),
        total_rows: displayed_total,
        kpis,
        charts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    /// T003: Testa a classificação semântica de status de projeto.
    #[test]
    fn classifies_project_status_accurately() {
        let active = classify_project_status(Some("Active"));
        assert!(active.is_active);
        assert_eq!(active.macro_label, "Ativo");
        assert_eq!(active.badge_label, "Ativo (Em Execução)");
        assert_eq!(active.tone, "emerald");

        let in_progress = classify_project_status(Some("In Progress"));
        assert!(in_progress.is_active);
        assert_eq!(in_progress.macro_label, "Ativo");
        assert_eq!(in_progress.badge_label, "Ativo (Em Andamento)");
        assert_eq!(in_progress.tone, "teal");

        let concluded = classify_project_status(Some("Concluded"));
        assert!(!concluded.is_active);
        assert_eq!(concluded.macro_label, "Inativo");
        assert_eq!(concluded.badge_label, "Inativo (Concluído)");

        let cancelled = classify_project_status(Some("Cancelled"));
        assert!(!cancelled.is_active);
        assert_eq!(cancelled.macro_label, "Inativo");
        assert_eq!(cancelled.badge_label, "Inativo (Cancelado)");

        for unk in [Some("Unknown"), Some("N/A"), Some(""), None] {
            let info = classify_project_status(unk);
            assert!(!info.is_active);
            assert_eq!(info.macro_label, "Inativo");
            assert_eq!(info.badge_label, "Inativo (Não Informado)");
        }
    }

    #[test]
    fn cleans_education_institution_noise() {
        assert_eq!(clean_education_institution(Some("(Carga Horária: 360h)")), None);
        assert_eq!(clean_education_institution(Some("  ")), None);
        assert_eq!(
            clean_education_institution(Some(
                "Universidade Federal do Espírito Santo, UFES, Brasil"
            )),
            Some("Universidade Federal do Espírito Santo, UFES, Brasil".to_string())
        );
    }

    /// SEP-041 T003: Testa o motor `FilterSpec` (`AND` / `OR`, 5 operadores e extração de JSON estruturado).
    #[test]
    fn evaluates_filter_spec_across_scalar_and_structured_json_columns() {
        let mut rec = Record::new();
        rec.insert("name".into(), Json::String("Projeto IA Serra".into()));
        rec.insert("status".into(), Json::String("Active".into()));
        rec.insert(
            "campus".into(),
            Json::String("[{\"id\":1,\"name\":\"Serra\"}]".into()),
        );
        rec.insert("description".into(), Json::Null);

        let and_spec = parse_filter_spec(Some(
            r#"{"logic":"and","rules":[{"field":"status","operator":"equals","value":"Active"},{"field":"campus","operator":"contains","value":"Serra"},{"field":"description","operator":"is_empty","value":""}]}"#,
        ));
        assert!(record_matches_filter_spec(&rec, &and_spec));

        let fail_and = parse_filter_spec(Some(
            r#"{"logic":"and","rules":[{"field":"status","operator":"equals","value":"Concluded"},{"field":"campus","operator":"contains","value":"Serra"}]}"#,
        ));
        assert!(!record_matches_filter_spec(&rec, &fail_and));

        let pass_or = parse_filter_spec(Some(
            r#"{"logic":"or","rules":[{"field":"status","operator":"equals","value":"Concluded"},{"field":"campus","operator":"equals","value":"Serra"}]}"#,
        ));
        assert!(record_matches_filter_spec(&rec, &pass_or));
    }

    #[test]
    fn computes_workspace_status_for_all_five_projects() {
        let conn = db::open_in_memory().unwrap();
        let empty_statuses = compute_projects_status(&conn).unwrap();
        assert_eq!(empty_statuses.len(), 5);
        assert!(empty_statuses.iter().all(|s| !s.active && s.total_rows == 0));

        conn.execute(
            "INSERT INTO hv2_initiatives (id, name, status) VALUES (1, 'Proj A', 'Active'), (2, 'Proj B', 'Concluded'), (3, 'Proj C', 'In Progress')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO src_acoes (id, acao_id, raw_json, titulo) VALUES (1, 'A1', '{}', 'Ação 1')",
            [],
        )
        .unwrap();

        let statuses = compute_projects_status(&conn).unwrap();
        let hv2 = statuses.iter().find(|s| s.id == "horizon-v2").unwrap();
        assert!(hv2.active);
        assert_eq!(hv2.total_rows, 3);
        assert_eq!(hv2.active_projects, Some(2));
        assert!(hv2.status_label.contains("2 projetos ativos"));
    }

    /// T015 & T019: Testa o Dashboard Geral e por Tabela com filtros e criação/exclusão de Gráficos Personalizados (`count` e `sum`).
    #[test]
    fn computes_filtered_analytics_and_custom_charts_across_scopes() {
        let conn = db::open_in_memory().unwrap();
        conn.execute(
            "INSERT INTO hv2_initiatives (id, name, status, campus, knowledge_areas, enrichment) VALUES
             (1, 'IA Aplicada', 'Active', '[{\"id\":1,\"name\":\"Serra\"}]', '[{\"id\":10,\"name\":\"Ciência da Computação\"}]', '{\"project_code\":\"6903\"}'),
             (2, 'Robótica', 'In Progress', '[{\"id\":1,\"name\":\"Serra\"}]', '[]', NULL),
             (3, 'Antigo', 'Concluded', '[{\"id\":2,\"name\":\"Vitória\"}]', '[]', NULL)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO hv2_fellowships (id, name, description, value) VALUES
             (1, 'PIBIC', 'Iniciação Científica', '700.0'),
             (2, 'PIVIC', 'Voluntário', '0.0')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO hv2_advisorships (id, name, status, advisorships) VALUES
             (10, 'Proj Orientação', 'Active', '[{\"id\":100,\"status\":\"Active\",\"supervisor_name\":\"Prof. Thiago\",\"person_name\":\"Aluno João\",\"fellowship\":{\"name\":\"PIBIC\",\"value\":700.0}}]')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO hv2_researchers (id, name, classification, academic_education, articles, initiatives) VALUES
             (1, 'Prof. Thiago', 'researcher', '[{\"institution\":\"(Carga Horária: 360h)\",\"degree\":\"Aperfeiçoamento\"},{\"institution\":\"UFES\",\"degree\":\"Doutorado\"}]', '[{\"id\":1},{\"id\":2}]', '[{\"id\":1}]')",
            [],
        )
        .unwrap();

        // 1. Filtra o Dashboard de Iniciativas apenas para Campus Serra (2 projetos ativos, 0 concluídos)
        let filter_serra = r#"{"logic":"and","rules":[{"field":"campus","operator":"contains","value":"Serra"}]}"#;
        let filtered_init =
            horizon_analytics_filtered(&conn, true, Some("initiatives"), Some(filter_serra))
                .unwrap();
        assert_eq!(filtered_init.total_rows, 2);
        assert!(filtered_init
            .kpis
            .iter()
            .any(|k| k.label == "Projetos Ativos" && k.value == "2"));

        // 2. Cria um gráfico personalizado em `initiatives` filtrado por Campus Serra agrupado por `status` (metric = count)
        let created = create_custom_chart(
            &conn,
            &NewCustomChartInput {
                scope: "horizon-v2".to_string(),
                entity: "initiatives".to_string(),
                title: "Status em Serra (Personalizado)".to_string(),
                group_by: "status".to_string(),
                metric: "count".to_string(),
                metric_field: None,
                kind: "donut".to_string(),
                filter_logic: "and".to_string(),
                filters: vec![ColumnFilter {
                    field: "campus".to_string(),
                    operator: "contains".to_string(),
                    value: "Serra".to_string(),
                }],
            },
        )
        .unwrap();
        assert!(created.id > 0);

        // 3. Cria um gráfico personalizado com métrica `sum` somando a quantidade de vínculos `articles` em `researchers`
        let created_sum = create_custom_chart(
            &conn,
            &NewCustomChartInput {
                scope: "horizon-v2".to_string(),
                entity: "researchers".to_string(),
                title: "Artigos por Classificação".to_string(),
                group_by: "classification".to_string(),
                metric: "sum".to_string(),
                metric_field: Some("articles".to_string()),
                kind: "bar".to_string(),
                filter_logic: "and".to_string(),
                filters: vec![],
            },
        )
        .unwrap();

        // O gráfico personalizado de initiatives aparece tanto no painel da tabela initiatives quanto no Dashboard Geral (Clarificação Q2)
        let table_dash =
            horizon_analytics_filtered(&conn, true, Some("initiatives"), None).unwrap();
        assert!(table_dash
            .charts
            .iter()
            .any(|c| c.is_custom && c.custom_id == Some(created.id)));

        let general_dash = horizon_analytics_filtered(&conn, true, None, None).unwrap();
        assert!(general_dash
            .charts
            .iter()
            .any(|c| c.is_custom && c.custom_id == Some(created.id)));
        let sum_chart = general_dash
            .charts
            .iter()
            .find(|c| c.custom_id == Some(created_sum.id))
            .unwrap();
        assert_eq!(sum_chart.items[0].count, 2);

        // Exclusão de gráfico personalizado
        delete_custom_chart(&conn, created.id).unwrap();
        let remaining = list_custom_charts(&conn, "horizon-v2", Some("initiatives")).unwrap();
        assert!(remaining.is_empty());
    }

    #[test]
    fn computes_src_egressos_and_egressos_dados_analytics() {
        let conn = db::open_in_memory().unwrap();
        conn.execute(
            "INSERT INTO src_acoes (id, acao_id, raw_json, titulo, natureza, tipo, coordenador, campus, total_participacoes)
             VALUES (1, '10', '{}', 'Curso Rust', 'Extensão', 'Curso', 'Prof. Ana', 'Serra', 2),
                    (2, '20', '{}', 'Pesquisa IA', 'Pesquisa', 'Projeto', 'Prof. João', 'Vitória', 0)",
            [],
        )
        .unwrap();
        let src_filtered = src_analytics_filtered(
            &conn,
            Some(r#"{"logic":"and","rules":[{"field":"natureza","operator":"equals","value":"Extensão"}]}"#),
        )
        .unwrap();
        assert_eq!(src_filtered.total_rows, 1);

        conn.execute(
            "INSERT INTO egressos_files (file_key, kind, primary_section, envelope_json) VALUES ('egressos_perfil', 'object_sections', 'egressos', '{}')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO egressos_rows (file_key, section_key, ord, raw_json) VALUES
             ('egressos_perfil', 'egressos', 0, '{\"nome\":\"Carlos\",\"curso\":\"BSI\",\"empresa_atual\":\"Banestes\",\"area_atual\":\"Backend\"}'),
             ('egressos_perfil', 'egressos', 1, '{\"nome\":\"Ana\",\"curso\":\"TPC\",\"empresa_atual\":\"Vale\",\"area_atual\":\"Dados\"}')",
            [],
        )
        .unwrap();
        let eg_filtered = egressos_analytics_filtered(
            &conn,
            Some("egressos_perfil"),
            Some(r#"{"logic":"and","rules":[{"field":"curso","operator":"equals","value":"BSI"}]}"#),
        )
        .unwrap();
        assert_eq!(eg_filtered.total_rows, 1);

        conn.execute(
            "INSERT INTO egressos_dados_files (file_key, kind, primary_section, envelope_json) VALUES ('alunos', 'object_sections', 'alunos', '{}')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO egressos_dados_rows (file_key, section_key, ord, raw_json) VALUES
             ('alunos', 'alunos', 0, '{\"id\":\"a1\",\"nome\":\"Carlos\",\"curso\":\"BSI\",\"ainda_em_tech\":true}'),
             ('alunos', 'alunos', 1, '{\"id\":\"a2\",\"nome\":\"Ana\",\"curso\":\"TPC\",\"ainda_em_tech\":false}')",
            [],
        )
        .unwrap();
        let ed_filtered = egressos_dados_analytics_filtered(
            &conn,
            Some("alunos"),
            Some(r#"{"logic":"and","rules":[{"field":"ainda_em_tech","operator":"equals","value":"Sim"}]}"#),
        )
        .unwrap();
        assert_eq!(ed_filtered.total_rows, 1);
    }
}
