//! Domínio Egressos-Dados (SEP-037): catálogo dos 39 datasets JSON do repositório
//! privado `ifesserra-lab/egressos-dados` (`alunos.json` na raiz + 38 arquivos em `data/*.json`),
//! importação e exportação sem perdas (*round-trip*) nos 4 formatos estruturais
//! (`object_sections`, `root_array`, `root_map`, `nested_map`), CRUD completo por aba/seção
//! com preservação do `"id"` textual de domínio via `"_row_id"` e propagação opcional.

use crate::crud::Record;
use crate::error::AppError;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value as Json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Caminho local sugerido quando existente na máquina do curador.
pub const DEFAULT_LOCAL_REPO_DIR: &str = "/home/thiagodeps/egressos-dados";

const SENTINEL_SECTION_KEY: &str = "__egressos_dados_section__";
const SENTINEL_SHAPE_KEY: &str = "__section_shape__";
const ROOT_SHAPE_KEY: &str = "__root_shape__";

const SHAPE_ARRAY_OBJECTS: &str = "array_objects";
const SHAPE_ARRAY_SCALARS: &str = "array_scalars";
const SHAPE_MAP_OBJECTS: &str = "map_objects";
const SHAPE_MAP_SCALARS: &str = "map_scalars";

/// Tipo estrutural do arquivo JSON raiz (`data-model.md` §2.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileKind {
    ObjectSections,
    RootArray,
    RootMap,
    NestedMap,
}

impl FileKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ObjectSections => "object_sections",
            Self::RootArray => "root_array",
            Self::RootMap => "root_map",
            Self::NestedMap => "nested_map",
        }
    }
}

/// Classificação de privacidade/publicação segundo `egressos_core.dados.CATALOGO`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicationKind {
    Pii,
    Interno,
    Publicavel,
}

impl PublicationKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pii => "pii",
            Self::Interno => "interno",
            Self::Publicavel => "publicavel",
        }
    }
}

/// Papel do dataset segundo `egressos_core.dados.CATALOGO`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetRole {
    Entrada,
    Saida,
}

impl DatasetRole {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Entrada => "entrada",
            Self::Saida => "saida",
        }
    }
}

/// Grupo de entidade para propagação cruzada opcional (`FR-007`).
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

/// Definição estática de cada um dos 39 arquivos JSON de `egressos-dados`.
#[derive(Clone, Copy, Debug)]
pub struct EgressosDadosFileDef {
    pub file_key: &'static str,
    pub file_name: &'static str,
    pub relative_path: &'static str,
    pub title: &'static str,
    pub kind: FileKind,
    pub publication: PublicationKind,
    pub role: DatasetRole,
    pub primary_section: &'static str,
    pub sections: &'static [&'static str],
    pub columns: &'static [&'static str],
    pub entity_group: EntityGroup,
}

/// Catálogo canônico 1:1 dos 39 datasets JSON de `/home/thiagodeps/egressos-dados` (`FR-003`).
pub const EGRESSOS_DADOS_FILES: &[EgressosDadosFileDef; 39] = &[
    // ---- 1..12: Datasets PII (1 na raiz + 11 em data/) ----
    EgressosDadosFileDef {
        file_key: "alunos",
        file_name: "alunos.json",
        relative_path: "alunos.json",
        title: "Cadastro de Alunos (Coorte)",
        kind: FileKind::ObjectSections,
        publication: PublicationKind::Pii,
        role: DatasetRole::Entrada,
        primary_section: "alunos",
        sections: &["alunos", "cursos", "convencoes"],
        columns: &[
            "id",
            "nome",
            "curso",
            "cargo_atual",
            "empresa_atual",
            "local_atual",
            "area_atual",
            "inicio_carreira_dev",
            "ainda_em_tech",
        ],
        entity_group: EntityGroup::Alumni,
    },
    EgressosDadosFileDef {
        file_key: "perfis_linkedin",
        file_name: "perfis_linkedin.json",
        relative_path: "data/perfis_linkedin.json",
        title: "Perfis LinkedIn (Coleta SIGA)",
        kind: FileKind::ObjectSections,
        publication: PublicationKind::Pii,
        role: DatasetRole::Saida,
        primary_section: "perfis",
        sections: &["perfis"],
        columns: &[
            "nome",
            "curso",
            "matricula",
            "achado",
            "confirmado",
            "cargo_atual",
            "empresa_atual",
            "slug",
        ],
        entity_group: EntityGroup::None,
    },
    EgressosDadosFileDef {
        file_key: "perfis_conferidos",
        file_name: "perfis_conferidos.json",
        relative_path: "data/perfis_conferidos.json",
        title: "Perfis Conferidos Manualmente",
        kind: FileKind::ObjectSections,
        publication: PublicationKind::Pii,
        role: DatasetRole::Entrada,
        primary_section: "conferidos",
        sections: &["conferidos"],
        columns: &["nome", "curso", "confirmado", "slug", "nota"],
        entity_group: EntityGroup::None,
    },
    EgressosDadosFileDef {
        file_key: "formandos_a_buscar",
        file_name: "formandos_a_buscar.json",
        relative_path: "data/formandos_a_buscar.json",
        title: "Fila de Formandos SIGA",
        kind: FileKind::ObjectSections,
        publication: PublicationKind::Pii,
        role: DatasetRole::Saida,
        primary_section: "fila",
        sections: &["fila"],
        columns: &["nome", "curso", "matricula", "semestres"],
        entity_group: EntityGroup::None,
    },
    EgressosDadosFileDef {
        file_key: "_novos_reais",
        file_name: "_novos_reais.json",
        relative_path: "data/_novos_reais.json",
        title: "Coorte Nova (Nomes Reais)",
        kind: FileKind::RootArray,
        publication: PublicationKind::Pii,
        role: DatasetRole::Entrada,
        primary_section: "_root",
        sections: &["_root"],
        columns: &[
            "id",
            "nome",
            "cargo_atual",
            "empresa_atual",
            "local_atual",
            "area_atual",
            "inicio_carreira_dev",
            "ainda_em_tech",
        ],
        entity_group: EntityGroup::Alumni,
    },
    EgressosDadosFileDef {
        file_key: "genero_map",
        file_name: "genero_map.json",
        relative_path: "data/genero_map.json",
        title: "Mapa de Gênero por Aluno",
        kind: FileKind::RootMap,
        publication: PublicationKind::Pii,
        role: DatasetRole::Saida,
        primary_section: "_root",
        sections: &["_root"],
        columns: &["_key", "value"],
        entity_group: EntityGroup::Alumni,
    },
    EgressosDadosFileDef {
        file_key: "horizon_pesquisa",
        file_name: "horizon_pesquisa.json",
        relative_path: "data/horizon_pesquisa.json",
        title: "Cruzamento Horizon Pesquisa",
        kind: FileKind::ObjectSections,
        publication: PublicationKind::Pii,
        role: DatasetRole::Entrada,
        primary_section: "encontrados",
        sections: &[
            "encontrados",
            "nao_encontrados",
            "sem_vinculo_pesquisa",
            "papeis_agregado",
            "tabelas_usadas",
        ],
        columns: &[
            "id",
            "papeis",
            "n_projetos",
            "tipos",
            "bolsa_ic",
        ],
        entity_group: EntityGroup::Alumni,
    },
    EgressosDadosFileDef {
        file_key: "lattes_orientacoes",
        file_name: "lattes_orientacoes.json",
        relative_path: "data/lattes_orientacoes.json",
        title: "Cruzamento Lattes Orientações",
        kind: FileKind::ObjectSections,
        publication: PublicationKind::Pii,
        role: DatasetRole::Entrada,
        primary_section: "encontrados",
        sections: &["encontrados", "tipos_agregado"],
        columns: &["id", "tipos", "n_ocorrencias"],
        entity_group: EntityGroup::Alumni,
    },
    EgressosDadosFileDef {
        file_key: "outros_labs",
        file_name: "outros_labs.json",
        relative_path: "data/outros_labs.json",
        title: "Outros Laboratórios (além do LEDS)",
        kind: FileKind::ObjectSections,
        publication: PublicationKind::Pii,
        role: DatasetRole::Entrada,
        primary_section: "egressos",
        sections: &["egressos", "labs", "detalhe_fonte"],
        columns: &["id", "labs", "bolsista"],
        entity_group: EntityGroup::Alumni,
    },
    EgressosDadosFileDef {
        file_key: "src_extensao",
        file_name: "src_extensao.json",
        relative_path: "data/src_extensao.json",
        title: "Cruzamento SRC Extensão",
        kind: FileKind::ObjectSections,
        publication: PublicationKind::Pii,
        role: DatasetRole::Saida,
        primary_section: "fapes_pesquisa",
        sections: &[
            "fapes_pesquisa",
            "bolsa_documentada_oficial",
            "genero",
            "encontrados",
            "bolsistas_extensao",
            "funcoes",
            "projetos_por_egresso",
        ],
        columns: &["value"],
        entity_group: EntityGroup::None,
    },
    EgressosDadosFileDef {
        file_key: "por_ano",
        file_name: "por_ano.json",
        relative_path: "data/por_ano.json",
        title: "Histórico Individual por Ano",
        kind: FileKind::RootArray,
        publication: PublicationKind::Pii,
        role: DatasetRole::Entrada,
        primary_section: "_root",
        sections: &["_root"],
        columns: &[
            "ano",
            "empresa",
            "exp",
            "survey",
            "fx",
            "p25",
            "med",
            "p75",
        ],
        entity_group: EntityGroup::None,
    },
    EgressosDadosFileDef {
        file_key: "consolidado_raw",
        file_name: "consolidado_raw.json",
        relative_path: "data/consolidado_raw.json",
        title: "Consolidado Bruto (Pré-Anonimização)",
        kind: FileKind::RootArray,
        publication: PublicationKind::Pii,
        role: DatasetRole::Entrada,
        primary_section: "_root",
        sections: &["_root"],
        columns: &["id", "track", "em_tech", "serie"],
        entity_group: EntityGroup::Alumni,
    },
    // ---- 13..18: Datasets Internos (em data/) ----
    EgressosDadosFileDef {
        file_key: "_empresas_lista",
        file_name: "_empresas_lista.json",
        relative_path: "data/_empresas_lista.json",
        title: "Lista de Empresas (Insumo)",
        kind: FileKind::RootArray,
        publication: PublicationKind::Interno,
        role: DatasetRole::Saida,
        primary_section: "_root",
        sections: &["_root"],
        columns: &["value"],
        entity_group: EntityGroup::None,
    },
    EgressosDadosFileDef {
        file_key: "_novos_anon5",
        file_name: "_novos_anon5.json",
        relative_path: "data/_novos_anon5.json",
        title: "Coorte Nova Anonimizada (5 campos)",
        kind: FileKind::RootArray,
        publication: PublicationKind::Interno,
        role: DatasetRole::Entrada,
        primary_section: "_root",
        sections: &["_root"],
        columns: &[
            "id",
            "nome",
            "cargo_atual",
            "empresa_atual",
            "local_atual",
            "area_atual",
            "inicio_carreira_dev",
        ],
        entity_group: EntityGroup::None,
    },
    EgressosDadosFileDef {
        file_key: "_novos_anon6",
        file_name: "_novos_anon6.json",
        relative_path: "data/_novos_anon6.json",
        title: "Coorte Nova Anonimizada (6 campos)",
        kind: FileKind::RootArray,
        publication: PublicationKind::Interno,
        role: DatasetRole::Entrada,
        primary_section: "_root",
        sections: &["_root"],
        columns: &[
            "id",
            "nome",
            "cargo_atual",
            "empresa_atual",
            "local_atual",
            "area_atual",
            "inicio_carreira_dev",
        ],
        entity_group: EntityGroup::None,
    },
    EgressosDadosFileDef {
        file_key: "agg_pv",
        file_name: "agg_pv.json",
        relative_path: "data/agg_pv.json",
        title: "Agregado de Valor Presente",
        kind: FileKind::RootArray,
        publication: PublicationKind::Interno,
        role: DatasetRole::Entrada,
        primary_section: "_root",
        sections: &["_root"],
        columns: &["exp", "med", "n"],
        entity_group: EntityGroup::None,
    },
    EgressosDadosFileDef {
        file_key: "fontes_estado",
        file_name: "fontes_estado.json",
        relative_path: "data/fontes_estado.json",
        title: "Estado do Vigia de Fontes",
        kind: FileKind::NestedMap,
        publication: PublicationKind::Interno,
        role: DatasetRole::Saida,
        primary_section: "fontes",
        sections: &["fontes"],
        columns: &["_key", "edicoes", "paginas"],
        entity_group: EntityGroup::None,
    },
    EgressosDadosFileDef {
        file_key: "mapa_mundi_detalhe",
        file_name: "mapa_mundi_detalhe.json",
        relative_path: "data/mapa_mundi_detalhe.json",
        title: "Mapa-Múndi Detalhado (50m)",
        kind: FileKind::ObjectSections,
        publication: PublicationKind::Interno,
        role: DatasetRole::Saida,
        primary_section: "paths",
        sections: &["paths", "viewBox", "lat_range"],
        columns: &["value"],
        entity_group: EntityGroup::None,
    },
    // ---- 19..39: Datasets Publicáveis (em data/) ----
    EgressosDadosFileDef {
        file_key: "egressos_perfil",
        file_name: "egressos_perfil.json",
        relative_path: "data/egressos_perfil.json",
        title: "Perfis de Egressos",
        kind: FileKind::ObjectSections,
        publication: PublicationKind::Publicavel,
        role: DatasetRole::Saida,
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
    EgressosDadosFileDef {
        file_key: "vitrine",
        file_name: "vitrine.json",
        relative_path: "data/vitrine.json",
        title: "Vitrine de Egressos",
        kind: FileKind::ObjectSections,
        publication: PublicationKind::Publicavel,
        role: DatasetRole::Saida,
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
    EgressosDadosFileDef {
        file_key: "panorama",
        file_name: "panorama.json",
        relative_path: "data/panorama.json",
        title: "Panorama do Coorte",
        kind: FileKind::ObjectSections,
        publication: PublicationKind::Publicavel,
        role: DatasetRole::Saida,
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
    EgressosDadosFileDef {
        file_key: "analise",
        file_name: "analise.json",
        relative_path: "data/analise.json",
        title: "Análise do Grupo",
        kind: FileKind::ObjectSections,
        publication: PublicationKind::Publicavel,
        role: DatasetRole::Saida,
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
    EgressosDadosFileDef {
        file_key: "consolidado",
        file_name: "consolidado.json",
        relative_path: "data/consolidado.json",
        title: "Consolidado Salarial",
        kind: FileKind::ObjectSections,
        publication: PublicationKind::Publicavel,
        role: DatasetRole::Saida,
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
            "mult",
        ],
        entity_group: EntityGroup::Alumni,
    },
    EgressosDadosFileDef {
        file_key: "impacto",
        file_name: "impacto.json",
        relative_path: "data/impacto.json",
        title: "Impacto na Carreira",
        kind: FileKind::ObjectSections,
        publication: PublicationKind::Publicavel,
        role: DatasetRole::Saida,
        primary_section: "indicadores",
        sections: &[
            "indicadores",
            "por_experiencia",
            "por_experiencia_hoje",
            "dispersao",
            "vias_de_formacao",
            "internacional_no_tempo",
            "funcoes",
            "metodos",
            "tecnologias",
            "senioridade",
            "perfis_agrupados",
            "achados",
            "ressalvas",
        ],
        columns: &["rotulo", "valor", "detalhe"],
        entity_group: EntityGroup::None,
    },
    EgressosDadosFileDef {
        file_key: "trajetoria",
        file_name: "trajetoria.json",
        relative_path: "data/trajetoria.json",
        title: "Trajetória Salarial",
        kind: FileKind::ObjectSections,
        publication: PublicationKind::Publicavel,
        role: DatasetRole::Saida,
        primary_section: "serie",
        sections: &[
            "serie",
            "trajetoria",
            "sm_por_ano",
            "regua",
            "tabela_em_reais",
            "por_origem_do_empregador",
            "multiplicadores",
            "corroboracao_por_edicao",
            "moeda_por_faixa",
            "trajetoria_coorte",
        ],
        columns: &[
            "ano",
            "n",
            "med_real",
            "med_nominal",
            "sm_ano",
            "em_sm",
            "p25_real",
            "p75_real",
        ],
        entity_group: EntityGroup::None,
    },
    EgressosDadosFileDef {
        file_key: "recorte_cursos",
        file_name: "recorte_cursos.json",
        relative_path: "data/recorte_cursos.json",
        title: "Recorte por Cursos",
        kind: FileKind::NestedMap,
        publication: PublicationKind::Publicavel,
        role: DatasetRole::Saida,
        primary_section: "cursos",
        sections: &["cursos"],
        columns: &[
            "_key",
            "nome",
            "grau",
            "campus",
            "n_formandos",
            "n_confirmados",
            "cobertura_pct",
        ],
        entity_group: EntityGroup::None,
    },
    EgressosDadosFileDef {
        file_key: "fapes_fomento",
        file_name: "fapes_fomento.json",
        relative_path: "data/fapes_fomento.json",
        title: "Fomento FAPES",
        kind: FileKind::ObjectSections,
        publication: PublicationKind::Publicavel,
        role: DatasetRole::Saida,
        primary_section: "projetos",
        sections: &["projetos"],
        columns: &[
            "projeto_id",
            "titulo",
            "situacao",
            "instituicao",
            "bolsa_sigla",
            "valor_alocado",
            "valor_pago",
        ],
        entity_group: EntityGroup::None,
    },
    EgressosDadosFileDef {
        file_key: "renda_por_senioridade",
        file_name: "renda_por_senioridade.json",
        relative_path: "data/renda_por_senioridade.json",
        title: "Renda por Senioridade",
        kind: FileKind::ObjectSections,
        publication: PublicationKind::Publicavel,
        role: DatasetRole::Saida,
        primary_section: "cargos",
        sections: &["cargos", "niveis_sem_referencia_do_modelo"],
        columns: &["nivel", "referencias", "por_trilha_no_modelo"],
        entity_group: EntityGroup::None,
    },
    EgressosDadosFileDef {
        file_key: "cargos_ao_longo_do_tempo",
        file_name: "cargos_ao_longo_do_tempo.json",
        relative_path: "data/cargos_ao_longo_do_tempo.json",
        title: "Cargos ao Longo do Tempo",
        kind: FileKind::ObjectSections,
        publication: PublicationKind::Publicavel,
        role: DatasetRole::Saida,
        primary_section: "cargos",
        sections: &["cargos", "edicoes"],
        columns: &[
            "cargo",
            "nome",
            "n_edicoes",
            "de",
            "ate",
            "ultimo_med_brl",
            "ultimo_n",
        ],
        entity_group: EntityGroup::None,
    },
    EgressosDadosFileDef {
        file_key: "codigofonte_2026",
        file_name: "codigofonte_2026.json",
        relative_path: "data/codigofonte_2026.json",
        title: "Pesquisa Código Fonte 2026",
        kind: FileKind::ObjectSections,
        publication: PublicationKind::Publicavel,
        role: DatasetRole::Entrada,
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
    EgressosDadosFileDef {
        file_key: "codigofonte_historico",
        file_name: "codigofonte_historico.json",
        relative_path: "data/codigofonte_historico.json",
        title: "Código Fonte — Série Histórica",
        kind: FileKind::NestedMap,
        publication: PublicationKind::Publicavel,
        role: DatasetRole::Entrada,
        primary_section: "media_por_senioridade",
        sections: &["media_por_senioridade", "ipca_fator_para_2026"],
        columns: &["_key", "Junior", "Pleno", "Senior"],
        entity_group: EntityGroup::None,
    },
    EgressosDadosFileDef {
        file_key: "so_benchmarks",
        file_name: "so_benchmarks.json",
        relative_path: "data/so_benchmarks.json",
        title: "Benchmarks Stack Overflow",
        kind: FileKind::ObjectSections,
        publication: PublicationKind::Publicavel,
        role: DatasetRole::Saida,
        primary_section: "por_pais",
        sections: &[
            "por_pais",
            "por_moeda_brasil",
            "serie_anual",
            "candidatas_a_referencia",
            "notas",
        ],
        columns: &[
            "pais",
            "rotulo",
            "n",
            "med_usd_anual",
            "med_usd_mes",
            "p25_usd_mes",
            "p75_usd_mes",
        ],
        entity_group: EntityGroup::None,
    },
    EgressosDadosFileDef {
        file_key: "salario_minimo",
        file_name: "salario_minimo.json",
        relative_path: "data/salario_minimo.json",
        title: "Série do Salário Mínimo",
        kind: FileKind::ObjectSections,
        publication: PublicationKind::Publicavel,
        role: DatasetRole::Saida,
        primary_section: "por_ano",
        sections: &[
            "por_ano",
            "notas",
            "sm_por_ano",
            "deflator_ipca_por_ano",
            "cambio_por_ano",
        ],
        columns: &[
            "ano",
            "jan",
            "dez",
            "media_ponderada",
            "reajuste_pct",
            "inpc_ano_anterior_pct",
            "ganho_real_pct",
        ],
        entity_group: EntityGroup::None,
    },
    EgressosDadosFileDef {
        file_key: "ibge_series",
        file_name: "ibge_series.json",
        relative_path: "data/ibge_series.json",
        title: "Séries IBGE (IPCA/INPC)",
        kind: FileKind::NestedMap,
        publication: PublicationKind::Publicavel,
        role: DatasetRole::Saida,
        primary_section: "series",
        sections: &["series"],
        columns: &["_key", "nome", "sidra_tabela", "variavel", "unidade"],
        entity_group: EntityGroup::None,
    },
    EgressosDadosFileDef {
        file_key: "empresas_porte",
        file_name: "empresas_porte.json",
        relative_path: "data/empresas_porte.json",
        title: "Porte das Empresas",
        kind: FileKind::RootMap,
        publication: PublicationKind::Publicavel,
        role: DatasetRole::Saida,
        primary_section: "_root",
        sections: &["_root"],
        columns: &[
            "_key",
            "empresa",
            "porte",
            "origem",
            "setor",
            "funcionarios",
        ],
        entity_group: EntityGroup::Company,
    },
    EgressosDadosFileDef {
        file_key: "empresas_aliases",
        file_name: "empresas_aliases.json",
        relative_path: "data/empresas_aliases.json",
        title: "Aliases de Empresas",
        kind: FileKind::RootMap,
        publication: PublicationKind::Publicavel,
        role: DatasetRole::Saida,
        primary_section: "_root",
        sections: &["_root"],
        columns: &["_key", "aliases", "atual", "n_egressos_atual"],
        entity_group: EntityGroup::Company,
    },
    EgressosDadosFileDef {
        file_key: "empresas_linkedin_data",
        file_name: "empresas_linkedin_data.json",
        relative_path: "data/empresas_linkedin_data.json",
        title: "Dados LinkedIn das Empresas",
        kind: FileKind::RootMap,
        publication: PublicationKind::Publicavel,
        role: DatasetRole::Saida,
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
    EgressosDadosFileDef {
        file_key: "empresas_linkedin_urls",
        file_name: "empresas_linkedin_urls.json",
        relative_path: "data/empresas_linkedin_urls.json",
        title: "URLs LinkedIn das Empresas",
        kind: FileKind::RootMap,
        publication: PublicationKind::Publicavel,
        role: DatasetRole::Saida,
        primary_section: "_root",
        sections: &["_root"],
        columns: &["_key", "url", "slug", "confianca", "via"],
        entity_group: EntityGroup::Company,
    },
    EgressosDadosFileDef {
        file_key: "mapa_mundi",
        file_name: "mapa_mundi.json",
        relative_path: "data/mapa_mundi.json",
        title: "Mapa-Múndi (Geometria SVG)",
        kind: FileKind::ObjectSections,
        publication: PublicationKind::Publicavel,
        role: DatasetRole::Saida,
        primary_section: "paths",
        sections: &["paths", "viewBox", "lat_range"],
        columns: &["value"],
        entity_group: EntityGroup::None,
    },
];

pub fn by_file_key(file_key: &str) -> Result<&'static EgressosDadosFileDef, AppError> {
    EGRESSOS_DADOS_FILES
        .iter()
        .find(|f| f.file_key == file_key)
        .ok_or_else(|| AppError::UnknownEntity(format!("egressos_dados:{file_key}")))
}

/// Retorna o grupo de entidade de uma seção específica (para propagação `FR-007`).
pub fn section_entity_group(file_key: &str, section_key: &str) -> EntityGroup {
    match (file_key, section_key) {
        ("alunos", "alunos")
        | ("_novos_reais", "_root")
        | ("genero_map", "_root")
        | ("horizon_pesquisa", "encontrados")
        | ("lattes_orientacoes", "encontrados")
        | ("outros_labs", "egressos")
        | ("consolidado_raw", "_root")
        | ("egressos_perfil", "egressos")
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
pub struct EgressosDadosImportProgress {
    pub phase: &'static str,
    pub file_key: Option<String>,
    pub files_done: usize,
    pub files_total: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EgressosDadosImportSummary {
    pub path: String,
    pub total_files: usize,
    pub total_rows: usize,
    pub replaced: bool,
    pub snapshot: Option<String>,
    pub imported_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EgressosDadosExportSummary {
    pub path: String,
    pub total_files: usize,
    pub total_rows: usize,
}

#[derive(Serialize, Debug, Clone)]
pub struct EgressosDadosSectionSummary {
    pub section_key: String,
    pub count: usize,
    pub entity_group: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct EgressosDadosFileSummary {
    pub file_key: String,
    pub file_name: String,
    pub relative_path: String,
    pub title: String,
    pub kind: String,
    pub publication: String,
    pub role: String,
    pub primary_section: String,
    pub sections: Vec<EgressosDadosSectionSummary>,
    pub columns: Vec<String>,
    pub entity_group: String,
    pub total_rows: usize,
}

#[derive(Serialize, Debug, Clone)]
pub struct EgressosDadosCatalogView {
    pub loaded: bool,
    pub imported_at: Option<String>,
    pub total_files: usize,
    pub total_rows: usize,
    pub default_local_dir: String,
    pub default_local_dir_exists: bool,
    pub files: Vec<EgressosDadosFileSummary>,
}

#[derive(Serialize, Debug, Clone)]
pub struct EgressosDadosRowPage {
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
pub struct EgressosDadosMutationResult {
    pub record: Record,
    pub propagated_count: usize,
}

#[derive(Serialize, Debug, Clone)]
pub struct EgressosDadosDeleteResult {
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
            for field in ["id", "label", "rotulo", "perfil"] {
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
                "matricula",
                "nome",
                "ano",
                "nivel",
                "cargo",
                "pais",
                "rotulo",
                "label",
                "area",
                "familia",
                "lab",
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
                    } else if let Some(sub_obj) = item.as_object() {
                        for sv in sub_obj.values() {
                            if let Some(s) = sv.as_str() {
                                parts.push(s.to_lowercase());
                            }
                        }
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

pub fn has_egressos_dados_data(conn: &Connection) -> Result<bool, AppError> {
    let n: i64 = conn.query_row(
        "SELECT COUNT(*) FROM egressos_dados_files",
        [],
        |r| r.get(0),
    )?;
    Ok(n > 0)
}

#[derive(serde::Serialize, Clone, Debug, PartialEq, Eq)]
pub struct EgressosDadosSnapshotInfo {
    pub file_name: String,
    pub path: String,
    pub size_bytes: u64,
    pub created_at_secs: u64,
}

/// Lista os snapshots de segurança exclusivos do domínio `egressos-dados`
/// (`egressos-dados-*`) armazenados em `data_dir/snapshots/`.
pub fn list_snapshots(data_dir: &Path) -> Result<Vec<EgressosDadosSnapshotInfo>, AppError> {
    let dir = data_dir.join("snapshots");
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut items = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.starts_with("egressos-dados-") {
            continue;
        }
        let meta = entry.metadata()?;
        let created_at_secs = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        items.push(EgressosDadosSnapshotInfo {
            file_name: name.to_string(),
            path: path.display().to_string(),
            size_bytes: meta.len(),
            created_at_secs,
        });
    }
    items.sort_by(|a, b| b.created_at_secs.cmp(&a.created_at_secs).then_with(|| b.file_name.cmp(&a.file_name)));
    Ok(items)
}

fn is_scalar_list_section(file_key: &str, section_key: &str) -> bool {
    if file_key == "_empresas_lista" && section_key == "_root" {
        return true;
    }
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
            | "tabelas_usadas"
            | "sem_vinculo_pesquisa"
            | "nao_encontrados"
            | "fapes_pesquisa"
            | "bolsa_documentada_oficial"
    )
}

/// Resolve automaticamente o diretório raiz do repositório (`alunos.json` + `data/*.json`),
/// aceitando tanto `/caminho/egressos-dados` quanto `/caminho/egressos-dados/data` (`R4`, `FR-008`).
pub fn resolve_repo_root(input_dir: &Path) -> PathBuf {
    let file_name = input_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    if file_name == "data" && !input_dir.join("alunos.json").exists() {
        if let Some(parent) = input_dir.parent() {
            if parent.join("alunos.json").exists() || parent.exists() {
                return parent.to_path_buf();
            }
        }
    }
    input_dir.to_path_buf()
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
    def: &'static EgressosDadosFileDef,
    envelope_json: String,
    rows: Vec<ParsedRow>,
}

fn parse_and_validate_single_file(
    def: &'static EgressosDadosFileDef,
    text: &str,
) -> Result<ParsedFile, AppError> {
    let root: Json = serde_json::from_str(text).map_err(|e| {
        AppError::Validation(format!(
            "arquivo '{}' não é um JSON válido: {e}",
            def.relative_path
        ))
    })?;

    let mut rows = Vec::new();

    if def.kind == FileKind::RootArray {
        let arr = root.as_array().ok_or_else(|| {
            AppError::Validation(format!(
                "arquivo '{}' deve ter uma lista JSON na raiz",
                def.relative_path
            ))
        })?;
        let all_objects = if arr.is_empty() {
            !is_scalar_list_section(def.file_key, "_root")
        } else {
            arr.iter().all(|item| item.is_object())
        };
        let shape = if all_objects {
            for (ord, item) in arr.iter().enumerate() {
                let item_obj = item.as_object().unwrap();
                let rec_key = extract_record_key(def.file_key, "_root", item_obj);
                let search_text = build_search_text(item_obj);
                rows.push(ParsedRow {
                    section_key: "_root".to_string(),
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
                    section_key: "_root".to_string(),
                    ord,
                    record_key: rec_key,
                    search_text,
                    raw_json: Json::Object(wrapper).to_string(),
                });
            }
            SHAPE_ARRAY_SCALARS
        };
        let mut env = Map::new();
        env.insert(ROOT_SHAPE_KEY.to_string(), Json::String(shape.to_string()));
        return Ok(ParsedFile {
            def,
            envelope_json: Json::Object(env).to_string(),
            rows,
        });
    }

    let root_obj = root.as_object().ok_or_else(|| {
        AppError::Validation(format!(
            "arquivo '{}' deve ter um objeto JSON na raiz",
            def.relative_path
        ))
    })?;

    if def.kind == FileKind::RootMap {
        let all_objects = if root_obj.is_empty() {
            def.file_key != "genero_map"
        } else {
            root_obj.values().all(|v| v.is_object())
        };
        let shape = if all_objects {
            for (ord, (k, v)) in root_obj.iter().enumerate() {
                let val_obj = v.as_object().ok_or_else(|| {
                    AppError::Validation(format!(
                        "arquivo '{}': entrada '{}' deve ser um objeto JSON",
                        def.relative_path, k
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
            SHAPE_MAP_OBJECTS
        } else {
            for (ord, (k, v)) in root_obj.iter().enumerate() {
                let mut row_map = Map::new();
                row_map.insert("_key".to_string(), Json::String(k.clone()));
                row_map.insert("value".to_string(), v.clone());
                let search_text = build_search_text(&row_map);
                rows.push(ParsedRow {
                    section_key: "_root".to_string(),
                    ord,
                    record_key: Some(k.clone()),
                    search_text,
                    raw_json: Json::Object(row_map).to_string(),
                });
            }
            SHAPE_MAP_SCALARS
        };
        let mut env = Map::new();
        env.insert(ROOT_SHAPE_KEY.to_string(), Json::String(shape.to_string()));
        return Ok(ParsedFile {
            def,
            envelope_json: Json::Object(env).to_string(),
            rows,
        });
    }

    if !root_obj.contains_key(def.primary_section) {
        return Err(AppError::Validation(format!(
            "arquivo '{}' não possui a seção obrigatória '{}'",
            def.relative_path, def.primary_section
        )));
    }

    let mut envelope_map = Map::new();
    for (k, v) in root_obj {
        if def.sections.contains(&k.as_str()) {
            let shape = match v {
                Json::Array(arr) => {
                    let all_objects = if arr.is_empty() {
                        !is_scalar_list_section(def.file_key, k)
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
                        def.relative_path, k
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

/// Importa e valida atomicamente os 39 arquivos JSON do repositório `egressos-dados`
/// (`alunos.json` na raiz + 38 arquivos em `data/*.json`) (`FR-008`).
pub fn import_dir<F>(
    conn: &mut Connection,
    source_dir: &Path,
    app_data_dir: &Path,
    mut on_progress: F,
) -> Result<EgressosDadosImportSummary, AppError>
where
    F: FnMut(EgressosDadosImportProgress),
{
    let repo_root = resolve_repo_root(source_dir);
    if !repo_root.exists() || !repo_root.is_dir() {
        return Err(AppError::Validation(format!(
            "A pasta '{}' não existe ou não é um diretório válido.",
            repo_root.display()
        )));
    }

    on_progress(EgressosDadosImportProgress {
        phase: "validating",
        file_key: None,
        files_done: 0,
        files_total: EGRESSOS_DADOS_FILES.len(),
    });

    // Verifica todos os 39 arquivos ANTES de abrir transação.
    let mut missing = Vec::new();
    for def in EGRESSOS_DADOS_FILES {
        if !repo_root.join(def.relative_path).is_file() {
            missing.push(def.relative_path);
        }
    }
    if !missing.is_empty() {
        return Err(AppError::Validation(format!(
            "Repositório egressos-dados incompleto — arquivos JSON obrigatórios ausentes: {}",
            missing.join(", ")
        )));
    }

    let mut parsed_files = Vec::with_capacity(EGRESSOS_DADOS_FILES.len());
    for def in EGRESSOS_DADOS_FILES {
        let file_path = repo_root.join(def.relative_path);
        let text = std::fs::read_to_string(&file_path)?;
        let parsed = parse_and_validate_single_file(def, &text)?;
        parsed_files.push(parsed);
    }

    let replaced = has_egressos_dados_data(conn)?;
    let snapshot = if replaced {
        Some(
            crate::import::snapshot_with_prefix(conn, app_data_dir, "egressos-dados-pre-import")?
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
    tx.execute("DELETE FROM egressos_dados_rows", [])?;
    tx.execute("DELETE FROM egressos_dados_files", [])?;

    for (idx, pf) in parsed_files.iter().enumerate() {
        tx.execute(
            "INSERT INTO egressos_dados_files (file_key, kind, primary_section, envelope_json, imported_at) \
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
                "INSERT INTO egressos_dados_rows (file_key, section_key, ord, record_key, search_text, raw_json) \
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

        on_progress(EgressosDadosImportProgress {
            phase: "importing",
            file_key: Some(pf.def.file_key.to_string()),
            files_done: idx + 1,
            files_total: EGRESSOS_DADOS_FILES.len(),
        });
    }

    tx.commit()?;

    on_progress(EgressosDadosImportProgress {
        phase: "done",
        file_key: None,
        files_done: EGRESSOS_DADOS_FILES.len(),
        files_total: EGRESSOS_DADOS_FILES.len(),
    });

    Ok(EgressosDadosImportSummary {
        path: repo_root.display().to_string(),
        total_files: EGRESSOS_DADOS_FILES.len(),
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
        "SELECT raw_json FROM egressos_dados_rows \
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

fn default_root_shape(def: &EgressosDadosFileDef) -> &'static str {
    match def.kind {
        FileKind::RootArray => {
            if is_scalar_list_section(def.file_key, "_root") {
                SHAPE_ARRAY_SCALARS
            } else {
                SHAPE_ARRAY_OBJECTS
            }
        }
        FileKind::RootMap => {
            if def.file_key == "genero_map" {
                SHAPE_MAP_SCALARS
            } else {
                SHAPE_MAP_OBJECTS
            }
        }
        FileKind::NestedMap => SHAPE_MAP_OBJECTS,
        FileKind::ObjectSections => SHAPE_ARRAY_OBJECTS,
    }
}

/// Reconstrói o `serde_json::Value` completo de um único arquivo `file_key` de `egressos-dados`.
pub fn reconstruct_file_json(conn: &Connection, file_key: &str) -> Result<Json, AppError> {
    let def = by_file_key(file_key)?;
    let envelope_str: Option<String> = conn
        .query_row(
            "SELECT envelope_json FROM egressos_dados_files WHERE file_key = ?1",
            [file_key],
            |r| r.get(0),
        )
        .optional()?;

    let envelope_str = envelope_str.ok_or_else(|| {
        AppError::Validation(format!(
            "O arquivo '{}' ainda não foi importado na base de Egressos — Dados.",
            def.relative_path
        ))
    })?;

    let envelope: Json = serde_json::from_str(&envelope_str)
        .map_err(|e| AppError::Internal(format!("envelope_json inválido: {e}")))?;
    let envelope_obj = envelope.as_object().cloned().unwrap_or_default();

    if def.kind == FileKind::RootArray || def.kind == FileKind::RootMap {
        let shape = envelope_obj
            .get(ROOT_SHAPE_KEY)
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| default_root_shape(def));
        return reconstruct_section(conn, file_key, "_root", shape);
    }

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

/// Exporta os 39 arquivos JSON em memória como `(relative_path, pretty_json_bytes)`
/// para uso tanto na exportação local quanto no upload atômico para o GitHub (`FR-009`, `FR-010`).
pub fn export_files_in_memory(conn: &Connection) -> Result<Vec<(String, Vec<u8>)>, AppError> {
    if !has_egressos_dados_data(conn)? {
        return Err(AppError::Validation(
            "Nenhuma base de Egressos — Dados carregada para exportar.".into(),
        ));
    }

    let mut out = Vec::with_capacity(EGRESSOS_DADOS_FILES.len());
    for def in EGRESSOS_DADOS_FILES {
        let json_val = reconstruct_file_json(conn, def.file_key)?;
        let mut pretty = serde_json::to_string_pretty(&json_val).map_err(|e| {
            AppError::Internal(format!("falha ao serializar {}: {e}", def.relative_path))
        })?;
        pretty.push('\n');
        out.push((def.relative_path.to_string(), pretty.into_bytes()));
    }
    Ok(out)
}

/// Exporta os 39 arquivos `.json` (`alunos.json` na raiz e 38 arquivos em `data/*.json`)
/// para o diretório raiz escolhido (`FR-009`).
pub fn export_dir(
    conn: &Connection,
    target_dir: &Path,
) -> Result<EgressosDadosExportSummary, AppError> {
    let repo_root = resolve_repo_root(target_dir);
    let files = export_files_in_memory(conn)?;
    std::fs::create_dir_all(&repo_root)?;
    std::fs::create_dir_all(repo_root.join("data"))?;

    for (rel_path, bytes) in &files {
        let full_path = repo_root.join(rel_path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(full_path, bytes)?;
    }

    let total_rows: i64 =
        conn.query_row("SELECT COUNT(*) FROM egressos_dados_rows", [], |r| r.get(0))?;

    Ok(EgressosDadosExportSummary {
        path: repo_root.display().to_string(),
        total_files: files.len(),
        total_rows: total_rows as usize,
    })
}

/// Retorna a visão completa do catálogo das 39 abas e contagens atuais (`egressos_dados_get_catalog`).
pub fn get_catalog(conn: &Connection) -> Result<EgressosDadosCatalogView, AppError> {
    let loaded = has_egressos_dados_data(conn)?;
    let imported_at: Option<String> = conn
        .query_row(
            "SELECT MAX(imported_at) FROM egressos_dados_files",
            [],
            |r| r.get(0),
        )
        .optional()?
        .flatten();

    let mut count_stmt = conn.prepare(
        "SELECT file_key, section_key, COUNT(*) FROM egressos_dados_rows GROUP BY file_key, section_key",
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
    let mut files = Vec::with_capacity(EGRESSOS_DADOS_FILES.len());
    for def in EGRESSOS_DADOS_FILES {
        let mut sections = Vec::with_capacity(def.sections.len());
        let mut file_total = 0usize;
        for &sec in def.sections {
            let c = section_counts
                .get(&(def.file_key.to_string(), sec.to_string()))
                .copied()
                .unwrap_or(0);
            file_total += c;
            sections.push(EgressosDadosSectionSummary {
                section_key: sec.to_string(),
                count: c,
                entity_group: section_entity_group(def.file_key, sec).as_str().to_string(),
            });
        }
        total_rows += file_total;
        files.push(EgressosDadosFileSummary {
            file_key: def.file_key.to_string(),
            file_name: def.file_name.to_string(),
            relative_path: def.relative_path.to_string(),
            title: def.title.to_string(),
            kind: def.kind.as_str().to_string(),
            publication: def.publication.as_str().to_string(),
            role: def.role.as_str().to_string(),
            primary_section: def.primary_section.to_string(),
            sections,
            columns: def.columns.iter().map(|c| (*c).to_string()).collect(),
            entity_group: def.entity_group.as_str().to_string(),
            total_rows: file_total,
        });
    }

    let default_path = Path::new(DEFAULT_LOCAL_REPO_DIR);
    let default_exists = default_path.is_dir() && default_path.join("alunos.json").is_file();

    Ok(EgressosDadosCatalogView {
        loaded,
        imported_at,
        total_files: if loaded { EGRESSOS_DADOS_FILES.len() } else { 0 },
        total_rows,
        default_local_dir: if default_exists {
            DEFAULT_LOCAL_REPO_DIR.to_string()
        } else {
            String::new()
        },
        default_local_dir_exists: default_exists,
        files,
    })
}

// --------------------------------------------------- CRUD & Propagação (US1)

fn section_shape_from_db(
    conn: &Connection,
    def: &EgressosDadosFileDef,
    section_key: &str,
) -> Result<String, AppError> {
    let env_str: Option<String> = conn
        .query_row(
            "SELECT envelope_json FROM egressos_dados_files WHERE file_key = ?1",
            [def.file_key],
            |r| r.get(0),
        )
        .optional()?;

    if let Some(s) = env_str {
        if let Ok(Json::Object(map)) = serde_json::from_str::<Json>(&s) {
            if def.kind == FileKind::RootArray || def.kind == FileKind::RootMap {
                if let Some(shape) = map.get(ROOT_SHAPE_KEY).and_then(|v| v.as_str()) {
                    return Ok(shape.to_string());
                }
            }
            if let Some(Json::Object(sentinel)) = map.get(section_key) {
                if let Some(shape) = sentinel.get(SENTINEL_SHAPE_KEY).and_then(|v| v.as_str()) {
                    return Ok(shape.to_string());
                }
            }
        }
    }

    if def.kind == FileKind::RootArray || def.kind == FileKind::RootMap {
        return Ok(default_root_shape(def).to_string());
    }
    if def.kind == FileKind::NestedMap {
        Ok(SHAPE_MAP_OBJECTS.to_string())
    } else if is_scalar_list_section(def.file_key, section_key) {
        Ok(SHAPE_ARRAY_SCALARS.to_string())
    } else {
        Ok(SHAPE_ARRAY_OBJECTS.to_string())
    }
}

/// Converte uma linha do banco em `Record`, injetando `"_row_id": id` e preservando
/// qualquer campo `"id"` textual existente no JSON original (`R3`).
fn row_to_record(id: i64, record_key: Option<String>, raw_json: &str) -> Result<Record, AppError> {
    let parsed: Json = serde_json::from_str(raw_json)
        .map_err(|e| AppError::Internal(format!("raw_json inválido na linha {id}: {e}")))?;
    let mut map = parsed.as_object().cloned().unwrap_or_default();
    map.insert("_row_id".to_string(), Json::Number(id.into()));
    if !map.contains_key("id") {
        map.insert("id".to_string(), Json::Number(id.into()));
    }
    if let Some(rk) = record_key {
        map.insert("_record_key".to_string(), Json::String(rk));
    }
    Ok(map)
}

fn infer_columns_for_section(
    def: &EgressosDadosFileDef,
    section_key: &str,
    items: &[Record],
) -> Vec<String> {
    if section_key == def.primary_section {
        return def.columns.iter().map(|c| (*c).to_string()).collect();
    }
    if let Some(first) = items.first() {
        let cols: Vec<String> = first
            .keys()
            .filter(|k| *k != "_row_id" && *k != "_record_key" && !(*k == "id" && first.get("id").map(|v| v.is_number()).unwrap_or(false)))
            .take(8)
            .cloned()
            .collect();
        if !cols.is_empty() {
            return cols;
        }
    }
    vec!["_key".to_string(), "value".to_string()]
}

/// Lista registros de uma seção de um dos 39 arquivos com busca, ordenação e paginação (`FR-006`).
pub fn list_rows(
    conn: &Connection,
    file_key: &str,
    section_key: Option<&str>,
    limit: Option<i64>,
    offset: Option<i64>,
    search: Option<&str>,
    sort: Option<&str>,
    order: Option<&str>,
) -> Result<EgressosDadosRowPage, AppError> {
    list_rows_filtered(conn, file_key, section_key, limit, offset, search, sort, order, None)
}

pub fn list_rows_filtered(
    conn: &Connection,
    file_key: &str,
    section_key: Option<&str>,
    limit: Option<i64>,
    offset: Option<i64>,
    search: Option<&str>,
    sort: Option<&str>,
    order: Option<&str>,
    filters: Option<&str>,
) -> Result<EgressosDadosRowPage, AppError> {
    let def = by_file_key(file_key)?;
    let filter_spec = crate::analytics::parse_filter_spec(filters);
    let sec = section_key
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(def.primary_section);
    if !def.sections.contains(&sec) {
        return Err(AppError::Validation(format!(
            "Seção '{}' não existe no arquivo '{}'.",
            sec, def.relative_path
        )));
    }

    let limit = limit.unwrap_or(50).clamp(1, 500);
    let offset = offset.unwrap_or(0).max(0);
    let q = search.map(str::trim).filter(|s| !s.is_empty());

    let mut stmt = conn.prepare(
        "SELECT id, record_key, search_text, raw_json FROM egressos_dados_rows \
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
        let rec = row_to_record(id, rk, &raw)?;
        if !filter_spec.is_empty() && !crate::analytics::record_matches_filter_spec(&rec, &filter_spec) {
            continue;
        }
        filtered.push(rec);
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

    Ok(EgressosDadosRowPage {
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

/// Busca um registro específico pelo `id` interno do SQLite (`_row_id`).
pub fn get_row(conn: &Connection, id: i64) -> Result<Record, AppError> {
    let row: Option<(Option<String>, String)> = conn
        .query_row(
            "SELECT record_key, raw_json FROM egressos_dados_rows WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;

    let (rk, raw) = row.ok_or(AppError::NotFound)?;
    row_to_record(id, rk, &raw)
}

fn clean_payload_for_storage(def: &EgressosDadosFileDef, payload: &Record) -> Record {
    let has_domain_id_col = def.columns.contains(&"id");
    payload
        .iter()
        .filter(|(k, v)| {
            if *k == "_row_id" || *k == "_record_key" {
                return false;
            }
            if *k == "id" && !has_domain_id_col && v.is_number() {
                return false;
            }
            true
        })
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

/// Cria um novo registro em `(file_key, section_key)` (`FR-006`).
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
            sec, def.relative_path
        )));
    }

    let cleaned = clean_payload_for_storage(def, payload);
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
            "SELECT COUNT(*) FROM egressos_dados_rows \
             WHERE file_key = ?1 AND section_key = ?2 AND record_key = ?3",
            params![file_key, sec, key_val],
            |r| r.get(0),
        )?;
        if dup > 0 {
            return Err(AppError::Conflict(format!(
                "Já existe um registro com a chave '{key_val}' em {}/{sec}.",
                def.relative_path
            )));
        }
    }

    let default_env = if def.kind == FileKind::RootArray || def.kind == FileKind::RootMap {
        let mut m = Map::new();
        m.insert(
            ROOT_SHAPE_KEY.to_string(),
            Json::String(default_root_shape(def).to_string()),
        );
        Json::Object(m).to_string()
    } else {
        "{}".to_string()
    };

    conn.execute(
        "INSERT OR IGNORE INTO egressos_dados_files (file_key, kind, primary_section, envelope_json) \
         VALUES (?1, ?2, ?3, ?4)",
        params![
            def.file_key,
            def.kind.as_str(),
            def.primary_section,
            default_env
        ],
    )?;

    let next_ord: i64 = conn.query_row(
        "SELECT COALESCE(MAX(ord), -1) + 1 FROM egressos_dados_rows WHERE file_key = ?1 AND section_key = ?2",
        params![file_key, sec],
        |r| r.get(0),
    )?;

    let rec_key = extract_record_key(file_key, sec, &cleaned);
    let search_text = build_search_text(&cleaned);
    let raw_json = Json::Object(cleaned).to_string();

    conn.execute(
        "INSERT INTO egressos_dados_rows (file_key, section_key, ord, record_key, search_text, raw_json) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![file_key, sec, next_ord, rec_key, search_text, raw_json],
    )?;
    let id = conn.last_insert_rowid();
    get_row(conn, id)
}

/// Atualiza um registro existente e, se `propagate == true` (`FR-007`), propaga
/// os campos equivalentes para os registros irmãos do mesmo grupo (`Alumni` ou `Company`).
pub fn update_row(
    conn: &mut Connection,
    id: i64,
    payload: &Record,
    propagate: bool,
) -> Result<EgressosDadosMutationResult, AppError> {
    let existing: Option<(String, String, Option<String>, String)> = conn
        .query_row(
            "SELECT file_key, section_key, record_key, raw_json FROM egressos_dados_rows WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?;

    let (file_key, section_key, old_record_key, old_raw) = existing.ok_or(AppError::NotFound)?;

    let def = by_file_key(&file_key)?;
    let mut current_obj: Map<String, Json> = serde_json::from_str(&old_raw)
        .map_err(|e| AppError::Internal(format!("raw_json inválido: {e}")))?;

    for (k, v) in clean_payload_for_storage(def, payload) {
        current_obj.insert(k, v);
    }

    let shape = section_shape_from_db(conn, def, &section_key)?;
    if shape == SHAPE_MAP_OBJECTS || shape == SHAPE_MAP_SCALARS {
        let key_val = current_obj
            .get("_key")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| AppError::Validation("O campo '_key' não pode ficar vazio.".into()))?;
        let dup: i64 = conn.query_row(
            "SELECT COUNT(*) FROM egressos_dados_rows \
             WHERE file_key = ?1 AND section_key = ?2 AND record_key = ?3 AND id != ?4",
            params![file_key, section_key, key_val, id],
            |r| r.get(0),
        )?;
        if dup > 0 {
            return Err(AppError::Conflict(format!(
                "Já existe outro registro com a chave '{key_val}' em {}/{section_key}.",
                def.relative_path
            )));
        }
    }

    let new_record_key = extract_record_key(&file_key, &section_key, &current_obj);
    let new_search = build_search_text(&current_obj);
    let new_raw = Json::Object(current_obj.clone()).to_string();

    let tx = conn.transaction()?;
    tx.execute(
        "UPDATE egressos_dados_rows SET record_key = ?1, search_text = ?2, raw_json = ?3 WHERE id = ?4",
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
    Ok(EgressosDadosMutationResult {
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
        "SELECT id, file_key, section_key, raw_json FROM egressos_dados_rows \
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
                        if target_obj.contains_key("id") {
                            target_obj.insert("id".into(), Json::String(nk.to_string()));
                            changed = true;
                        }
                        if target_obj.contains_key("_key") {
                            target_obj.insert("_key".into(), Json::String(nk.to_string()));
                            changed = true;
                        }
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
                for same_key in [
                    "nome",
                    "monograma",
                    "curso",
                    "area_atual",
                    "inicio_carreira_dev",
                    "ainda_em_tech",
                    "trilha",
                    "em_tech",
                    "anos",
                ] {
                    if let Some(val) = source_obj.get(same_key) {
                        if target_obj.contains_key(same_key) && target_obj.get(same_key) != Some(val)
                        {
                            target_obj.insert(same_key.to_string(), val.clone());
                            changed = true;
                        }
                    }
                }
                if let Some(cargo) = source_obj
                    .get("cargo_atual")
                    .or_else(|| source_obj.get("cargo"))
                {
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
                if let Some(loc) = source_obj
                    .get("local_atual")
                    .or_else(|| source_obj.get("lugar"))
                {
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
                for same_key in [
                    "porte",
                    "origem",
                    "setor",
                    "nome_oficial",
                    "url",
                    "funcionarios",
                ] {
                    if let Some(val) = source_obj.get(same_key) {
                        if target_obj.contains_key(same_key) && target_obj.get(same_key) != Some(val)
                        {
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
                "UPDATE egressos_dados_rows SET record_key = ?1, search_text = ?2, raw_json = ?3 WHERE id = ?4",
                params![new_rk, new_st, new_raw, target_id],
            )?;
            count += 1;
        }
    }

    Ok(count)
}

/// Remove um registro pelo `id` e, se `propagate == true` (`FR-007`), remove também
/// os registros irmãos do mesmo grupo (`Alumni` ou `Company`).
pub fn delete_row(
    conn: &mut Connection,
    id: i64,
    propagate: bool,
) -> Result<EgressosDadosDeleteResult, AppError> {
    let existing: Option<(String, String, Option<String>)> = conn
        .query_row(
            "SELECT file_key, section_key, record_key FROM egressos_dados_rows WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;

    let (file_key, section_key, record_key) = existing.ok_or(AppError::NotFound)?;

    let tx = conn.transaction()?;
    let deleted = tx.execute("DELETE FROM egressos_dados_rows WHERE id = ?1", [id])?;

    let mut propagated_deleted = 0usize;
    if propagate {
        if let Some(ref rk) = record_key {
            let group = section_entity_group(&file_key, &section_key);
            if group != EntityGroup::None {
                let mut stmt = tx.prepare(
                    "SELECT id, file_key, section_key FROM egressos_dados_rows WHERE record_key = ?1",
                )?;
                let siblings: Vec<(i64, String, String)> = stmt
                    .query_map([rk], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
                    .collect::<Result<Vec<_>, _>>()?;
                drop(stmt);

                for (sib_id, s_file, s_sec) in siblings {
                    if section_entity_group(&s_file, &s_sec) == group {
                        tx.execute("DELETE FROM egressos_dados_rows WHERE id = ?1", [sib_id])?;
                        propagated_deleted += 1;
                    }
                }
            }
        }
    }

    tx.commit()?;
    Ok(EgressosDadosDeleteResult {
        deleted,
        propagated_deleted,
    })
}

/// Retorna os metadados de nível de arquivo (`envelope`) ocultando os marcadores internos.
pub fn get_envelope(conn: &Connection, file_key: &str) -> Result<Record, AppError> {
    let def = by_file_key(file_key)?;
    let env_str: Option<String> = conn
        .query_row(
            "SELECT envelope_json FROM egressos_dados_files WHERE file_key = ?1",
            [def.file_key],
            |r| r.get(0),
        )
        .optional()?;

    let env_str = env_str.unwrap_or_else(|| "{}".to_string());
    let parsed: Json = serde_json::from_str(&env_str)
        .map_err(|e| AppError::Internal(format!("envelope_json inválido: {e}")))?;
    let obj = parsed.as_object().cloned().unwrap_or_default();

    let mut clean = Map::new();
    for (k, v) in obj {
        if k == ROOT_SHAPE_KEY {
            continue;
        }
        let is_sentinel = v
            .as_object()
            .and_then(|o| o.get(SENTINEL_SECTION_KEY))
            .is_some();
        if !is_sentinel {
            clean.insert(k, v);
        }
    }
    Ok(clean)
}

/// Atualiza os metadados de nível de arquivo (`envelope`) preservando os marcadores internos.
pub fn update_envelope(
    conn: &Connection,
    file_key: &str,
    payload: &Record,
) -> Result<Record, AppError> {
    let def = by_file_key(file_key)?;
    if def.kind == FileKind::RootMap || def.kind == FileKind::RootArray {
        return Err(AppError::Validation(format!(
            "O arquivo '{}' não possui envelope de metadados editável.",
            def.relative_path
        )));
    }

    let env_str: Option<String> = conn
        .query_row(
            "SELECT envelope_json FROM egressos_dados_files WHERE file_key = ?1",
            [def.file_key],
            |r| r.get(0),
        )
        .optional()?;

    let current_env: Json = serde_json::from_str(&env_str.unwrap_or_else(|| "{}".to_string()))
        .map_err(|e| AppError::Internal(format!("envelope_json inválido: {e}")))?;
    let current_obj = current_env.as_object().cloned().unwrap_or_default();

    let mut merged = Map::new();
    for (k, v) in &current_obj {
        let is_sentinel = v
            .as_object()
            .and_then(|o| o.get(SENTINEL_SECTION_KEY))
            .is_some();
        if is_sentinel {
            merged.insert(k.clone(), v.clone());
        } else if let Some(new_val) = payload.get(k) {
            merged.insert(k.clone(), new_val.clone());
        }
    }
    for (k, v) in payload {
        if def.sections.contains(&k.as_str()) {
            return Err(AppError::Validation(format!(
                "A chave '{k}' é uma seção de tabela e não pode ser sobrescrita pelos metadados."
            )));
        }
        if !merged.contains_key(k) {
            merged.insert(k.clone(), v.clone());
        }
    }

    conn.execute(
        "INSERT INTO egressos_dados_files (file_key, kind, primary_section, envelope_json) \
         VALUES (?1, ?2, ?3, ?4) \
         ON CONFLICT(file_key) DO UPDATE SET envelope_json = excluded.envelope_json",
        params![
            def.file_key,
            def.kind.as_str(),
            def.primary_section,
            Json::Object(merged).to_string()
        ],
    )?;

    get_envelope(conn, file_key)
}

/// Utilitário público para criar os 39 arquivos JSON sintéticos em um diretório temporário
/// (`alunos.json` na raiz + 38 em `data/*.json`), utilizado em testes unitários e de integração.
pub fn write_synthetic_egressos_dados_repo(repo_root: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(repo_root.join("data"))?;
    for def in EGRESSOS_DADOS_FILES {
        let val = match def.kind {
            FileKind::RootArray => {
                if is_scalar_list_section(def.file_key, "_root") {
                    serde_json::json!(["Accenture Brasil", "Metris Energy"])
                } else {
                    let mut item = Map::new();
                    for &c in def.columns {
                        if c == "id" {
                            item.insert("id".into(), Json::String("egresso_demo_1".into()));
                        } else if c == "nome" {
                            item.insert("nome".into(), Json::String("Egresso Demo 1".into()));
                        } else {
                            item.insert(c.into(), Json::String(format!("val_{c}")));
                        }
                    }
                    Json::Array(vec![Json::Object(item)])
                }
            }
            FileKind::RootMap => {
                if def.file_key == "genero_map" {
                    serde_json::json!({
                        "egresso_demo_1": "M",
                        "egresso_demo_2": "F"
                    })
                } else {
                    serde_json::json!({
                        "Empresa Demo": {
                            "empresa": "Empresa Demo",
                            "porte": "Média",
                            "origem": "Internacional",
                            "setor": "Tecnologia"
                        }
                    })
                }
            }
            FileKind::NestedMap => {
                let mut root = Map::new();
                root.insert(
                    "titulo".into(),
                    Json::String(format!("Título {}", def.title)),
                );
                for &sec in def.sections {
                    let mut inner = Map::new();
                    let mut entry = Map::new();
                    entry.insert("nome".into(), Json::String(format!("Item {sec}")));
                    inner.insert("bsi".into(), Json::Object(entry));
                    root.insert(sec.into(), Json::Object(inner));
                }
                Json::Object(root)
            }
            FileKind::ObjectSections => {
                let mut root = Map::new();
                root.insert(
                    "fonte".into(),
                    Json::String(format!("Fonte {}", def.title)),
                );
                for &sec in def.sections {
                    if is_scalar_list_section(def.file_key, sec) {
                        root.insert(sec.into(), serde_json::json!(["item_1", "item_2"]));
                    } else if matches!(
                        sec,
                        "cursos"
                            | "convencoes"
                            | "papeis_agregado"
                            | "tipos_agregado"
                            | "detalhe_fonte"
                            | "genero"
                            | "funcoes"
                            | "projetos_por_egresso"
                    ) && sec != def.primary_section
                    {
                        root.insert(sec.into(), serde_json::json!({"chave_1": "valor_1"}));
                    } else {
                        let mut item = Map::new();
                        for &c in def.columns {
                            if c == "id" || c == "label" || c == "rotulo" {
                                item.insert(c.into(), Json::String("egresso_demo_1".into()));
                            } else if c == "perfil" {
                                item.insert(c.into(), Json::String("Perfil egresso_demo_1".into()));
                            } else if c == "nome" {
                                item.insert(c.into(), Json::String("Egresso Demo 1".into()));
                            } else {
                                item.insert(c.into(), Json::String(format!("val_{c}")));
                            }
                        }
                        root.insert(sec.into(), Json::Array(vec![Json::Object(item)]));
                    }
                }
                Json::Object(root)
            }
        };
        let mut text = serde_json::to_string_pretty(&val).unwrap();
        text.push('\n');
        let full = repo_root.join(def.relative_path);
        if let Some(p) = full.parent() {
            std::fs::create_dir_all(p)?;
        }
        std::fs::write(full, text)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use tempfile::TempDir;

    #[test]
    fn catalog_defines_all_39_canonical_files_with_roles_and_publications() {
        assert_eq!(EGRESSOS_DADOS_FILES.len(), 39);
        let pii_count = EGRESSOS_DADOS_FILES
            .iter()
            .filter(|f| f.publication == PublicationKind::Pii)
            .count();
        let interno_count = EGRESSOS_DADOS_FILES
            .iter()
            .filter(|f| f.publication == PublicationKind::Interno)
            .count();
        let pub_count = EGRESSOS_DADOS_FILES
            .iter()
            .filter(|f| f.publication == PublicationKind::Publicavel)
            .count();
        assert_eq!(pii_count, 12, "esperados 12 datasets PII");
        assert_eq!(interno_count, 6, "esperados 6 datasets internos");
        assert_eq!(pub_count, 21, "esperados 21 datasets publicáveis");

        let alunos = by_file_key("alunos").unwrap();
        assert_eq!(alunos.relative_path, "alunos.json");
        assert_eq!(alunos.primary_section, "alunos");
        assert_eq!(alunos.sections, &["alunos", "cursos", "convencoes"]);

        for def in EGRESSOS_DADOS_FILES {
            if def.file_key != "alunos" {
                assert!(
                    def.relative_path.starts_with("data/"),
                    "dataset {} deveria residir em data/",
                    def.file_key
                );
            }
            assert!(
                def.sections.contains(&def.primary_section),
                "dataset {}: primary_section '{}' deve constar em sections",
                def.file_key,
                def.primary_section
            );
        }
    }

    #[test]
    fn synthetic_import_and_export_round_trip_preserves_all_39_files_and_4_shapes() {
        let mut conn = db::open_in_memory().unwrap();
        let src_dir = TempDir::new().unwrap();
        let app_dir = TempDir::new().unwrap();
        let out_dir = TempDir::new().unwrap();

        write_synthetic_egressos_dados_repo(src_dir.path()).unwrap();

        // Também valida que passar a subpasta `/data` resolve automaticamente a raiz!
        let summary = import_dir(
            &mut conn,
            &src_dir.path().join("data"),
            app_dir.path(),
            |_| {},
        )
        .unwrap();
        assert_eq!(summary.total_files, 39);
        assert!(summary.total_rows >= 39);
        assert!(!summary.replaced);

        // Segunda importação deve gerar snapshot egressos-dados-pre-import
        let summary2 = import_dir(&mut conn, src_dir.path(), app_dir.path(), |_| {}).unwrap();
        assert!(summary2.replaced);
        assert!(summary2
            .snapshot
            .as_deref()
            .unwrap_or_default()
            .starts_with("egressos-dados-pre-import"));

        let exp = export_dir(&conn, out_dir.path()).unwrap();
        assert_eq!(exp.total_files, 39);

        for def in EGRESSOS_DADOS_FILES {
            let orig_str = std::fs::read_to_string(src_dir.path().join(def.relative_path)).unwrap();
            let exp_str = std::fs::read_to_string(out_dir.path().join(def.relative_path)).unwrap();
            let orig_json: Json = serde_json::from_str(&orig_str).unwrap();
            let exp_json: Json = serde_json::from_str(&exp_str).unwrap();
            assert_eq!(
                orig_json, exp_json,
                "divergência round-trip no arquivo '{}'",
                def.relative_path
            );
        }
    }

    #[test]
    fn real_egressos_dados_repository_round_trip_matches_100_percent_when_present() {
        let real_root = Path::new(DEFAULT_LOCAL_REPO_DIR);
        if !real_root.join("alunos.json").is_file() {
            return;
        }

        let mut conn = db::open_in_memory().unwrap();
        let app_dir = TempDir::new().unwrap();
        let out_dir = TempDir::new().unwrap();

        let summary = import_dir(&mut conn, real_root, app_dir.path(), |_| {}).unwrap();
        assert_eq!(summary.total_files, 39);
        assert!(summary.total_rows > 1500);

        export_dir(&conn, out_dir.path()).unwrap();

        for def in EGRESSOS_DADOS_FILES {
            let orig_str = std::fs::read_to_string(real_root.join(def.relative_path)).unwrap();
            let exp_str = std::fs::read_to_string(out_dir.path().join(def.relative_path)).unwrap();
            let orig_json: Json = serde_json::from_str(&orig_str).unwrap();
            let exp_json: Json = serde_json::from_str(&exp_str).unwrap();
            assert_eq!(
                orig_json, exp_json,
                "SC-002: divergência semântica no arquivo real '{}'",
                def.relative_path
            );
        }
    }

    #[test]
    fn crud_preserves_textual_id_in_alunos_and_propagates_across_related_tables() {
        let mut conn = db::open_in_memory().unwrap();
        let src_dir = TempDir::new().unwrap();
        let app_dir = TempDir::new().unwrap();
        write_synthetic_egressos_dados_repo(src_dir.path()).unwrap();
        import_dir(&mut conn, src_dir.path(), app_dir.path(), |_| {}).unwrap();

        let page = list_rows(&conn, "alunos", Some("alunos"), Some(50), Some(0), None, None, None).unwrap();
        assert_eq!(page.total, 1);
        let first = &page.items[0];
        // O campo "id" textual ("egresso_demo_1") deve estar preservado!
        assert_eq!(first.get("id").and_then(|v| v.as_str()), Some("egresso_demo_1"));
        let row_id = first.get("_row_id").and_then(|v| v.as_i64()).expect("_row_id numérico");

        // Atualiza com propagate = true e verifica propagação para _novos_reais e egressos_perfil
        let mut upd_payload = Record::new();
        upd_payload.insert("id".into(), Json::String("egresso_demo_1".into()));
        upd_payload.insert("cargo_atual".into(), Json::String("Principal Engineer".into()));
        upd_payload.insert("empresa_atual".into(), Json::String("LabTech".into()));

        let res = update_row(&mut conn, row_id, &upd_payload, true).unwrap();
        assert_eq!(
            res.record.get("cargo_atual").and_then(|v| v.as_str()),
            Some("Principal Engineer")
        );
        assert_eq!(res.record.get("id").and_then(|v| v.as_str()), Some("egresso_demo_1"));
        assert!(res.propagated_count >= 2);

        let novos_page = list_rows(&conn, "_novos_reais", None, None, None, None, None, None).unwrap();
        assert_eq!(
            novos_page.items[0].get("cargo_atual").and_then(|v| v.as_str()),
            Some("Principal Engineer")
        );

        // Cria um novo aluno em alunos.json preservando o id textual
        let mut create_payload = Record::new();
        create_payload.insert("id".into(), Json::String("novo_aluno".into()));
        create_payload.insert("nome".into(), Json::String("Novo Aluno IFES".into()));
        create_payload.insert("curso".into(), Json::String("bsi".into()));
        let created = create_row(&conn, "alunos", Some("alunos"), &create_payload).unwrap();
        assert_eq!(created.get("id").and_then(|v| v.as_str()), Some("novo_aluno"));
        let created_row_id = created.get("_row_id").and_then(|v| v.as_i64()).unwrap();

        // Testa list_rows_filtered (SEP-041 T006)
        let filter_bsi = r#"{"logic":"and","rules":[{"field":"curso","operator":"equals","value":"bsi"}]}"#;
        let filtered_bsi = list_rows_filtered(
            &conn,
            "alunos",
            Some("alunos"),
            Some(50),
            Some(0),
            None,
            None,
            None,
            Some(filter_bsi),
        )
        .unwrap();
        assert!(filtered_bsi.total >= 1);

        // Exclui o novo aluno
        let del = delete_row(&mut conn, created_row_id, false).unwrap();
        assert_eq!(del.deleted, 1);
    }
}
