//! Domínio Egressos: catálogo das 21 abas JSON da pasta `dados/`, importação e
//! exportação sem perdas (*round-trip*), CRUD por aba/seção e propagação
//! opcional entre tabelas relacionadas (SEP-035).
//!
//! Decisões de pesquisa aplicadas (`specs/035-egressos-data-workspace/research.md`):
//! - R1: tabelas dedicadas `egressos_files` e `egressos_rows` em `003_egressos_tables.sql`
//! - R2: catálogo estático `EGRESSOS_FILES` 1:1 com os 21 arquivos de `/home/thiagodeps/egressos/dados`
//! - R3: propagação opcional (`propagate: bool`) para os grupos `Alumni` e `Company` (FR-008a)
//! - R4: importação e exportação direta de pasta local (`dados/`) com snapshot `egressos-*`
//! - R5: sincronização GitHub de diretório em commit único via Git Data API

use crate::crud::Record;
use crate::error::AppError;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value as Json};
use std::collections::HashMap;
use std::path::Path;

/// Caminho local padrão (vazio por padrão — o curador escolhe a pasta via diálogo ou link GitHub).
pub const DEFAULT_LOCAL_DADOS_DIR: &str = "";

const SENTINEL_SECTION_KEY: &str = "__egressos_section__";
const SENTINEL_SHAPE_KEY: &str = "__section_shape__";

const SHAPE_ARRAY_OBJECTS: &str = "array_objects";
const SHAPE_ARRAY_SCALARS: &str = "array_scalars";
const SHAPE_MAP_OBJECTS: &str = "map_objects";
const SHAPE_MAP_SCALARS: &str = "map_scalars";

/// Tipo estrutural do arquivo JSON raiz (`data-model.md` §1.1 e §2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileKind {
    ObjectSections,
    RootMap,
    NestedMap,
}

impl FileKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ObjectSections => "object_sections",
            Self::RootMap => "root_map",
            Self::NestedMap => "nested_map",
        }
    }
}

/// Grupo de entidade para propagação cruzada opcional (Clarification Q2 / FR-008a).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityGroup {
    Alumni,
    Company,
    None,
}

impl EntityGroup {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Alumni => "alumni",
            Self::Company => "company",
            Self::None => "none",
        }
    }
}

/// Definição estática de cada um dos 21 arquivos JSON de `dados/`.
#[derive(Clone, Copy, Debug)]
pub struct EgressosFileDef {
    pub file_key: &'static str,
    pub file_name: &'static str,
    pub title: &'static str,
    pub kind: FileKind,
    pub primary_section: &'static str,
    pub sections: &'static [&'static str],
    pub columns: &'static [&'static str],
    pub entity_group: EntityGroup,
}

/// Catálogo canônico 1:1 dos 21 arquivos JSON da pasta `/home/thiagodeps/egressos/dados` (FR-007).
pub const EGRESSOS_FILES: &[EgressosFileDef; 21] = &[
    EgressosFileDef {
        file_key: "egressos_perfil",
        file_name: "egressos_perfil.json",
        title: "Perfis de Egressos",
        kind: FileKind::ObjectSections,
        primary_section: "egressos",
        sections: &["egressos", "cursos"],
        columns: &[
            "label",
            "nome",
            "curso",
            "cargo_atual",
            "empresa_atual",
            "local_atual",
            "area_atual",
        ],
        entity_group: EntityGroup::Alumni,
    },
    EgressosFileDef {
        file_key: "vitrine",
        file_name: "vitrine.json",
        title: "Vitrine de Egressos",
        kind: FileKind::ObjectSections,
        primary_section: "perfis",
        sections: &[
            "perfis",
            "empresas",
            "empresas_verificadas",
            "contratando",
            "porte",
            "setor",
            "regiao_das_empresas",
            "paises_no_exterior",
            "trilhas",
            "regioes",
        ],
        columns: &["rotulo", "nome", "curso", "cargo", "empresa", "lugar"],
        entity_group: EntityGroup::Alumni,
    },
    EgressosFileDef {
        file_key: "panorama",
        file_name: "panorama.json",
        title: "Panorama do Coorte",
        kind: FileKind::ObjectSections,
        primary_section: "coorte",
        sections: &["coorte", "salto", "areas", "familias", "origem_comum"],
        columns: &[
            "rotulo",
            "cargo_atual",
            "empregador_atual",
            "origem",
            "modalidade",
            "area_atual_nome",
            "inicio_carreira",
        ],
        entity_group: EntityGroup::Alumni,
    },
    EgressosFileDef {
        file_key: "analise",
        file_name: "analise.json",
        title: "Análise do Grupo",
        kind: FileKind::ObjectSections,
        primary_section: "rows",
        sections: &[
            "rows",
            "top_tech",
            "funcoes",
            "metodos",
            "intl_timeline",
            "trajetoria_destaque",
            "hist_multiplicadores",
        ],
        columns: &[
            "label",
            "curso",
            "anos",
            "n_empresas",
            "n_exp",
            "trilha",
            "em_tech",
            "exterior",
        ],
        entity_group: EntityGroup::Alumni,
    },
    EgressosFileDef {
        file_key: "consolidado",
        file_name: "consolidado.json",
        title: "Consolidado Salarial",
        kind: FileKind::ObjectSections,
        primary_section: "perfis",
        sections: &["perfis", "agregado", "pv", "por_ano_sm", "series_por_perfil"],
        columns: &[
            "perfil",
            "trilha",
            "em_tech",
            "anos",
            "bolsa",
            "med_ini",
            "med_atual",
            "cresc",
        ],
        entity_group: EntityGroup::Alumni,
    },
    EgressosFileDef {
        file_key: "empresas_porte",
        file_name: "empresas_porte.json",
        title: "Empresas — Porte e Setor",
        kind: FileKind::RootMap,
        primary_section: "_root",
        sections: &["_root"],
        columns: &["_key", "empresa", "porte", "origem", "setor", "funcionarios"],
        entity_group: EntityGroup::Company,
    },
    EgressosFileDef {
        file_key: "empresas_aliases",
        file_name: "empresas_aliases.json",
        title: "Empresas — Aliases",
        kind: FileKind::RootMap,
        primary_section: "_root",
        sections: &["_root"],
        columns: &["_key", "aliases", "atual", "n_egressos_atual"],
        entity_group: EntityGroup::Company,
    },
    EgressosFileDef {
        file_key: "empresas_linkedin_data",
        file_name: "empresas_linkedin_data.json",
        title: "Empresas — Dados LinkedIn",
        kind: FileKind::RootMap,
        primary_section: "_root",
        sections: &["_root"],
        columns: &[
            "_key",
            "nome_oficial",
            "industry_linkedin",
            "headcount_linkedin",
            "hq_local",
            "website",
            "founded",
        ],
        entity_group: EntityGroup::Company,
    },
    EgressosFileDef {
        file_key: "empresas_linkedin_urls",
        file_name: "empresas_linkedin_urls.json",
        title: "Empresas — URLs LinkedIn",
        kind: FileKind::RootMap,
        primary_section: "_root",
        sections: &["_root"],
        columns: &["_key", "url", "slug", "confianca", "via"],
        entity_group: EntityGroup::Company,
    },
    EgressosFileDef {
        file_key: "fapes_fomento",
        file_name: "fapes_fomento.json",
        title: "Fomento FAPES / PRODEST",
        kind: FileKind::ObjectSections,
        primary_section: "projetos",
        sections: &["projetos"],
        columns: &[
            "projeto_id",
            "titulo",
            "situacao",
            "instituicao",
            "bolsa_sigla",
            "bolsistas",
            "valor_alocado",
        ],
        entity_group: EntityGroup::None,
    },
    EgressosFileDef {
        file_key: "impacto",
        file_name: "impacto.json",
        title: "Indicadores de Impacto",
        kind: FileKind::ObjectSections,
        primary_section: "indicadores",
        sections: &[
            "indicadores",
            "por_experiencia",
            "por_experiencia_hoje",
            "achados",
            "ressalvas",
            "dispersao",
            "vias_de_formacao",
            "internacional_no_tempo",
            "funcoes",
            "metodos",
            "tecnologias",
            "senioridade",
            "perfis_agrupados",
        ],
        columns: &["rotulo", "valor"],
        entity_group: EntityGroup::None,
    },
    EgressosFileDef {
        file_key: "trajetoria",
        file_name: "trajetoria.json",
        title: "Trajetória Salarial",
        kind: FileKind::ObjectSections,
        primary_section: "serie",
        sections: &[
            "serie",
            "trajetoria",
            "trajetoria_coorte",
            "sm_por_ano",
            "regua",
            "tabela_em_reais",
            "por_origem_do_empregador",
            "multiplicadores",
            "corroboracao_por_edicao",
            "moeda_por_faixa",
        ],
        columns: &["ano", "n", "med_real", "med_nominal", "sm_ano", "em_sm"],
        entity_group: EntityGroup::None,
    },
    EgressosFileDef {
        file_key: "renda_por_senioridade",
        file_name: "renda_por_senioridade.json",
        title: "Renda por Senioridade",
        kind: FileKind::ObjectSections,
        primary_section: "cargos",
        sections: &["cargos", "niveis_sem_referencia_do_modelo"],
        columns: &["nivel", "referencias", "por_trilha_no_modelo"],
        entity_group: EntityGroup::None,
    },
    EgressosFileDef {
        file_key: "cargos_ao_longo_do_tempo",
        file_name: "cargos_ao_longo_do_tempo.json",
        title: "Cargos ao Longo do Tempo",
        kind: FileKind::ObjectSections,
        primary_section: "cargos",
        sections: &["cargos", "edicoes"],
        columns: &["cargo", "nome", "n_edicoes", "de", "ate", "variacao_pct"],
        entity_group: EntityGroup::None,
    },
    EgressosFileDef {
        file_key: "codigofonte_2026",
        file_name: "codigofonte_2026.json",
        title: "Pesquisa Código Fonte 2026",
        kind: FileKind::ObjectSections,
        primary_section: "por_senioridade",
        sections: &[
            "por_senioridade",
            "por_contrato",
            "por_regiao_top10",
            "por_linguagem",
            "recortes_indisponiveis_na_pagina",
            "observacoes",
        ],
        columns: &["nivel", "media"],
        entity_group: EntityGroup::None,
    },
    EgressosFileDef {
        file_key: "codigofonte_historico",
        file_name: "codigofonte_historico.json",
        title: "Pesquisa Código Fonte — Histórico",
        kind: FileKind::NestedMap,
        primary_section: "media_por_senioridade",
        sections: &["media_por_senioridade", "ipca_fator_para_2026"],
        columns: &[
            "_key",
            "Estágio",
            "Júnior",
            "Pleno",
            "Sênior",
            "Especialista/Tech Lead",
        ],
        entity_group: EntityGroup::None,
    },
    EgressosFileDef {
        file_key: "salario_minimo",
        file_name: "salario_minimo.json",
        title: "Salário Mínimo Nacional",
        kind: FileKind::ObjectSections,
        primary_section: "por_ano",
        sections: &["por_ano", "notas"],
        columns: &[
            "ano",
            "jan",
            "dez",
            "media_ponderada",
            "reajuste_pct",
            "ganho_real_pct",
        ],
        entity_group: EntityGroup::None,
    },
    EgressosFileDef {
        file_key: "ibge_series",
        file_name: "ibge_series.json",
        title: "Séries IBGE / Macroeconômicas",
        kind: FileKind::NestedMap,
        primary_section: "series",
        sections: &["series"],
        columns: &["_key", "fonte", "unidade"],
        entity_group: EntityGroup::None,
    },
    EgressosFileDef {
        file_key: "so_benchmarks",
        file_name: "so_benchmarks.json",
        title: "Stack Overflow Benchmarks",
        kind: FileKind::ObjectSections,
        primary_section: "por_pais",
        sections: &[
            "por_pais",
            "serie_anual",
            "por_moeda_brasil",
            "candidatas_a_referencia",
            "notas",
        ],
        columns: &["pais", "usd_mes", "n"],
        entity_group: EntityGroup::None,
    },
    EgressosFileDef {
        file_key: "recorte_cursos",
        file_name: "recorte_cursos.json",
        title: "Recorte por Cursos (BSI e ECA)",
        kind: FileKind::NestedMap,
        primary_section: "cursos",
        sections: &["cursos"],
        columns: &["_key", "nome", "sigla", "n"],
        entity_group: EntityGroup::None,
    },
    EgressosFileDef {
        file_key: "mapa_mundi",
        file_name: "mapa_mundi.json",
        title: "Mapa-Múndi (Camada Base)",
        kind: FileKind::ObjectSections,
        primary_section: "paths",
        sections: &["paths", "viewBox", "lat_range"],
        columns: &["value"],
        entity_group: EntityGroup::None,
    },
];

/// Busca a definição de um arquivo pelo seu `file_key` canônico.
pub fn by_file_key(file_key: &str) -> Result<&'static EgressosFileDef, AppError> {
    EGRESSOS_FILES
        .iter()
        .find(|f| f.file_key == file_key)
        .ok_or_else(|| AppError::UnknownEntity(format!("egressos:{file_key}")))
}

/// Retorna o grupo de entidade de uma seção específica (para propagação FR-008a).
pub fn section_entity_group(file_key: &str, section_key: &str) -> EntityGroup {
    match (file_key, section_key) {
        ("egressos_perfil", "egressos")
        | ("vitrine", "perfis")
        | ("panorama", "coorte")
        | ("panorama", "salto")
        | ("analise", "rows")
        | ("consolidado", "perfis") => EntityGroup::Alumni,
        ("empresas_porte", "_root")
        | ("empresas_aliases", "_root")
        | ("empresas_linkedin_data", "_root")
        | ("empresas_linkedin_urls", "_root")
        | ("vitrine", "empresas")
        | ("vitrine", "empresas_verificadas")
        | ("vitrine", "contratando") => EntityGroup::Company,
        _ => EntityGroup::None,
    }
}

// ----------------------------------------------------------------- payloads IPC

#[derive(Serialize, Debug, Clone)]
pub struct EgressosImportProgress {
    pub phase: &'static str,
    pub file_key: Option<String>,
    pub files_done: usize,
    pub files_total: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EgressosImportSummary {
    pub path: String,
    pub total_files: usize,
    pub total_rows: usize,
    pub replaced: bool,
    pub snapshot: Option<String>,
    pub imported_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EgressosExportSummary {
    pub path: String,
    pub total_files: usize,
    pub total_rows: usize,
}

#[derive(Serialize, Debug, Clone)]
pub struct EgressosSectionSummary {
    pub section_key: String,
    pub count: usize,
    pub entity_group: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct EgressosFileSummary {
    pub file_key: String,
    pub file_name: String,
    pub title: String,
    pub kind: String,
    pub primary_section: String,
    pub sections: Vec<EgressosSectionSummary>,
    pub columns: Vec<String>,
    pub entity_group: String,
    pub total_rows: usize,
}

#[derive(Serialize, Debug, Clone)]
pub struct EgressosCatalogView {
    pub loaded: bool,
    pub imported_at: Option<String>,
    pub total_files: usize,
    pub total_rows: usize,
    pub default_local_dir: String,
    pub default_local_dir_exists: bool,
    pub files: Vec<EgressosFileSummary>,
}

#[derive(Serialize, Debug, Clone)]
pub struct EgressosRowPage {
    pub items: Vec<Record>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub file_key: String,
    pub section_key: String,
    pub columns: Vec<String>,
    pub entity_group: String,
    pub is_map_section: bool,
}

#[derive(Serialize, Debug, Clone)]
pub struct EgressosMutationResult {
    pub record: Record,
    pub propagated_count: usize,
}

#[derive(Serialize, Debug, Clone)]
pub struct EgressosDeleteResult {
    pub deleted: usize,
    pub propagated_deleted: usize,
}

// ------------------------------------------------ helpers de chave e busca

fn normalize_alumni_key(raw: &str) -> String {
    let trimmed = raw.trim();
    trimmed
        .strip_prefix("Perfil ")
        .unwrap_or(trimmed)
        .trim()
        .to_string()
}

fn extract_record_key(file_key: &str, section_key: &str, obj: &Map<String, Json>) -> Option<String> {
    if let Some(k) = obj.get("_key").and_then(|v| v.as_str()) {
        let t = k.trim();
        if !t.is_empty() {
            return Some(t.to_string());
        }
    }
    match section_entity_group(file_key, section_key) {
        EntityGroup::Alumni => {
            for field in ["label", "rotulo", "perfil"] {
                if let Some(s) = obj.get(field).and_then(|v| v.as_str()) {
                    let norm = normalize_alumni_key(s);
                    if !norm.is_empty() {
                        return Some(norm);
                    }
                }
            }
            None
        }
        EntityGroup::Company => {
            for field in ["_key", "nome", "empresa"] {
                if let Some(s) = obj.get(field).and_then(|v| v.as_str()) {
                    let t = s.trim();
                    if !t.is_empty() {
                        return Some(t.to_string());
                    }
                }
            }
            None
        }
        EntityGroup::None => {
            for field in [
                "projeto_id",
                "id",
                "ano",
                "nivel",
                "cargo",
                "pais",
                "rotulo",
                "label",
                "area",
                "familia",
            ] {
                if let Some(v) = obj.get(field) {
                    if let Some(s) = v.as_str() {
                        let t = s.trim();
                        if !t.is_empty() {
                            return Some(t.to_string());
                        }
                    } else if v.is_number() {
                        return Some(v.to_string());
                    }
                }
            }
            None
        }
    }
}

fn build_search_text(obj: &Map<String, Json>) -> String {
    let mut parts = Vec::new();
    for (k, v) in obj {
        if k.starts_with("__") {
            continue;
        }
        match v {
            Json::String(s) => parts.push(s.to_lowercase()),
            Json::Number(n) => parts.push(n.to_string()),
            Json::Bool(b) => parts.push(b.to_string()),
            Json::Array(arr) => {
                for item in arr {
                    if let Some(s) = item.as_str() {
                        parts.push(s.to_lowercase());
                    } else if item.is_number() {
                        parts.push(item.to_string());
                    }
                }
            }
            _ => {}
        }
    }
    parts.join(" ")
}

fn now_iso_stamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

fn has_egressos_data(conn: &Connection) -> Result<bool, AppError> {
    let n: i64 = conn.query_row("SELECT COUNT(*) FROM egressos_files", [], |r| r.get(0))?;
    Ok(n > 0)
}

fn is_scalar_list_section(section_key: &str) -> bool {
    matches!(
        section_key,
        "notas"
            | "trilhas"
            | "regioes"
            | "edicoes"
            | "paths"
            | "viewBox"
            | "lat_range"
            | "trajetoria_coorte"
            | "recortes_indisponiveis_na_pagina"
            | "observacoes"
    )
}

// ------------------------------------------------ decomposição & validação

struct ParsedRow {
    section_key: String,
    ord: usize,
    record_key: Option<String>,
    search_text: String,
    raw_json: String,
}

struct ParsedFile {
    def: &'static EgressosFileDef,
    envelope_json: String,
    rows: Vec<ParsedRow>,
}

fn parse_and_validate_single_file(
    def: &'static EgressosFileDef,
    text: &str,
) -> Result<ParsedFile, AppError> {
    let root: Json = serde_json::from_str(text).map_err(|e| {
        AppError::Validation(format!(
            "arquivo '{}' não é um JSON válido: {e}",
            def.file_name
        ))
    })?;
    let root_obj = root.as_object().ok_or_else(|| {
        AppError::Validation(format!(
            "arquivo '{}' deve ter um objeto JSON na raiz",
            def.file_name
        ))
    })?;

    let mut rows = Vec::new();

    if def.kind == FileKind::RootMap {
        for (ord, (k, v)) in root_obj.iter().enumerate() {
            let val_obj = v.as_object().ok_or_else(|| {
                AppError::Validation(format!(
                    "arquivo '{}': entrada '{}' deve ser um objeto JSON",
                    def.file_name, k
                ))
            })?;
            let mut row_map = Map::new();
            row_map.insert("_key".to_string(), Json::String(k.clone()));
            for (fk, fv) in val_obj {
                row_map.insert(fk.clone(), fv.clone());
            }
            let search_text = build_search_text(&row_map);
            rows.push(ParsedRow {
                section_key: "_root".to_string(),
                ord,
                record_key: Some(k.clone()),
                search_text,
                raw_json: Json::Object(row_map).to_string(),
            });
        }
        return Ok(ParsedFile {
            def,
            envelope_json: "{}".to_string(),
            rows,
        });
    }

    if !root_obj.contains_key(def.primary_section) {
        return Err(AppError::Validation(format!(
            "arquivo '{}' não possui a seção obrigatória '{}'",
            def.file_name, def.primary_section
        )));
    }

    let mut envelope_map = Map::new();
    for (k, v) in root_obj {
        if def.sections.contains(&k.as_str()) {
            let shape = match v {
                Json::Array(arr) => {
                    let all_objects = if arr.is_empty() {
                        !is_scalar_list_section(k)
                    } else {
                        arr.iter().all(|item| item.is_object())
                    };
                    if all_objects {
                        for (ord, item) in arr.iter().enumerate() {
                            let item_obj = item.as_object().unwrap();
                            let rec_key = extract_record_key(def.file_key, k, item_obj);
                            let search_text = build_search_text(item_obj);
                            rows.push(ParsedRow {
                                section_key: k.clone(),
                                ord,
                                record_key: rec_key,
                                search_text,
                                raw_json: Json::Object(item_obj.clone()).to_string(),
                            });
                        }
                        SHAPE_ARRAY_OBJECTS
                    } else {
                        for (ord, item) in arr.iter().enumerate() {
                            let mut wrapper = Map::new();
                            wrapper.insert("value".to_string(), item.clone());
                            let rec_key = item
                                .as_str()
                                .map(str::to_string)
                                .or_else(|| Some(ord.to_string()));
                            let search_text = build_search_text(&wrapper);
                            rows.push(ParsedRow {
                                section_key: k.clone(),
                                ord,
                                record_key: rec_key,
                                search_text,
                                raw_json: Json::Object(wrapper).to_string(),
                            });
                        }
                        SHAPE_ARRAY_SCALARS
                    }
                }
                Json::Object(sub_map) => {
                    let all_objects =
                        !sub_map.is_empty() && sub_map.values().all(|val| val.is_object());
                    if all_objects {
                        for (ord, (sub_k, sub_v)) in sub_map.iter().enumerate() {
                            let sub_obj = sub_v.as_object().unwrap();
                            let mut row_map = Map::new();
                            row_map.insert("_key".to_string(), Json::String(sub_k.clone()));
                            for (fk, fv) in sub_obj {
                                row_map.insert(fk.clone(), fv.clone());
                            }
                            let search_text = build_search_text(&row_map);
                            rows.push(ParsedRow {
                                section_key: k.clone(),
                                ord,
                                record_key: Some(sub_k.clone()),
                                search_text,
                                raw_json: Json::Object(row_map).to_string(),
                            });
                        }
                        SHAPE_MAP_OBJECTS
                    } else {
                        for (ord, (sub_k, sub_v)) in sub_map.iter().enumerate() {
                            let mut row_map = Map::new();
                            row_map.insert("_key".to_string(), Json::String(sub_k.clone()));
                            row_map.insert("value".to_string(), sub_v.clone());
                            let search_text = build_search_text(&row_map);
                            rows.push(ParsedRow {
                                section_key: k.clone(),
                                ord,
                                record_key: Some(sub_k.clone()),
                                search_text,
                                raw_json: Json::Object(row_map).to_string(),
                            });
                        }
                        SHAPE_MAP_SCALARS
                    }
                }
                _ => {
                    return Err(AppError::Validation(format!(
                        "arquivo '{}': seção '{}' deve ser uma lista ou objeto JSON",
                        def.file_name, k
                    )));
                }
            };

            let mut sentinel = Map::new();
            sentinel.insert(
                SENTINEL_SECTION_KEY.to_string(),
                Json::String(k.clone()),
            );
            sentinel.insert(
                SENTINEL_SHAPE_KEY.to_string(),
                Json::String(shape.to_string()),
            );
            envelope_map.insert(k.clone(), Json::Object(sentinel));
        } else {
            envelope_map.insert(k.clone(), v.clone());
        }
    }

    Ok(ParsedFile {
        def,
        envelope_json: Json::Object(envelope_map).to_string(),
        rows,
    })
}

/// Importa e valida atomicamente os 21 arquivos JSON de um diretório local
/// (`FR-004`, `FR-005`, `FR-006`).
pub fn import_dir<F>(
    conn: &mut Connection,
    source_dir: &Path,
    app_data_dir: &Path,
    mut on_progress: F,
) -> Result<EgressosImportSummary, AppError>
where
    F: FnMut(EgressosImportProgress),
{
    if !source_dir.exists() || !source_dir.is_dir() {
        return Err(AppError::Validation(format!(
            "A pasta '{}' não existe ou não é um diretório válido.",
            source_dir.display()
        )));
    }

    on_progress(EgressosImportProgress {
        phase: "validating",
        file_key: None,
        files_done: 0,
        files_total: EGRESSOS_FILES.len(),
    });

    // C1: Verifica todos os 21 arquivos ANTES de abrir transação.
    let mut missing = Vec::new();
    for def in EGRESSOS_FILES {
        if !source_dir.join(def.file_name).is_file() {
            missing.push(def.file_name);
        }
    }
    if !missing.is_empty() {
        return Err(AppError::Validation(format!(
            "Pasta do Egressos incompleta — arquivos JSON obrigatórios ausentes: {}",
            missing.join(", ")
        )));
    }

    let mut parsed_files = Vec::with_capacity(EGRESSOS_FILES.len());
    for def in EGRESSOS_FILES {
        let file_path = source_dir.join(def.file_name);
        let text = std::fs::read_to_string(&file_path)?;
        let parsed = parse_and_validate_single_file(def, &text)?;
        parsed_files.push(parsed);
    }

    // FR-006: cria snapshot apenas quando já havia dados do Egressos carregados.
    let replaced = has_egressos_data(conn)?;
    let snapshot = if replaced {
        Some(
            crate::import::snapshot_with_prefix(conn, app_data_dir, "egressos-pre-import")?
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string(),
        )
    } else {
        None
    };

    let imported_at = now_iso_stamp();
    let mut total_rows = 0usize;

    let tx = conn.transaction()?;
    tx.execute("DELETE FROM egressos_rows", [])?;
    tx.execute("DELETE FROM egressos_files", [])?;

    for (idx, pf) in parsed_files.iter().enumerate() {
        tx.execute(
            "INSERT INTO egressos_files (file_key, kind, primary_section, envelope_json, imported_at) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                pf.def.file_key,
                pf.def.kind.as_str(),
                pf.def.primary_section,
                pf.envelope_json,
                imported_at,
            ],
        )?;

        for r in &pf.rows {
            tx.execute(
                "INSERT INTO egressos_rows (file_key, section_key, ord, record_key, search_text, raw_json) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    pf.def.file_key,
                    r.section_key,
                    r.ord as i64,
                    r.record_key,
                    r.search_text,
                    r.raw_json,
                ],
            )?;
            total_rows += 1;
        }

        on_progress(EgressosImportProgress {
            phase: "importing",
            file_key: Some(pf.def.file_key.to_string()),
            files_done: idx + 1,
            files_total: EGRESSOS_FILES.len(),
        });
    }

    tx.commit()?;

    on_progress(EgressosImportProgress {
        phase: "done",
        file_key: None,
        files_done: EGRESSOS_FILES.len(),
        files_total: EGRESSOS_FILES.len(),
    });

    Ok(EgressosImportSummary {
        path: source_dir.display().to_string(),
        total_files: EGRESSOS_FILES.len(),
        total_rows,
        replaced,
        snapshot,
        imported_at,
    })
}

// --------------------------------------------------------- reconstrução / export

fn strip_key_preserving_order(obj: Map<String, Json>, key_to_strip: &str) -> Map<String, Json> {
    obj.into_iter()
        .filter(|(k, _)| k != key_to_strip)
        .collect()
}

fn reconstruct_section(
    conn: &Connection,
    file_key: &str,
    section_key: &str,
    shape: &str,
) -> Result<Json, AppError> {
    let mut stmt = conn.prepare(
        "SELECT raw_json FROM egressos_rows \
         WHERE file_key = ?1 AND section_key = ?2 \
         ORDER BY ord ASC, id ASC",
    )?;
    let raw_rows: Vec<String> = stmt
        .query_map(params![file_key, section_key], |r| r.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    match shape {
        SHAPE_ARRAY_OBJECTS => {
            let mut arr = Vec::with_capacity(raw_rows.len());
            for raw in raw_rows {
                let v: Json = serde_json::from_str(&raw)
                    .map_err(|e| AppError::Internal(format!("raw_json corrompido: {e}")))?;
                arr.push(v);
            }
            Ok(Json::Array(arr))
        }
        SHAPE_ARRAY_SCALARS => {
            let mut arr = Vec::with_capacity(raw_rows.len());
            for raw in raw_rows {
                let v: Json = serde_json::from_str(&raw)
                    .map_err(|e| AppError::Internal(format!("raw_json corrompido: {e}")))?;
                let scalar = v
                    .as_object()
                    .and_then(|o| o.get("value"))
                    .cloned()
                    .unwrap_or(Json::Null);
                arr.push(scalar);
            }
            Ok(Json::Array(arr))
        }
        SHAPE_MAP_OBJECTS => {
            let mut map = Map::new();
            for raw in raw_rows {
                let v: Json = serde_json::from_str(&raw)
                    .map_err(|e| AppError::Internal(format!("raw_json corrompido: {e}")))?;
                let obj = v.as_object().cloned().unwrap_or_default();
                let key = obj
                    .get("_key")
                    .and_then(|k| k.as_str())
                    .unwrap_or_default()
                    .to_string();
                let rest = strip_key_preserving_order(obj, "_key");
                map.insert(key, Json::Object(rest));
            }
            Ok(Json::Object(map))
        }
        SHAPE_MAP_SCALARS => {
            let mut map = Map::new();
            for raw in raw_rows {
                let v: Json = serde_json::from_str(&raw)
                    .map_err(|e| AppError::Internal(format!("raw_json corrompido: {e}")))?;
                let obj = v.as_object().cloned().unwrap_or_default();
                let key = obj
                    .get("_key")
                    .and_then(|k| k.as_str())
                    .unwrap_or_default()
                    .to_string();
                let val = obj.get("value").cloned().unwrap_or(Json::Null);
                map.insert(key, val);
            }
            Ok(Json::Object(map))
        }
        other => Err(AppError::Internal(format!(
            "shape de seção desconhecido: {other}"
        ))),
    }
}

/// Reconstrói o `serde_json::Value` completo de um único arquivo `file_key`.
pub fn reconstruct_file_json(conn: &Connection, file_key: &str) -> Result<Json, AppError> {
    let def = by_file_key(file_key)?;
    let envelope_str: Option<String> = conn
        .query_row(
            "SELECT envelope_json FROM egressos_files WHERE file_key = ?1",
            [file_key],
            |r| r.get(0),
        )
        .optional()?;

    let envelope_str = envelope_str.ok_or_else(|| {
        AppError::Validation(format!(
            "O arquivo '{}' ainda não foi importado na base do Egressos.",
            def.file_name
        ))
    })?;

    if def.kind == FileKind::RootMap {
        return reconstruct_section(conn, file_key, "_root", SHAPE_MAP_OBJECTS);
    }

    let envelope: Json = serde_json::from_str(&envelope_str)
        .map_err(|e| AppError::Internal(format!("envelope_json inválido: {e}")))?;
    let envelope_obj = envelope.as_object().cloned().unwrap_or_default();

    let mut out = Map::new();
    for (k, v) in envelope_obj {
        if let Some(sentinel) = v.as_object() {
            if let (Some(sec), Some(shape)) = (
                sentinel.get(SENTINEL_SECTION_KEY).and_then(|s| s.as_str()),
                sentinel.get(SENTINEL_SHAPE_KEY).and_then(|s| s.as_str()),
            ) {
                let sec_val = reconstruct_section(conn, file_key, sec, shape)?;
                out.insert(k, sec_val);
                continue;
            }
        }
        out.insert(k, v);
    }
    Ok(Json::Object(out))
}

/// Exporta os 21 arquivos JSON em memória como `(file_name, pretty_json_bytes)`
/// para uso tanto na exportação local quanto no upload atômico para o GitHub.
pub fn export_files_in_memory(conn: &Connection) -> Result<Vec<(String, Vec<u8>)>, AppError> {
    if !has_egressos_data(conn)? {
        return Err(AppError::Validation(
            "Nenhuma base do Egressos carregada para exportar.".into(),
        ));
    }

    let mut out = Vec::with_capacity(EGRESSOS_FILES.len());
    for def in EGRESSOS_FILES {
        let json_val = reconstruct_file_json(conn, def.file_key)?;
        let mut pretty = serde_json::to_string_pretty(&json_val)
            .map_err(|e| AppError::Internal(format!("falha ao serializar {}: {e}", def.file_name)))?;
        pretty.push('\n');
        out.push((def.file_name.to_string(), pretty.into_bytes()));
    }
    Ok(out)
}

/// Exporta os 21 arquivos `.json` diretamente para a pasta local escolhida (`FR-010`).
pub fn export_dir(conn: &Connection, target_dir: &Path) -> Result<EgressosExportSummary, AppError> {
    let files = export_files_in_memory(conn)?;
    std::fs::create_dir_all(target_dir)?;

    for (file_name, bytes) in &files {
        std::fs::write(target_dir.join(file_name), bytes)?;
    }

    let total_rows: i64 =
        conn.query_row("SELECT COUNT(*) FROM egressos_rows", [], |r| r.get(0))?;

    Ok(EgressosExportSummary {
        path: target_dir.display().to_string(),
        total_files: files.len(),
        total_rows: total_rows as usize,
    })
}

/// Retorna a visão completa do catálogo das 21 abas e contagens atuais (`egressos_get_catalog`).
pub fn get_catalog(conn: &Connection) -> Result<EgressosCatalogView, AppError> {
    let loaded = has_egressos_data(conn)?;
    let imported_at: Option<String> = conn
        .query_row(
            "SELECT MAX(imported_at) FROM egressos_files",
            [],
            |r| r.get(0),
        )
        .optional()?
        .flatten();

    let mut count_stmt = conn.prepare(
        "SELECT file_key, section_key, COUNT(*) FROM egressos_rows GROUP BY file_key, section_key",
    )?;
    let mut section_counts: HashMap<(String, String), usize> = HashMap::new();
    let rows = count_stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, i64>(2)? as usize,
        ))
    })?;
    for row in rows {
        let (fk, sk, c) = row?;
        section_counts.insert((fk, sk), c);
    }

    let mut total_rows = 0usize;
    let mut files = Vec::with_capacity(EGRESSOS_FILES.len());
    for def in EGRESSOS_FILES {
        let mut sections = Vec::with_capacity(def.sections.len());
        let mut file_total = 0usize;
        for &sec in def.sections {
            let c = section_counts
                .get(&(def.file_key.to_string(), sec.to_string()))
                .copied()
                .unwrap_or(0);
            file_total += c;
            sections.push(EgressosSectionSummary {
                section_key: sec.to_string(),
                count: c,
                entity_group: section_entity_group(def.file_key, sec).as_str().to_string(),
            });
        }
        total_rows += file_total;
        files.push(EgressosFileSummary {
            file_key: def.file_key.to_string(),
            file_name: def.file_name.to_string(),
            title: def.title.to_string(),
            kind: def.kind.as_str().to_string(),
            primary_section: def.primary_section.to_string(),
            sections,
            columns: def.columns.iter().map(|c| (*c).to_string()).collect(),
            entity_group: def.entity_group.as_str().to_string(),
            total_rows: file_total,
        });
    }

    Ok(EgressosCatalogView {
        loaded,
        imported_at,
        total_files: if loaded { EGRESSOS_FILES.len() } else { 0 },
        total_rows,
        default_local_dir: String::new(),
        default_local_dir_exists: false,
        files,
    })
}

// --------------------------------------------------- CRUD & Propagação (US3)

fn section_shape_from_db(
    conn: &Connection,
    def: &EgressosFileDef,
    section_key: &str,
) -> Result<String, AppError> {
    if def.kind == FileKind::RootMap {
        return Ok(SHAPE_MAP_OBJECTS.to_string());
    }
    let env_str: Option<String> = conn
        .query_row(
            "SELECT envelope_json FROM egressos_files WHERE file_key = ?1",
            [def.file_key],
            |r| r.get(0),
        )
        .optional()?;
    if let Some(s) = env_str {
        if let Ok(Json::Object(map)) = serde_json::from_str::<Json>(&s) {
            if let Some(Json::Object(sentinel)) = map.get(section_key) {
                if let Some(shape) = sentinel.get(SENTINEL_SHAPE_KEY).and_then(|v| v.as_str()) {
                    return Ok(shape.to_string());
                }
            }
        }
    }
    if def.kind == FileKind::NestedMap {
        Ok(SHAPE_MAP_OBJECTS.to_string())
    } else if is_scalar_list_section(section_key) {
        Ok(SHAPE_ARRAY_SCALARS.to_string())
    } else {
        Ok(SHAPE_ARRAY_OBJECTS.to_string())
    }
}

fn row_to_record(id: i64, record_key: Option<String>, raw_json: &str) -> Result<Record, AppError> {
    let parsed: Json = serde_json::from_str(raw_json)
        .map_err(|e| AppError::Internal(format!("raw_json inválido na linha {id}: {e}")))?;
    let mut map = parsed.as_object().cloned().unwrap_or_default();
    map.insert("id".to_string(), Json::Number(id.into()));
    if let Some(rk) = record_key {
        map.insert("_record_key".to_string(), Json::String(rk));
    }
    Ok(map)
}

fn infer_columns_for_section(
    def: &EgressosFileDef,
    section_key: &str,
    items: &[Record],
) -> Vec<String> {
    if section_key == def.primary_section {
        return def.columns.iter().map(|c| (*c).to_string()).collect();
    }
    if let Some(first) = items.first() {
        let cols: Vec<String> = first
            .keys()
            .filter(|k| *k != "id" && *k != "_record_key")
            .take(8)
            .cloned()
            .collect();
        if !cols.is_empty() {
            return cols;
        }
    }
    vec!["_key".to_string(), "value".to_string()]
}

/// Lista registros de uma seção de um dos 21 arquivos com busca, ordenação e paginação (`FR-008`).
pub fn list_rows(
    conn: &Connection,
    file_key: &str,
    section_key: Option<&str>,
    limit: Option<i64>,
    offset: Option<i64>,
    search: Option<&str>,
    sort: Option<&str>,
    order: Option<&str>,
) -> Result<EgressosRowPage, AppError> {
    let def = by_file_key(file_key)?;
    let sec = section_key
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(def.primary_section);
    if !def.sections.contains(&sec) {
        return Err(AppError::Validation(format!(
            "Seção '{}' não existe no arquivo '{}'.",
            sec, def.file_name
        )));
    }

    let limit = limit.unwrap_or(50).clamp(1, 500);
    let offset = offset.unwrap_or(0).max(0);
    let q = search.map(str::trim).filter(|s| !s.is_empty());

    let mut stmt = conn.prepare(
        "SELECT id, record_key, search_text, raw_json FROM egressos_rows \
         WHERE file_key = ?1 AND section_key = ?2 \
         ORDER BY ord ASC, id ASC",
    )?;
    let all_rows: Vec<(i64, Option<String>, String, String)> = stmt
        .query_map(params![file_key, sec], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut filtered = Vec::new();
    for (id, rk, st, raw) in all_rows {
        if let Some(term) = q {
            let term_low = term.to_lowercase();
            let id_match = term.parse::<i64>().ok() == Some(id);
            let rk_match = rk
                .as_ref()
                .map(|k| k.to_lowercase().contains(&term_low))
                .unwrap_or(false);
            if !id_match && !rk_match && !st.contains(&term_low) {
                continue;
            }
        }
        filtered.push(row_to_record(id, rk, &raw)?);
    }

    if let Some(sort_col) = sort.map(str::trim).filter(|s| !s.is_empty()) {
        let desc = matches!(order.map(str::trim), Some(o) if o.eq_ignore_ascii_case("desc"));
        filtered.sort_by(|a, b| {
            let va = a.get(sort_col);
            let vb = b.get(sort_col);
            let cmp = compare_json_values(va, vb);
            if desc { cmp.reverse() } else { cmp }
        });
    }

    let total = filtered.len() as i64;
    let items: Vec<Record> = filtered
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect();

    let columns = infer_columns_for_section(def, sec, &items);
    let shape = section_shape_from_db(conn, def, sec)?;
    let is_map_section = shape == SHAPE_MAP_OBJECTS || shape == SHAPE_MAP_SCALARS;

    Ok(EgressosRowPage {
        items,
        total,
        limit,
        offset,
        file_key: file_key.to_string(),
        section_key: sec.to_string(),
        columns,
        entity_group: section_entity_group(file_key, sec).as_str().to_string(),
        is_map_section,
    })
}

fn compare_json_values(a: Option<&Json>, b: Option<&Json>) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    match (a, b) {
        (None | Some(Json::Null), None | Some(Json::Null)) => Ordering::Equal,
        (None | Some(Json::Null), _) => Ordering::Greater,
        (_, None | Some(Json::Null)) => Ordering::Less,
        (Some(Json::Number(na)), Some(Json::Number(nb))) => na
            .as_f64()
            .unwrap_or(0.0)
            .partial_cmp(&nb.as_f64().unwrap_or(0.0))
            .unwrap_or(Ordering::Equal),
        (Some(va), Some(vb)) => {
            let sa = va
                .as_str()
                .map(str::to_lowercase)
                .unwrap_or_else(|| va.to_string().to_lowercase());
            let sb = vb
                .as_str()
                .map(str::to_lowercase)
                .unwrap_or_else(|| vb.to_string().to_lowercase());
            sa.cmp(&sb)
        }
    }
}

/// Busca um registro específico pelo `id` interno do SQLite.
pub fn get_row(conn: &Connection, id: i64) -> Result<Record, AppError> {
    let row: Option<(Option<String>, String)> = conn
        .query_row(
            "SELECT record_key, raw_json FROM egressos_rows WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;

    let (rk, raw) = row.ok_or(AppError::NotFound)?;
    row_to_record(id, rk, &raw)
}

fn clean_payload_for_storage(payload: &Record) -> Record {
    payload
        .iter()
        .filter(|(k, _)| *k != "id" && *k != "_record_key")
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

/// Cria um novo registro em `(file_key, section_key)` (`FR-008`).
pub fn create_row(
    conn: &Connection,
    file_key: &str,
    section_key: Option<&str>,
    payload: &Record,
) -> Result<Record, AppError> {
    let def = by_file_key(file_key)?;
    let sec = section_key
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(def.primary_section);
    if !def.sections.contains(&sec) {
        return Err(AppError::Validation(format!(
            "Seção '{}' não existe no arquivo '{}'.",
            sec, def.file_name
        )));
    }

    let cleaned = clean_payload_for_storage(payload);
    if cleaned.is_empty() {
        return Err(AppError::Validation(
            "O registro não pode ser criado vazio.".into(),
        ));
    }

    let shape = section_shape_from_db(conn, def, sec)?;
    let is_map = shape == SHAPE_MAP_OBJECTS || shape == SHAPE_MAP_SCALARS;

    if is_map {
        let key_val = cleaned
            .get("_key")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                AppError::Validation(
                    "O campo '_key' (chave identificadora) é obrigatório nesta tabela.".into(),
                )
            })?;

        let dup: i64 = conn.query_row(
            "SELECT COUNT(*) FROM egressos_rows \
             WHERE file_key = ?1 AND section_key = ?2 AND record_key = ?3",
            params![file_key, sec, key_val],
            |r| r.get(0),
        )?;
        if dup > 0 {
            return Err(AppError::Conflict(format!(
                "Já existe um registro com a chave '{key_val}' em {}/{sec}.",
                def.file_name
            )));
        }
    }

    // Garante que o arquivo exista em egressos_files mesmo se criado antes de import
    conn.execute(
        "INSERT OR IGNORE INTO egressos_files (file_key, kind, primary_section, envelope_json) \
         VALUES (?1, ?2, ?3, '{}')",
        params![def.file_key, def.kind.as_str(), def.primary_section],
    )?;

    let next_ord: i64 = conn.query_row(
        "SELECT COALESCE(MAX(ord), -1) + 1 FROM egressos_rows WHERE file_key = ?1 AND section_key = ?2",
        params![file_key, sec],
        |r| r.get(0),
    )?;

    let rec_key = extract_record_key(file_key, sec, &cleaned);
    let search_text = build_search_text(&cleaned);
    let raw_json = Json::Object(cleaned).to_string();

    conn.execute(
        "INSERT INTO egressos_rows (file_key, section_key, ord, record_key, search_text, raw_json) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![file_key, sec, next_ord, rec_key, search_text, raw_json],
    )?;
    let id = conn.last_insert_rowid();
    get_row(conn, id)
}

/// Atualiza um registro existente e, se `propagate == true` (`FR-008a`), propaga
/// os campos equivalentes para os registros irmãos do mesmo grupo (`Alumni` ou `Company`).
pub fn update_row(
    conn: &mut Connection,
    id: i64,
    payload: &Record,
    propagate: bool,
) -> Result<EgressosMutationResult, AppError> {
    let existing: Option<(String, String, Option<String>, String)> = conn
        .query_row(
            "SELECT file_key, section_key, record_key, raw_json FROM egressos_rows WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?;

    let (file_key, section_key, old_record_key, old_raw) =
        existing.ok_or(AppError::NotFound)?;

    let def = by_file_key(&file_key)?;
    let mut current_obj: Map<String, Json> = serde_json::from_str(&old_raw)
        .map_err(|e| AppError::Internal(format!("raw_json inválido: {e}")))?;

    for (k, v) in clean_payload_for_storage(payload) {
        current_obj.insert(k, v);
    }

    let shape = section_shape_from_db(conn, def, &section_key)?;
    if shape == SHAPE_MAP_OBJECTS || shape == SHAPE_MAP_SCALARS {
        let key_val = current_obj
            .get("_key")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                AppError::Validation("O campo '_key' não pode ficar vazio.".into())
            })?;
        let dup: i64 = conn.query_row(
            "SELECT COUNT(*) FROM egressos_rows \
             WHERE file_key = ?1 AND section_key = ?2 AND record_key = ?3 AND id != ?4",
            params![file_key, section_key, key_val, id],
            |r| r.get(0),
        )?;
        if dup > 0 {
            return Err(AppError::Conflict(format!(
                "Já existe outro registro com a chave '{key_val}' em {}/{section_key}.",
                def.file_name
            )));
        }
    }

    let new_record_key = extract_record_key(&file_key, &section_key, &current_obj);
    let new_search = build_search_text(&current_obj);
    let new_raw = Json::Object(current_obj.clone()).to_string();

    let tx = conn.transaction()?;
    tx.execute(
        "UPDATE egressos_rows SET record_key = ?1, search_text = ?2, raw_json = ?3 WHERE id = ?4",
        params![new_record_key, new_search, new_raw, id],
    )?;

    let mut propagated_count = 0usize;
    if propagate {
        if let Some(ref lookup_key) = old_record_key.or_else(|| new_record_key.clone()) {
            let group = section_entity_group(&file_key, &section_key);
            propagated_count = propagate_update_in_tx(
                &tx,
                id,
                group,
                lookup_key,
                new_record_key.as_deref(),
                &current_obj,
            )?;
        }
    }

    tx.commit()?;
    let updated_record = get_row(conn, id)?;
    Ok(EgressosMutationResult {
        record: updated_record,
        propagated_count,
    })
}

fn propagate_update_in_tx(
    tx: &rusqlite::Transaction<'_>,
    source_id: i64,
    group: EntityGroup,
    old_key: &str,
    new_key: Option<&str>,
    source_obj: &Map<String, Json>,
) -> Result<usize, AppError> {
    if group == EntityGroup::None {
        return Ok(0);
    }

    let mut stmt = tx.prepare(
        "SELECT id, file_key, section_key, raw_json FROM egressos_rows \
         WHERE record_key = ?1 AND id != ?2",
    )?;
    let candidates: Vec<(i64, String, String, String)> = stmt
        .query_map(params![old_key, source_id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    drop(stmt);

    let mut count = 0usize;
    for (target_id, t_file, t_sec, t_raw) in candidates {
        if section_entity_group(&t_file, &t_sec) != group {
            continue;
        }
        let mut target_obj: Map<String, Json> = serde_json::from_str(&t_raw)
            .map_err(|e| AppError::Internal(format!("raw_json inválido: {e}")))?;
        let mut changed = false;

        match group {
            EntityGroup::Alumni => {
                if let Some(nk) = new_key {
                    if nk != old_key {
                        if target_obj.contains_key("label") {
                            target_obj.insert("label".into(), Json::String(nk.to_string()));
                            changed = true;
                        }
                        if target_obj.contains_key("rotulo") {
                            target_obj.insert("rotulo".into(), Json::String(nk.to_string()));
                            changed = true;
                        }
                        if target_obj.contains_key("perfil") {
                            target_obj
                                .insert("perfil".into(), Json::String(format!("Perfil {nk}")));
                            changed = true;
                        }
                    }
                }
                for same_key in ["nome", "monograma", "curso", "area_atual", "trilha", "em_tech", "anos"] {
                    if let Some(val) = source_obj.get(same_key) {
                        if target_obj.contains_key(same_key) && target_obj.get(same_key) != Some(val) {
                            target_obj.insert(same_key.to_string(), val.clone());
                            changed = true;
                        }
                    }
                }
                if let Some(cargo) = source_obj.get("cargo_atual").or_else(|| source_obj.get("cargo")) {
                    for k in ["cargo_atual", "cargo"] {
                        if target_obj.contains_key(k) && target_obj.get(k) != Some(cargo) {
                            target_obj.insert(k.to_string(), cargo.clone());
                            changed = true;
                        }
                    }
                }
                if let Some(emp) = source_obj
                    .get("empresa_atual")
                    .or_else(|| source_obj.get("empresa"))
                    .or_else(|| source_obj.get("empregador_atual"))
                {
                    for k in ["empresa_atual", "empresa", "empregador_atual"] {
                        if target_obj.contains_key(k) && target_obj.get(k) != Some(emp) {
                            target_obj.insert(k.to_string(), emp.clone());
                            changed = true;
                        }
                    }
                }
                if let Some(loc) = source_obj.get("local_atual").or_else(|| source_obj.get("lugar")) {
                    for k in ["local_atual", "lugar"] {
                        if target_obj.contains_key(k) && target_obj.get(k) != Some(loc) {
                            target_obj.insert(k.to_string(), loc.clone());
                            changed = true;
                        }
                    }
                }
            }
            EntityGroup::Company => {
                if let Some(nk) = new_key {
                    if nk != old_key {
                        for k in ["_key", "empresa", "nome"] {
                            if target_obj.contains_key(k) {
                                target_obj.insert(k.to_string(), Json::String(nk.to_string()));
                                changed = true;
                            }
                        }
                    }
                }
                for same_key in ["porte", "origem", "setor", "nome_oficial", "url", "funcionarios"] {
                    if let Some(val) = source_obj.get(same_key) {
                        if target_obj.contains_key(same_key) && target_obj.get(same_key) != Some(val) {
                            target_obj.insert(same_key.to_string(), val.clone());
                            changed = true;
                        }
                    }
                }
            }
            EntityGroup::None => {}
        }

        if changed {
            let new_rk = extract_record_key(&t_file, &t_sec, &target_obj);
            let new_st = build_search_text(&target_obj);
            let new_raw = Json::Object(target_obj).to_string();
            tx.execute(
                "UPDATE egressos_rows SET record_key = ?1, search_text = ?2, raw_json = ?3 WHERE id = ?4",
                params![new_rk, new_st, new_raw, target_id],
            )?;
            count += 1;
        }
    }

    Ok(count)
}

/// Remove um registro pelo `id` e, se `propagate == true` (`FR-008a`), remove também
/// os registros irmãos do mesmo grupo (`Alumni` ou `Company`).
pub fn delete_row(
    conn: &mut Connection,
    id: i64,
    propagate: bool,
) -> Result<EgressosDeleteResult, AppError> {
    let existing: Option<(String, String, Option<String>)> = conn
        .query_row(
            "SELECT file_key, section_key, record_key FROM egressos_rows WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;

    let (file_key, section_key, record_key) = existing.ok_or(AppError::NotFound)?;

    let tx = conn.transaction()?;
    let deleted = tx.execute("DELETE FROM egressos_rows WHERE id = ?1", [id])?;

    let mut propagated_deleted = 0usize;
    if propagate {
        if let Some(ref rk) = record_key {
            let group = section_entity_group(&file_key, &section_key);
            if group != EntityGroup::None {
                let mut stmt = tx.prepare(
                    "SELECT id, file_key, section_key FROM egressos_rows WHERE record_key = ?1",
                )?;
                let siblings: Vec<(i64, String, String)> = stmt
                    .query_map([rk], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
                    .collect::<Result<Vec<_>, _>>()?;
                drop(stmt);

                for (sib_id, s_file, s_sec) in siblings {
                    if section_entity_group(&s_file, &s_sec) == group {
                        tx.execute("DELETE FROM egressos_rows WHERE id = ?1", [sib_id])?;
                        propagated_deleted += 1;
                    }
                }
            }
        }
    }

    tx.commit()?;
    Ok(EgressosDeleteResult {
        deleted,
        propagated_deleted,
    })
}

/// Retorna os metadados de nível de arquivo (`envelope`) ocultando os marcadores internos de seção (`FR-009`).
pub fn get_envelope(conn: &Connection, file_key: &str) -> Result<Record, AppError> {
    let def = by_file_key(file_key)?;
    let env_str: Option<String> = conn
        .query_row(
            "SELECT envelope_json FROM egressos_files WHERE file_key = ?1",
            [def.file_key],
            |r| r.get(0),
        )
        .optional()?;

    let env_str = env_str.unwrap_or_else(|| "{}".to_string());
    let env: Json = serde_json::from_str(&env_str)
        .map_err(|e| AppError::Internal(format!("envelope_json inválido: {e}")))?;
    let obj = env.as_object().cloned().unwrap_or_default();

    let visible: Record = obj
        .into_iter()
        .filter(|(_, v)| {
            !v.as_object()
                .map(|o| o.contains_key(SENTINEL_SECTION_KEY))
                .unwrap_or(false)
        })
        .collect();
    Ok(visible)
}

/// Atualiza os metadados de nível de arquivo (`envelope`) preservando intactos os marcadores de seção (`FR-009`).
pub fn update_envelope(
    conn: &Connection,
    file_key: &str,
    payload: &Record,
) -> Result<Record, AppError> {
    let def = by_file_key(file_key)?;
    let env_str: Option<String> = conn
        .query_row(
            "SELECT envelope_json FROM egressos_files WHERE file_key = ?1",
            [def.file_key],
            |r| r.get(0),
        )
        .optional()?;

    let env_str = env_str.ok_or_else(|| {
        AppError::Validation(format!(
            "O arquivo '{}' ainda não foi importado.",
            def.file_name
        ))
    })?;
    let env: Json = serde_json::from_str(&env_str)
        .map_err(|e| AppError::Internal(format!("envelope_json inválido: {e}")))?;
    let mut current = env.as_object().cloned().unwrap_or_default();

    for (k, v) in payload {
        if def.sections.contains(&k.as_str()) {
            continue;
        }
        current.insert(k.clone(), v.clone());
    }

    conn.execute(
        "UPDATE egressos_files SET envelope_json = ?1 WHERE file_key = ?2",
        params![Json::Object(current).to_string(), def.file_key],
    )?;
    get_envelope(conn, file_key)
}

/// Gera um diretório `dados/` completo e autocontido com os 21 arquivos JSON
/// canônicos do projeto Egressos para uso em testes unitários e de integração
/// (inclusive em runners de CI onde `/home/thiagodeps/egressos/dados` não existe).
pub fn write_sample_dados_dir(dest: &Path) {
    use serde_json::json;
    std::fs::create_dir_all(dest).expect("criar diretório de fixture dados/");

    for def in EGRESSOS_FILES {
        let val: Json = match def.file_key {
            "egressos_perfil" => {
                let mut egressos = Vec::with_capacity(50);
                for i in 0..50 {
                    let label = if i == 0 {
                        "A".to_string()
                    } else {
                        format!("E{:02}", i)
                    };
                    egressos.push(json!({
                        "label": label,
                        "nome": format!("Egresso {label}"),
                        "curso": if i % 2 == 0 { "bsi" } else { "eca" },
                        "cargo_atual": "Engenheiro de Software",
                        "empresa_atual": "IFES",
                        "local_atual": "Vitória, ES",
                        "area_atual": "Desenvolvimento"
                    }));
                }
                json!({
                    "titulo": "Perfis de Egressos",
                    "atualizado_em": "2026-10-01",
                    "egressos": egressos,
                    "cursos": [
                        { "sigla": "bsi", "nome": "Sistemas de Informação" },
                        { "sigla": "eca", "nome": "Engenharia de Controle e Automação" }
                    ]
                })
            }
            "vitrine" => json!({
                "titulo": "Vitrine de Egressos",
                "perfis": [
                    {
                        "rotulo": "A",
                        "nome": "Egresso A",
                        "curso": "bsi",
                        "cargo": "Engenheiro de Software",
                        "empresa": "IFES",
                        "lugar": "Vitória, ES"
                    }
                ],
                "empresas": [{ "empresa": "Akamai Technologies", "porte": "Grande", "setor": "Cloud" }],
                "empresas_verificadas": [{ "empresa": "Akamai Technologies", "porte": "Grande" }],
                "contratando": [{ "empresa": "AEVO", "vagas": 3 }],
                "porte": [{ "porte": "Grande", "n": 10 }],
                "setor": [{ "setor": "Tecnologia", "n": 25 }],
                "regiao_das_empresas": [{ "regiao": "Sudeste", "n": 30 }],
                "paises_no_exterior": ["Estados Unidos", "Portugal"],
                "trilhas": [{ "trilha": "Engenharia de Software", "n": 20 }],
                "regioes": [{ "regiao": "Grande Vitória", "n": 35 }]
            }),
            "panorama" => json!({
                "titulo": "Panorama do Coorte",
                "coorte": [
                    {
                        "rotulo": "A",
                        "cargo_atual": "Engenheiro de Software",
                        "empregador_atual": "IFES",
                        "origem": "Nacional",
                        "modalidade": "Remoto",
                        "area_atual_nome": "Desenvolvimento",
                        "inicio_carreira": 2020
                    }
                ],
                "salto": [{ "rotulo": "A", "de": "Estágio", "para": "Pleno" }],
                "areas": [{ "area": "Desenvolvimento", "n": 30 }],
                "familias": [{ "familia": "Backend", "n": 15 }],
                "origem_comum": [{ "origem": "IFES Serra", "n": 50 }]
            }),
            "analise" => json!({
                "titulo": "Análise do Grupo",
                "rows": [
                    {
                        "label": "A",
                        "curso": "bsi",
                        "anos": 4,
                        "n_empresas": 2,
                        "n_exp": 3,
                        "trilha": "Tech",
                        "em_tech": true,
                        "exterior": false
                    }
                ],
                "top_tech": [{ "tech": "Rust", "n": 12 }],
                "funcoes": [{ "funcao": "Backend", "n": 18 }],
                "metodos": [{ "metodo": "Scrum", "n": 25 }],
                "intl_timeline": [{ "ano": 2025, "n": 5 }],
                "trajetoria_destaque": [{ "label": "A", "resumo": "Destaque" }],
                "hist_multiplicadores": [{ "faixa": "2x-4x", "n": 20 }]
            }),
            "consolidado" => json!({
                "titulo": "Consolidado Salarial",
                "perfis": [
                    {
                        "perfil": "A",
                        "trilha": "Tech",
                        "em_tech": true,
                        "anos": 4,
                        "bolsa": true,
                        "med_ini": 3500.0,
                        "med_atual": 12000.0,
                        "cresc": 3.43
                    }
                ],
                "agregado": [{ "metrica": "mediana", "valor": 10500.0 }],
                "pv": [{ "grupo": "Com IC", "med": 11000.0 }],
                "por_ano_sm": [{ "ano": 2025, "sm": 7.5 }],
                "series_por_perfil": [{ "perfil": "A", "ano": 2025, "valor": 12000.0 }]
            }),
            "empresas_porte" => json!({
                "AEVO": {
                    "empresa": "AEVO",
                    "porte": "Média",
                    "origem": "Nacional",
                    "setor": "SaaS",
                    "funcionarios": "51-200"
                },
                "Akamai Technologies": {
                    "empresa": "Akamai Technologies",
                    "porte": "Grande",
                    "origem": "Internacional",
                    "setor": "Cloud",
                    "funcionarios": "10000+"
                }
            }),
            "empresas_aliases" => json!({
                "AEVO": {
                    "aliases": ["Aevo Inovação"],
                    "atual": true,
                    "n_egressos_atual": 2
                },
                "Akamai Technologies": {
                    "aliases": ["Akamai"],
                    "atual": true,
                    "n_egressos_atual": 1
                }
            }),
            "empresas_linkedin_data" => json!({
                "Akamai Technologies": {
                    "nome_oficial": "Akamai Technologies",
                    "industry_linkedin": "Technology, Information and Internet",
                    "headcount_linkedin": "10001+",
                    "hq_local": "Cambridge, MA",
                    "website": "https://www.akamai.com",
                    "founded": 1998
                }
            }),
            "empresas_linkedin_urls" => json!({
                "Akamai Technologies": {
                    "url": "https://www.linkedin.com/company/akamai-technologies",
                    "slug": "akamai-technologies",
                    "confianca": "alta",
                    "via": "manual"
                }
            }),
            "codigofonte_historico" => json!({
                "fonte": "Código Fonte TV",
                "media_por_senioridade": {
                    "2025": {
                        "Estágio": 2100.0,
                        "Júnior": 4500.0,
                        "Pleno": 8500.0,
                        "Sênior": 14500.0,
                        "Especialista/Tech Lead": 19500.0
                    }
                },
                "ipca_fator_para_2026": {
                    "2024": 1.09,
                    "2025": 1.04
                }
            }),
            "ibge_series" => json!({
                "fonte_geral": "IBGE",
                "series": {
                    "ipca": {
                        "fonte": "IBGE/SIDRA",
                        "unidade": "% a.a."
                    }
                }
            }),
            "recorte_cursos" => json!({
                "instituicao": "IFES Campus Serra",
                "cursos": {
                    "bsi": {
                        "nome": "Sistemas de Informação",
                        "sigla": "BSI",
                        "n": 30
                    },
                    "eca": {
                        "nome": "Engenharia de Controle e Automação",
                        "sigla": "ECA",
                        "n": 20
                    }
                }
            }),
            "mapa_mundi" => json!({
                "projection": "equirectangular",
                "paths": ["M0 0 L10 10 Z", "M20 20 L30 30 Z"],
                "viewBox": [0, 0, 1000, 500],
                "lat_range": [-60, 85]
            }),
            _ => {
                let mut root = Map::new();
                root.insert("titulo".into(), json!(def.title));
                for &sec in def.sections {
                    let mut row = Map::new();
                    for (idx, &col) in def.columns.iter().enumerate() {
                        if col != "_key" {
                            row.insert(col.to_string(), json!(format!("{}_{}", col, idx + 1)));
                        }
                    }
                    root.insert(sec.to_string(), json!([Json::Object(row)]));
                }
                Json::Object(root)
            }
        };

        let text = serde_json::to_string_pretty(&val).expect("serializar fixture JSON");
        std::fs::write(dest.join(def.file_name), text).expect("gravar arquivo de fixture JSON");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use serde_json::json;
    use std::collections::HashSet;

    fn copy_reference_dados_to(dest: &Path) {
        write_sample_dados_dir(dest);
    }

    /// SEP-035 T005: o catálogo declara exatamente os 21 arquivos de `dados/`
    /// com chaves únicas, seções primárias válidas e resolução por `by_file_key`.
    #[test]
    fn catalog_covers_all_twenty_one_canonical_files() {
        assert_eq!(EGRESSOS_FILES.len(), 21);
        let mut keys = HashSet::new();
        let mut names = HashSet::new();
        for def in EGRESSOS_FILES {
            assert!(keys.insert(def.file_key), "file_key duplicado: {}", def.file_key);
            assert!(
                names.insert(def.file_name),
                "file_name duplicado: {}",
                def.file_name
            );
            assert_eq!(format!("{}.json", def.file_key), def.file_name);
            assert!(
                def.sections.contains(&def.primary_section),
                "{}: primary_section '{}' deve estar em sections",
                def.file_key,
                def.primary_section
            );
            assert!(!def.columns.is_empty());
            assert_eq!(by_file_key(def.file_key).unwrap().file_name, def.file_name);
        }
        assert!(matches!(
            by_file_key("inexistente"),
            Err(AppError::UnknownEntity(_))
        ));
    }

    /// SEP-035 T012: validação atômica na importação, ignorando arquivos extras
    /// e criando snapshot `egressos-pre-import-*` ao substituir base existente.
    #[test]
    fn import_validates_atomically_ignores_extras_and_snapshots_on_reimport() {
        let mut conn = db::open_in_memory().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let dados_dir = tmp.path().join("dados");
        let app_data = tmp.path().join("app_data");
        copy_reference_dados_to(&dados_dir);

        // Arquivo extra estranho deve ser ignorado (C2)
        std::fs::write(dados_dir.join(".DS_Store"), b"ignored").unwrap();

        let summary1 = import_dir(&mut conn, &dados_dir, &app_data, |_| {}).unwrap();
        assert_eq!(summary1.total_files, 21);
        assert!(summary1.total_rows >= 50);
        assert!(!summary1.replaced);
        assert!(summary1.snapshot.is_none());

        // Segundo import sobre base existente deve criar snapshot egressos-pre-import-*
        let summary2 = import_dir(&mut conn, &dados_dir, &app_data, |_| {}).unwrap();
        assert!(summary2.replaced);
        let snap_name = summary2.snapshot.expect("snapshot esperado no reimport");
        assert!(snap_name.starts_with("egressos-pre-import-"));
        assert!(app_data.join("snapshots").join(&snap_name).exists());

        // Se faltar 1 arquivo obrigatório, aborta e mantém a base intacta (FR-005)
        let broken_dir = tmp.path().join("broken");
        copy_reference_dados_to(&broken_dir);
        std::fs::remove_file(broken_dir.join("vitrine.json")).unwrap();
        let err = import_dir(&mut conn, &broken_dir, &app_data, |_| {}).unwrap_err();
        assert!(matches!(err, AppError::Validation(ref msg) if msg.contains("vitrine.json")));

        let after_failed: i64 = conn
            .query_row("SELECT COUNT(*) FROM egressos_rows", [], |r| r.get(0))
            .unwrap();
        assert_eq!(after_failed as usize, summary2.total_rows);
    }

    /// SEP-035 T013 / SC-003: importar os 21 arquivos de `dados/`
    /// e exportar sem edições produz 21 arquivos JSON 100% equivalentes campo a campo.
    #[test]
    fn round_trip_of_all_twenty_one_real_files_is_lossless() {
        let mut conn = db::open_in_memory().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let app_data = tmp.path().join("app_data");
        let export_out = tmp.path().join("exported");
        let ref_dir = tmp.path().join("ref_dados");
        copy_reference_dados_to(&ref_dir);

        import_dir(&mut conn, &ref_dir, &app_data, |_| {}).unwrap();
        let exp_summary = export_dir(&conn, &export_out).unwrap();
        assert_eq!(exp_summary.total_files, 21);

        for def in EGRESSOS_FILES {
            let orig_text = std::fs::read_to_string(ref_dir.join(def.file_name)).unwrap();
            let exp_text = std::fs::read_to_string(export_out.join(def.file_name)).unwrap();
            let orig_val: Json = serde_json::from_str(&orig_text).unwrap();
            let exp_val: Json = serde_json::from_str(&exp_text).unwrap();
            assert_eq!(
                orig_val, exp_val,
                "divergência no round-trip do arquivo {}",
                def.file_name
            );
        }
    }

    /// SEP-035 T021: CRUD completo (list, search, sort, create, get, update, delete)
    /// e edição do envelope de metadados, incluindo unicidade de `_key` em mapas.
    #[test]
    fn crud_and_envelope_work_across_section_types() {
        let mut conn = db::open_in_memory().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let dados_dir = tmp.path().join("dados");
        copy_reference_dados_to(&dados_dir);
        import_dir(
            &mut conn,
            &dados_dir,
            tmp.path(),
            |_| {},
        )
        .unwrap();

        // 1. List + Search + Sort em egressos_perfil
        let page = list_rows(
            &conn,
            "egressos_perfil",
            None,
            Some(10),
            Some(0),
            None,
            Some("label"),
            Some("asc"),
        )
        .unwrap();
        assert_eq!(page.total, 50);
        assert_eq!(page.items.len(), 10);

        // 2. Create + Get + Update + Delete em egressos_perfil (propagate = false)
        let mut new_rec = Record::new();
        new_rec.insert("label".into(), json!("AZ"));
        new_rec.insert("nome".into(), json!("Fulano de Teste"));
        new_rec.insert("curso".into(), json!("bsi"));
        new_rec.insert("cargo_atual".into(), json!("Tech Lead"));
        new_rec.insert("empresa_atual".into(), json!("IFES"));
        let created = create_row(&conn, "egressos_perfil", None, &new_rec).unwrap();
        let created_id = created.get("id").and_then(|v| v.as_i64()).unwrap();

        let mut patch = Record::new();
        patch.insert("cargo_atual".into(), json!("Principal Engineer"));
        let updated = update_row(&mut conn, created_id, &patch, false).unwrap();
        assert_eq!(updated.propagated_count, 0);
        assert_eq!(
            updated.record.get("cargo_atual"),
            Some(&json!("Principal Engineer"))
        );

        let del_res = delete_row(&mut conn, created_id, false).unwrap();
        assert_eq!(del_res.deleted, 1);
        assert!(matches!(get_row(&conn, created_id), Err(AppError::NotFound)));

        // 3. Unicidade de `_key` em tabela baseada em mapa (`empresas_porte`)
        let mut dup_company = Record::new();
        dup_company.insert("_key".into(), json!("AEVO"));
        dup_company.insert("porte".into(), json!("Grande"));
        let err = create_row(&conn, "empresas_porte", None, &dup_company).unwrap_err();
        assert!(matches!(err, AppError::Conflict(_)));

        // 4. Edição de envelope de metadados (FR-009)
        let mut env_patch = Record::new();
        env_patch.insert("titulo".into(), json!("Título Customizado no Teste"));
        let updated_env = update_envelope(&conn, "egressos_perfil", &env_patch).unwrap();
        assert_eq!(
            updated_env.get("titulo"),
            Some(&json!("Título Customizado no Teste"))
        );
        let reconstructed = reconstruct_file_json(&conn, "egressos_perfil").unwrap();
        assert_eq!(
            reconstructed.get("titulo"),
            Some(&json!("Título Customizado no Teste"))
        );
        assert_eq!(
            reconstructed
                .get("egressos")
                .and_then(|v| v.as_array())
                .map(|a| a.len()),
            Some(50)
        );
    }

    /// SEP-035 T022 / FR-008a: propagação opcional (`propagate = true`) sincroniza
    /// alterações de Egresso (`Alumni`) e de Empresa (`Company`) nas tabelas irmãs.
    #[test]
    fn optional_propagation_updates_and_deletes_across_related_tables() {
        let mut conn = db::open_in_memory().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let dados_dir = tmp.path().join("dados");
        copy_reference_dados_to(&dados_dir);
        import_dir(
            &mut conn,
            &dados_dir,
            tmp.path(),
            |_| {},
        )
        .unwrap();

        // Localiza egresso "A" em egressos_perfil
        let page_a = list_rows(
            &conn,
            "egressos_perfil",
            Some("egressos"),
            Some(50),
            Some(0),
            Some("A"),
            None,
            None,
        )
        .unwrap();
        let row_a = page_a
            .items
            .iter()
            .find(|r| r.get("label") == Some(&json!("A")))
            .unwrap();
        let id_a = row_a.get("id").and_then(|v| v.as_i64()).unwrap();

        let mut patch_a = Record::new();
        patch_a.insert("cargo_atual".into(), json!("Diretor Global de IA"));
        patch_a.insert("empresa_atual".into(), json!("OpenLab Internacional"));
        let res_a = update_row(&mut conn, id_a, &patch_a, true).unwrap();
        assert!(
            res_a.propagated_count >= 2,
            "deve propagar para vitrine:perfis e panorama:coorte"
        );

        let vitrine_json = reconstruct_file_json(&conn, "vitrine").unwrap();
        let vitrine_a = vitrine_json["perfis"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p.get("rotulo") == Some(&json!("A")))
            .unwrap();
        assert_eq!(vitrine_a.get("cargo"), Some(&json!("Diretor Global de IA")));
        assert_eq!(
            vitrine_a.get("empresa"),
            Some(&json!("OpenLab Internacional"))
        );

        let panorama_json = reconstruct_file_json(&conn, "panorama").unwrap();
        let panorama_a = panorama_json["coorte"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p.get("rotulo") == Some(&json!("A")))
            .unwrap();
        assert_eq!(
            panorama_a.get("cargo_atual"),
            Some(&json!("Diretor Global de IA"))
        );
        assert_eq!(
            panorama_a.get("empregador_atual"),
            Some(&json!("OpenLab Internacional"))
        );

        // Testa propagação de Empresa ("Akamai Technologies")
        let page_emp = list_rows(
            &conn,
            "empresas_porte",
            Some("_root"),
            Some(10),
            Some(0),
            Some("Akamai Technologies"),
            None,
            None,
        )
        .unwrap();
        let row_emp = page_emp
            .items
            .iter()
            .find(|r| r.get("_key") == Some(&json!("Akamai Technologies")))
            .unwrap();
        let id_emp = row_emp.get("id").and_then(|v| v.as_i64()).unwrap();

        let mut patch_emp = Record::new();
        patch_emp.insert("porte".into(), json!("Big Tech Global"));
        let res_emp = update_row(&mut conn, id_emp, &patch_emp, true).unwrap();
        assert!(res_emp.propagated_count >= 1);
    }
}
