//! IPC surface (SEP-017). Thin: validate, delegate, return. No logic here.
//!
//! `#[tauri::command(async)]` runs these synchronous bodies off the main thread,
//! which is what makes the blocking rusqlite driver comfortable (AD-08).

use crate::crud::{self, Page, Record};
use crate::error::AppError;
use crate::state::AppState;
use tauri::State;

#[tauri::command(async)]
#[allow(clippy::too_many_arguments)]
pub fn list_entities(
    state: State<'_, AppState>,
    entity: String,
    limit: Option<i64>,
    offset: Option<i64>,
    search: Option<String>,
    sort: Option<String>,
    order: Option<String>,
    filters: Option<String>,
) -> Result<Page, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crud::list_filtered(
        &conn,
        &entity,
        limit,
        offset,
        search.as_deref(),
        sort.as_deref(),
        order.as_deref(),
        filters.as_deref(),
    )
}

#[tauri::command(async)]
pub fn get_entity(state: State<'_, AppState>, entity: String, id: i64) -> Result<Record, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crud::get(&conn, &entity, id)
}

#[tauri::command(async)]
pub fn create_entity(
    state: State<'_, AppState>,
    entity: String,
    payload: Record,
) -> Result<Record, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crud::create(&conn, &entity, &payload)
}

#[tauri::command(async)]
pub fn update_entity(
    state: State<'_, AppState>,
    entity: String,
    id: i64,
    payload: Record,
) -> Result<Record, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crud::update(&conn, &entity, id, &payload)
}

/// Returns unit, which reaches JavaScript as `null`. That is what removes the
/// spurious error dialog on every successful delete (FR-013): the current
/// `apiFetch` parses a 204 empty body and throws before checking the status.
#[tauri::command(async)]
pub fn delete_entity(state: State<'_, AppState>, entity: String, id: i64) -> Result<(), AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crud::delete(&conn, &entity, id)
}

// ------------------------------------------------ Horizon v2 CRUD (SEP-039)

#[tauri::command(async)]
#[allow(clippy::too_many_arguments)]
pub fn hv2_list_entities(
    state: State<'_, AppState>,
    entity: String,
    limit: Option<i64>,
    offset: Option<i64>,
    search: Option<String>,
    sort: Option<String>,
    order: Option<String>,
    filters: Option<String>,
) -> Result<Page, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crud::hv2_list_filtered(
        &conn,
        &entity,
        limit,
        offset,
        search.as_deref(),
        sort.as_deref(),
        order.as_deref(),
        filters.as_deref(),
    )
}

#[tauri::command(async)]
pub fn hv2_get_entity(
    state: State<'_, AppState>,
    entity: String,
    id: i64,
) -> Result<Record, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crud::hv2_get(&conn, &entity, id)
}

#[tauri::command(async)]
pub fn hv2_create_entity(
    state: State<'_, AppState>,
    entity: String,
    payload: Record,
) -> Result<Record, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crud::hv2_create(&conn, &entity, &payload)
}

#[tauri::command(async)]
pub fn hv2_update_entity(
    state: State<'_, AppState>,
    entity: String,
    id: i64,
    payload: Record,
) -> Result<Record, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crud::hv2_update(&conn, &entity, id, &payload)
}

#[tauri::command(async)]
pub fn hv2_delete_entity(
    state: State<'_, AppState>,
    entity: String,
    id: i64,
) -> Result<(), AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crud::hv2_delete(&conn, &entity, id)
}

#[derive(serde::Serialize)]
pub struct LoginResult {
    /// Kept only so the existing LoginForm keeps working unchanged; it carries
    /// no cryptographic meaning in a local app. Real sessions arrive in SEP-021.
    pub access_token: String,
    pub token_type: String,
}

#[tauri::command(async)]
pub fn login(
    state: State<'_, AppState>,
    email: String,
    password: String,
) -> Result<LoginResult, AppError> {
    {
        let conn = state.db()?;
        crate::auth::verify_credentials(&conn, &email, &password)?;
    }
    state.open_session(&email)?;
    Ok(LoginResult {
        access_token: "local-session".into(),
        token_type: "bearer".into(),
    })
}

#[tauri::command(async)]
pub fn register(
    state: State<'_, AppState>,
    email: String,
    password: String,
    password_confirm: String,
) -> Result<(), AppError> {
    let conn = state.db()?;
    crate::auth::register_admin(&conn, &email, &password, &password_confirm)
}

// ---------------------------------------------------------------- data import


/// Import the canonical archive (SEP-018).
///
/// The dialog is opened here, in Rust (decision Q1), so the frontend needs no
/// filesystem permission at all. `path` is accepted for the drag-and-drop path,
/// where the window already knows the file.
#[tauri::command(async)]
pub fn import_canonical_zip(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    path: Option<String>,
) -> Result<Option<crate::import::ImportSummary>, AppError> {
    use tauri::Emitter;
    use tauri_plugin_dialog::DialogExt;

    let chosen = match path {
        Some(p) => std::path::PathBuf::from(p),
        None => {
            let picked = app
                .dialog()
                .file()
                .add_filter("Pacote canônico", &["zip"])
                .blocking_pick_file();
            // Dismissing the dialog is a no-op, not an error (FR-001).
            match picked {
                Some(f) => f.into_path().map_err(|e| AppError::Internal(e.to_string()))?,
                None => return Ok(None),
            }
        }
    };

    if !crate::import::looks_like_zip(&chosen) {
        return Err(AppError::Internal(
            "Apenas arquivos .zip contendo .parquet são aceitos.".into(),
        ));
    }

    let bytes = std::fs::read(&chosen)?;
    let data_dir = app_data_dir(&app)?;

    // Own connection: the import runs for seconds and must not hold the lock
    // the interface uses (FR-013).
    state.require_session()?;
    let mut conn = state.etl_connection()?;
    let summary = crate::import::import_archive(&mut conn, &bytes, &data_dir, |t| {
        let _ = app.emit("import://progress", t);
    })?;

    Ok(Some(summary))
}

fn app_data_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, AppError> {
    use tauri::Manager;
    app.path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("diretório de dados indisponível: {e}")))
}

/// Export the curated base (SEP-019). Save dialog opened in Rust (Q1).
#[tauri::command(async)]
pub fn export_canonical_zip(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<crate::export::ExportSummary>, AppError> {
    use tauri::Emitter;
    use tauri_plugin_dialog::DialogExt;

    let target = match app
        .dialog()
        .file()
        .set_file_name(crate::export::DEFAULT_FILENAME)
        .add_filter("Pacote canônico", &["zip"])
        .blocking_save_file()
    {
        Some(f) => f.into_path().map_err(|e| AppError::Internal(e.to_string()))?,
        None => return Ok(None),
    };

    let data_dir = app_data_dir(&app)?;
    let original = std::fs::read(data_dir.join(crate::import::ORIGINAL_ARCHIVE)).ok();

    state.require_session()?;
    let conn = state.etl_connection()?;
    let (bytes, tables, preserved) =
        crate::export::build_archive(&conn, original.as_deref(), |t| {
            let _ = app.emit("export://progress", t);
        })?;

    std::fs::write(&target, &bytes)?;

    Ok(Some(crate::export::ExportSummary {
        tables,
        preserved_entries: preserved,
        path: target.display().to_string(),
    }))
}

/// Import the canonical archive into the isolated Horizon v2 workspace (`hv2_*` tables) (SEP-039).
#[tauri::command(async)]
pub fn import_horizon_v2_zip(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    path: Option<String>,
) -> Result<Option<crate::import::ImportSummary>, AppError> {
    use tauri::Emitter;
    use tauri_plugin_dialog::DialogExt;

    let chosen = match path {
        Some(p) => std::path::PathBuf::from(p),
        None => {
            let picked = app
                .dialog()
                .file()
                .add_filter("Pacote canônico", &["zip"])
                .blocking_pick_file();
            match picked {
                Some(f) => f.into_path().map_err(|e| AppError::Internal(e.to_string()))?,
                None => return Ok(None),
            }
        }
    };

    if !crate::import::looks_like_zip(&chosen) {
        return Err(AppError::Internal(
            "Apenas arquivos .zip contendo dados canônicos são aceitos.".into(),
        ));
    }

    let bytes = std::fs::read(&chosen)?;
    let data_dir = app_data_dir(&app)?;

    state.require_session()?;
    let mut conn = state.etl_connection()?;
    let summary = crate::import::import_horizon_v2_archive(&mut conn, &bytes, &data_dir, |t| {
        let _ = app.emit("horizon-v2-import://progress", t);
    })?;

    Ok(Some(summary))
}

/// Export the curated Horizon v2 base (`hv2_*` tables) preserving `horizon_v2_original.zip` entries (SEP-039).
#[tauri::command(async)]
pub fn export_horizon_v2_zip(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<crate::export::ExportSummary>, AppError> {
    use tauri::Emitter;
    use tauri_plugin_dialog::DialogExt;

    let target = match app
        .dialog()
        .file()
        .set_file_name(crate::export::DEFAULT_FILENAME)
        .add_filter("Pacote canônico", &["zip"])
        .blocking_save_file()
    {
        Some(f) => f.into_path().map_err(|e| AppError::Internal(e.to_string()))?,
        None => return Ok(None),
    };

    let data_dir = app_data_dir(&app)?;
    let original = std::fs::read(data_dir.join(crate::import::HORIZON_V2_ORIGINAL_ARCHIVE)).ok();

    state.require_session()?;
    let conn = state.etl_connection()?;
    let (bytes, tables, preserved) =
        crate::export::build_horizon_v2_archive(&conn, original.as_deref(), |t| {
            let _ = app.emit("horizon-v2-export://progress", t);
        })?;

    std::fs::write(&target, &bytes)?;

    Ok(Some(crate::export::ExportSummary {
        tables,
        preserved_entries: preserved,
        path: target.display().to_string(),
    }))
}

// ------------------------------------------------------------------- SRC (033)

/// Import the SRC consolidated JSON (SEP-033, FR-006..009). Dialog opened in
/// Rust like the canonical import; own connection so the UI lock stays free.
#[tauri::command(async)]
pub fn import_src_json(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    path: Option<String>,
) -> Result<Option<crate::src_domain::SrcImportSummary>, AppError> {
    use tauri::Emitter;
    use tauri_plugin_dialog::DialogExt;

    let chosen = match path {
        Some(p) => std::path::PathBuf::from(p),
        None => {
            let picked = app
                .dialog()
                .file()
                .add_filter("Consolidado do SRC", &["json"])
                .blocking_pick_file();
            match picked {
                Some(f) => f.into_path().map_err(|e| AppError::Internal(e.to_string()))?,
                None => return Ok(None),
            }
        }
    };

    let text = std::fs::read_to_string(&chosen)?;
    let data_dir = app_data_dir(&app)?;

    state.require_session()?;
    let _ = app.emit("src-import://progress", "carregando consolidado");
    let mut conn = state.etl_connection()?;
    let summary = crate::src_domain::import(&mut conn, &text, &data_dir)?;
    let _ = app.emit("src-import://progress", "importação concluída");

    Ok(Some(summary))
}

/// Export the SRC base as a consolidated JSON (SEP-033, FR-014). Same dialog
/// discipline: save dialog in Rust, front-end never touches the filesystem.
#[tauri::command(async)]
pub fn export_src_json(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<crate::src_domain::SrcExportSummary>, AppError> {
    use tauri_plugin_dialog::DialogExt;

    let target = match app
        .dialog()
        .file()
        .set_file_name("src_consolidado.json")
        .add_filter("Consolidado do SRC", &["json"])
        .blocking_save_file()
    {
        Some(f) => f.into_path().map_err(|e| AppError::Internal(e.to_string()))?,
        None => return Ok(None),
    };

    state.require_session()?;
    let conn = state.etl_connection()?;
    let json = crate::src_domain::export(&conn)?;
    std::fs::write(&target, json)?;

    let counts = crate::src_domain::export_summary(&conn)?;
    Ok(Some(crate::src_domain::SrcExportSummary {
        total_acoes: counts.0,
        total_participacoes: counts.1,
        path: target.display().to_string(),
    }))
}

// SEP-033 / US3: curadoria das ações e participações. Mesma disciplina do
// CRUD genérico (validar, delegar, retornar), porém sempre nos comandos
// dedicados — o caminho do Horizon fica invariante (R6).

#[tauri::command(async)]
pub fn src_list_acoes(
    state: State<'_, AppState>,
    limit: Option<i64>,
    offset: Option<i64>,
    search: Option<String>,
    sort: Option<String>,
    order: Option<String>,
    filters: Option<String>,
) -> Result<crate::src_domain::SrcAcaoPage, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::src_domain::list_acoes_filtered(
        &conn,
        limit,
        offset,
        search.as_deref(),
        sort.as_deref(),
        order.as_deref(),
        filters.as_deref(),
    )
}

#[tauri::command(async)]
pub fn src_get_acao(state: State<'_, AppState>, id: i64) -> Result<crate::crud::Record, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::src_domain::get_acao(&conn, id)
}

#[tauri::command(async)]
pub fn src_create_acao(
    state: State<'_, AppState>,
    payload: crate::crud::Record,
) -> Result<i64, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::src_domain::create_acao(&conn, &payload)
}

#[tauri::command(async)]
pub fn src_update_acao(
    state: State<'_, AppState>,
    id: i64,
    payload: crate::crud::Record,
) -> Result<(), AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::src_domain::update_acao(&conn, id, &payload)
}

#[tauri::command(async)]
pub fn src_delete_acao(
    state: State<'_, AppState>,
    id: i64,
    force: Option<bool>,
) -> Result<(), AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::src_domain::delete_acao(&conn, id, force.unwrap_or(false))
}

#[tauri::command(async)]
pub fn src_list_participacoes(
    state: State<'_, AppState>,
    acao_id: i64,
) -> Result<Vec<crate::crud::Record>, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::src_domain::list_participacoes(&conn, acao_id)
}

#[tauri::command(async)]
pub fn src_create_participacao(
    state: State<'_, AppState>,
    acao_id: i64,
    payload: crate::crud::Record,
) -> Result<i64, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::src_domain::create_participacao(&conn, acao_id, &payload)
}

#[tauri::command(async)]
pub fn src_update_participacao(
    state: State<'_, AppState>,
    id: i64,
    payload: crate::crud::Record,
) -> Result<(), AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::src_domain::update_participacao(&conn, id, &payload)
}

#[tauri::command(async)]
pub fn src_delete_participacao(
    state: State<'_, AppState>,
    id: i64,
) -> Result<(), AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::src_domain::delete_participacao(&conn, id)
}

#[tauri::command(async)]
pub fn src_get_meta(state: State<'_, AppState>) -> Result<crate::crud::Record, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::src_domain::get_meta(&conn)
}

#[tauri::command(async)]
pub fn src_update_meta(
    state: State<'_, AppState>,
    campus: Option<String>,
) -> Result<(), AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::src_domain::update_meta(&conn, campus.as_deref())
}

// -------------------------------------------------------------- Egressos (035)

#[tauri::command(async)]
pub fn import_egressos_dir(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    path: Option<String>,
) -> Result<Option<crate::egressos_domain::EgressosImportSummary>, AppError> {
    use tauri::Emitter;
    use tauri_plugin_dialog::DialogExt;

    state.require_session()?;

    let chosen = match path.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(p) => std::path::PathBuf::from(p),
        None => match app.dialog().file().blocking_pick_folder() {
            Some(f) => f.into_path().map_err(|e| AppError::Internal(e.to_string()))?,
            None => return Ok(None),
        },
    };

    let data_dir = app_data_dir(&app)?;
    let mut conn = state.etl_connection()?;
    let summary = crate::egressos_domain::import_dir(&mut conn, &chosen, &data_dir, |p| {
        let _ = app.emit("egressos-import://progress", p);
    })?;

    Ok(Some(summary))
}

#[tauri::command(async)]
pub fn export_egressos_dir(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    path: Option<String>,
) -> Result<Option<crate::egressos_domain::EgressosExportSummary>, AppError> {
    use tauri_plugin_dialog::DialogExt;

    state.require_session()?;

    let target = match path.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(p) => std::path::PathBuf::from(p),
        None => match app.dialog().file().blocking_pick_folder() {
            Some(f) => f.into_path().map_err(|e| AppError::Internal(e.to_string()))?,
            None => return Ok(None),
        },
    };

    let conn = state.etl_connection()?;
    let summary = crate::egressos_domain::export_dir(&conn, &target)?;
    Ok(Some(summary))
}

#[tauri::command(async)]
pub fn egressos_get_catalog(
    state: State<'_, AppState>,
) -> Result<crate::egressos_domain::EgressosCatalogView, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::egressos_domain::get_catalog(&conn)
}

#[tauri::command(async)]
#[allow(clippy::too_many_arguments)]
pub fn egressos_list_rows(
    state: State<'_, AppState>,
    file_key: String,
    section_key: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
    search: Option<String>,
    sort: Option<String>,
    order: Option<String>,
    filters: Option<String>,
) -> Result<crate::egressos_domain::EgressosRowPage, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::egressos_domain::list_rows_filtered(
        &conn,
        &file_key,
        section_key.as_deref(),
        limit,
        offset,
        search.as_deref(),
        sort.as_deref(),
        order.as_deref(),
        filters.as_deref(),
    )
}

#[tauri::command(async)]
pub fn egressos_get_row(
    state: State<'_, AppState>,
    id: i64,
) -> Result<crate::crud::Record, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::egressos_domain::get_row(&conn, id)
}

#[tauri::command(async)]
pub fn egressos_create_row(
    state: State<'_, AppState>,
    file_key: String,
    section_key: Option<String>,
    payload: crate::crud::Record,
) -> Result<crate::crud::Record, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::egressos_domain::create_row(&conn, &file_key, section_key.as_deref(), &payload)
}

#[tauri::command(async)]
pub fn egressos_update_row(
    state: State<'_, AppState>,
    id: i64,
    payload: crate::crud::Record,
    propagate: Option<bool>,
) -> Result<crate::egressos_domain::EgressosMutationResult, AppError> {
    state.require_session()?;
    let mut conn = state.db()?;
    crate::egressos_domain::update_row(&mut conn, id, &payload, propagate.unwrap_or(false))
}

#[tauri::command(async)]
pub fn egressos_delete_row(
    state: State<'_, AppState>,
    id: i64,
    propagate: Option<bool>,
) -> Result<crate::egressos_domain::EgressosDeleteResult, AppError> {
    state.require_session()?;
    let mut conn = state.db()?;
    crate::egressos_domain::delete_row(&mut conn, id, propagate.unwrap_or(false))
}

#[tauri::command(async)]
pub fn egressos_get_envelope(
    state: State<'_, AppState>,
    file_key: String,
) -> Result<crate::crud::Record, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::egressos_domain::get_envelope(&conn, &file_key)
}

#[tauri::command(async)]
pub fn egressos_update_envelope(
    state: State<'_, AppState>,
    file_key: String,
    payload: crate::crud::Record,
) -> Result<crate::crud::Record, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::egressos_domain::update_envelope(&conn, &file_key, &payload)
}

// ----------------------------------------------------------- special ops (020)

// ------------------------------------------------------- GitHub sync (034)
//
// Toda a conversa com github.com acontece no núcleo (Princípio VI); aqui só
// orquestramos: sessão, config local (sync_domain), cliente HTTP
// (sync_github) e os imports/exports já existentes — o ritual de segurança
// do import manual é o MESMO, só muda a origem dos bytes (FR-003).

#[tauri::command(async)]
pub fn github_get_config(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    project: String,
) -> Result<crate::sync_domain::SyncConfigView, AppError> {
    state.require_session()?;
    let file = crate::sync_domain::load_config(&app_data_dir(&app)?)?;
    crate::sync_domain::config_view(&project, &file)
}

#[tauri::command(async)]
pub async fn github_download(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    project: String,
    url: String,
) -> Result<serde_json::Value, AppError> {
    use tauri::Emitter;
    state.require_session()?;

    let data_dir = app_data_dir(&app)?;
    let config = crate::sync_domain::load_config(&data_dir)?;
    let client = crate::sync_github::GitHubClient::new(
        crate::sync_github::GITHUB_API,
        config.token,
    );
    let mut conn = state.etl_connection()?;

    let result = if project == "egressos" {
        let dir_source = crate::sync_domain::normalize_egressos_source(&url)?;
        crate::sync_github::download_egressos_dir(
            &mut conn,
            &data_dir,
            &dir_source,
            &client,
            &|p| {
                let _ = app.emit("sync://progress", p);
            },
        )
        .await
        .and_then(|summary| {
            serde_json::to_value(summary)
                .map_err(|e| AppError::Internal(format!("resumo ilegível: {e}")))
        })
    } else {
        let source = crate::sync_domain::normalize_url(&url)?;
        crate::sync_github::download_and_import(
            &mut conn,
            &data_dir,
            &project,
            &source,
            &client,
            &|p| {
                let _ = app.emit("sync://progress", p);
            },
        )
        .await
    };

    let _ = app.emit(
        "sync://progress",
        crate::sync_domain::SyncProgress {
            operation: "download",
            project: project.clone(),
            phase: if result.is_ok() { "done" } else { "failed" },
            bytes_done: 0,
            bytes_total: None,
        },
    );
    result
}

#[tauri::command(async)]
pub fn github_set_config(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    project: String,
    repo: String,
    branch: String,
    path: String,
) -> Result<(), AppError> {
    state.require_session()?;
    crate::sync_domain::set_project_config(&app_data_dir(&app)?, &project, &repo, &branch, &path)?;
    Ok(())
}

#[tauri::command(async)]
pub async fn github_check_destination(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    project: String,
) -> Result<crate::sync_domain::CheckDestinationView, AppError> {
    state.require_session()?;
    let data_dir = app_data_dir(&app)?;
    let config = crate::sync_domain::load_config(&data_dir)?;
    let client = crate::sync_github::GitHubClient::new(
        crate::sync_github::GITHUB_API,
        config.token,
    );
    crate::sync_github::check_project_destination(&data_dir, &project, &client).await
}

#[tauri::command(async)]
pub async fn github_list_branches(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    project: String,
    repo: Option<String>,
) -> Result<Vec<crate::sync_domain::RemoteBranchInfo>, AppError> {
    state.require_session()?;
    let data_dir = app_data_dir(&app)?;
    let config = crate::sync_domain::load_config(&data_dir)?;
    let client = crate::sync_github::GitHubClient::new(
        crate::sync_github::GITHUB_API,
        config.token,
    );
    crate::sync_github::list_project_branches(&data_dir, &project, repo.as_deref(), &client).await
}

#[tauri::command(async)]
#[allow(clippy::too_many_arguments)]
pub async fn github_upload(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    project: String,
    confirm_overwrite: Option<bool>,
    create_new_branch: Option<bool>,
    base_branch: Option<String>,
    new_branch: Option<String>,
    target_branch: Option<String>,
) -> Result<crate::sync_github::UploadResult, AppError> {
    use tauri::Emitter;
    state.require_session()?;
    let data_dir = app_data_dir(&app)?;
    let config = crate::sync_domain::load_config(&data_dir)?;
    let client = crate::sync_github::GitHubClient::new(
        crate::sync_github::GITHUB_API,
        config.token,
    );
    let branch_options = crate::sync_domain::UploadBranchOptions {
        create_new_branch: create_new_branch.unwrap_or(false),
        base_branch,
        new_branch,
        target_branch,
    };

    let result = if project == "egressos" {
        let mut conn = state.etl_connection()?;
        crate::sync_github::upload_egressos_dir_with_options(
            &mut conn,
            &data_dir,
            confirm_overwrite.unwrap_or(false),
            &branch_options,
            &client,
            &|p| {
                let _ = app.emit("sync://progress", p);
            },
        )
        .await
    } else if project == "src" {
        let bytes = {
            let conn = state.etl_connection()?;
            crate::src_domain::export(&conn)?.into_bytes()
        };

        crate::sync_github::upload_horizon_export_with_options(
            bytes,
            &data_dir,
            &project,
            confirm_overwrite.unwrap_or(false),
            &branch_options,
            &client,
            &|p| {
                let _ = app.emit("sync://progress", p);
            },
        )
        .await
    } else if project == "horizon_v2" {
        let bytes = {
            let conn = state.etl_connection()?;
            let original =
                std::fs::read(data_dir.join(crate::import::HORIZON_V2_ORIGINAL_ARCHIVE)).ok();
            let (b, _tables, _preserved) =
                crate::export::build_horizon_v2_archive(&conn, original.as_deref(), |_| {})?;
            b
        };

        crate::sync_github::upload_horizon_export_with_options(
            bytes,
            &data_dir,
            &project,
            confirm_overwrite.unwrap_or(false),
            &branch_options,
            &client,
            &|p| {
                let _ = app.emit("sync://progress", p);
            },
        )
        .await
    } else {
        let bytes = {
            let conn = state.etl_connection()?;
            let original = std::fs::read(data_dir.join(crate::import::ORIGINAL_ARCHIVE)).ok();
            let (b, _tables, _preserved) =
                crate::export::build_archive(&conn, original.as_deref(), |_| {})?;
            b
        };

        crate::sync_github::upload_horizon_export_with_options(
            bytes,
            &data_dir,
            &project,
            confirm_overwrite.unwrap_or(false),
            &branch_options,
            &client,
            &|p| {
                let _ = app.emit("sync://progress", p);
            },
        )
        .await
    };

    let _ = app.emit(
        "sync://progress",
        crate::sync_domain::SyncProgress {
            operation: "upload",
            project: project.clone(),
            phase: if result.is_ok() { "done" } else { "failed" },
            bytes_done: 0,
            bytes_total: None,
        },
    );

    result
}

#[tauri::command(async)]
pub fn github_save_token(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    token: String,
) -> Result<(), AppError> {
    state.require_session()?;
    let token = token.trim();
    if token.is_empty() {
        return Err(AppError::SyncConfig("Token de acesso não pode ficar vazio.".into()));
    }
    let data_dir = app_data_dir(&app)?;
    let mut file = crate::sync_domain::load_config(&data_dir)?;
    file.token = Some(token.to_string());
    crate::sync_domain::save_config(&data_dir, &file)?;
    Ok(())
}

#[tauri::command(async)]
pub async fn github_test_token(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<crate::sync_github::TokenInfo, AppError> {
    state.require_session()?;
    let data_dir = app_data_dir(&app)?;
    let file = crate::sync_domain::load_config(&data_dir)?;
    let token = file.token.ok_or_else(|| {
        AppError::SyncConfig("Nenhum token configurado.".into())
    })?;
    let client = crate::sync_github::GitHubClient::new(
        crate::sync_github::GITHUB_API,
        Some(token),
    );
    client.test_token().await
}

#[tauri::command(async)]
pub fn github_remove_token(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    state.require_session()?;
    let data_dir = app_data_dir(&app)?;
    let mut file = crate::sync_domain::load_config(&data_dir)?;
    file.token = None;
    crate::sync_domain::save_config(&data_dir, &file)?;
    Ok(())
}

// ------------------------------------------- Egressos-Dados Org Auth (SEP-036)

#[derive(serde::Serialize, Clone, Debug, PartialEq, Eq)]
pub struct EgressosDadosWorkspaceView {
    pub organization: String,
    pub login: String,
    pub token_hint: String,
    pub tables_ready: bool,
}

#[tauri::command(async)]
pub async fn egressos_dados_login(
    state: State<'_, AppState>,
    token: String,
) -> Result<crate::state::EgressosDadosAuthStatus, AppError> {
    state.require_session()?;
    let token = token.trim().to_string();
    if token.is_empty() {
        let _ = state.close_egressos_dados_session();
        return Err(AppError::Validation(
            "Informe o Token de Acesso Pessoal do GitHub.".into(),
        ));
    }

    let client = crate::sync_github::GitHubClient::new(
        crate::sync_github::GITHUB_API,
        Some(token.clone()),
    );
    match client
        .verify_org_membership(crate::state::REQUIRED_EGRESSOS_DADOS_ORG)
        .await
    {
        Ok(info) => state.open_egressos_dados_session(&info.login, &info.organization, &token),
        Err(err) => {
            let _ = state.close_egressos_dados_session();
            Err(err)
        }
    }
}

#[tauri::command(async)]
pub fn egressos_dados_status(
    state: State<'_, AppState>,
) -> Result<crate::state::EgressosDadosAuthStatus, AppError> {
    state.current_egressos_dados_status()
}

#[tauri::command(async)]
pub fn egressos_dados_logout(
    state: State<'_, AppState>,
) -> Result<crate::state::EgressosDadosAuthStatus, AppError> {
    state.require_session()?;
    state.close_egressos_dados_session()
}

#[tauri::command(async)]
pub fn egressos_dados_get_workspace(
    state: State<'_, AppState>,
) -> Result<EgressosDadosWorkspaceView, AppError> {
    let auth = state.require_egressos_dados_session()?;
    Ok(EgressosDadosWorkspaceView {
        organization: auth.organization,
        login: auth.login,
        token_hint: auth.token_hint,
        tables_ready: true,
    })
}

// ------------------------------------------- Egressos-Dados Tables & Sync (SEP-037)

#[tauri::command(async)]
pub fn egressos_dados_get_catalog(
    state: State<'_, AppState>,
) -> Result<crate::egressos_dados_domain::EgressosDadosCatalogView, AppError> {
    state.require_egressos_dados_session()?;
    let conn = state.db()?;
    crate::egressos_dados_domain::get_catalog(&conn)
}

#[tauri::command(async)]
#[allow(clippy::too_many_arguments)]
pub fn egressos_dados_list_rows(
    state: State<'_, AppState>,
    file_key: String,
    section_key: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
    search: Option<String>,
    sort: Option<String>,
    order: Option<String>,
    filters: Option<String>,
) -> Result<crate::egressos_dados_domain::EgressosDadosRowPage, AppError> {
    state.require_egressos_dados_session()?;
    let conn = state.db()?;
    crate::egressos_dados_domain::list_rows_filtered(
        &conn,
        &file_key,
        section_key.as_deref(),
        limit,
        offset,
        search.as_deref(),
        sort.as_deref(),
        order.as_deref(),
        filters.as_deref(),
    )
}

#[tauri::command(async)]
pub fn egressos_dados_get_row(
    state: State<'_, AppState>,
    id: i64,
) -> Result<crate::crud::Record, AppError> {
    state.require_egressos_dados_session()?;
    let conn = state.db()?;
    crate::egressos_dados_domain::get_row(&conn, id)
}

#[tauri::command(async)]
pub fn egressos_dados_create_row(
    state: State<'_, AppState>,
    file_key: String,
    section_key: Option<String>,
    payload: crate::crud::Record,
) -> Result<crate::crud::Record, AppError> {
    state.require_egressos_dados_session()?;
    let conn = state.db()?;
    crate::egressos_dados_domain::create_row(&conn, &file_key, section_key.as_deref(), &payload)
}

#[tauri::command(async)]
pub fn egressos_dados_update_row(
    state: State<'_, AppState>,
    id: i64,
    payload: crate::crud::Record,
    propagate: Option<bool>,
) -> Result<crate::egressos_dados_domain::EgressosDadosMutationResult, AppError> {
    state.require_egressos_dados_session()?;
    let mut conn = state.db()?;
    crate::egressos_dados_domain::update_row(&mut conn, id, &payload, propagate.unwrap_or(false))
}

#[tauri::command(async)]
pub fn egressos_dados_delete_row(
    state: State<'_, AppState>,
    id: i64,
    propagate: Option<bool>,
) -> Result<crate::egressos_dados_domain::EgressosDadosDeleteResult, AppError> {
    state.require_egressos_dados_session()?;
    let mut conn = state.db()?;
    crate::egressos_dados_domain::delete_row(&mut conn, id, propagate.unwrap_or(false))
}

#[tauri::command(async)]
pub fn egressos_dados_get_envelope(
    state: State<'_, AppState>,
    file_key: String,
) -> Result<crate::crud::Record, AppError> {
    state.require_egressos_dados_session()?;
    let conn = state.db()?;
    crate::egressos_dados_domain::get_envelope(&conn, &file_key)
}

#[tauri::command(async)]
pub fn egressos_dados_update_envelope(
    state: State<'_, AppState>,
    file_key: String,
    payload: crate::crud::Record,
) -> Result<crate::crud::Record, AppError> {
    state.require_egressos_dados_session()?;
    let conn = state.db()?;
    crate::egressos_dados_domain::update_envelope(&conn, &file_key, &payload)
}

#[tauri::command(async)]
pub fn egressos_dados_import_dir(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    path: Option<String>,
) -> Result<Option<crate::egressos_dados_domain::EgressosDadosImportSummary>, AppError> {
    use tauri::Emitter;
    use tauri_plugin_dialog::DialogExt;

    state.require_egressos_dados_session()?;

    let chosen = match path.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(p) => std::path::PathBuf::from(p),
        None => match app.dialog().file().blocking_pick_folder() {
            Some(f) => f.into_path().map_err(|e| AppError::Internal(e.to_string()))?,
            None => return Ok(None),
        },
    };

    let data_dir = app_data_dir(&app)?;
    let mut conn = state.etl_connection()?;
    let summary =
        crate::egressos_dados_domain::import_dir(&mut conn, &chosen, &data_dir, |p| {
            let _ = app.emit("egressos-dados-import://progress", p);
        })?;

    Ok(Some(summary))
}

#[tauri::command(async)]
pub fn egressos_dados_export_dir(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    path: Option<String>,
) -> Result<Option<crate::egressos_dados_domain::EgressosDadosExportSummary>, AppError> {
    use tauri_plugin_dialog::DialogExt;

    state.require_egressos_dados_session()?;

    let target = match path.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(p) => std::path::PathBuf::from(p),
        None => match app.dialog().file().blocking_pick_folder() {
            Some(f) => f.into_path().map_err(|e| AppError::Internal(e.to_string()))?,
            None => return Ok(None),
        },
    };

    let conn = state.etl_connection()?;
    let summary = crate::egressos_dados_domain::export_dir(&conn, &target)?;
    Ok(Some(summary))
}

#[tauri::command(async)]
pub fn egressos_dados_list_snapshots(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<crate::egressos_dados_domain::EgressosDadosSnapshotInfo>, AppError> {
    state.require_egressos_dados_session()?;
    let data_dir = app_data_dir(&app)?;
    crate::egressos_dados_domain::list_snapshots(&data_dir)
}

#[tauri::command(async)]
pub fn egressos_dados_sync_get_config(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<crate::sync_domain::SyncConfigView, AppError> {
    let auth = state.require_egressos_dados_session()?;
    let file = crate::sync_domain::load_config(&app_data_dir(&app)?)?;
    let mut view = crate::sync_domain::config_view("egressos_dados", &file)?;
    view.has_token = true;
    view.token_hint = Some(auth.token_hint);
    Ok(view)
}

#[tauri::command(async)]
pub fn egressos_dados_sync_set_config(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    repo: String,
    branch: String,
    path: String,
) -> Result<crate::sync_domain::SyncConfigView, AppError> {
    let auth = state.require_egressos_dados_session()?;
    let data_dir = app_data_dir(&app)?;
    crate::sync_domain::set_project_config(
        &data_dir,
        "egressos_dados",
        &repo,
        &branch,
        &path,
    )?;
    let file = crate::sync_domain::load_config(&data_dir)?;
    let mut view = crate::sync_domain::config_view("egressos_dados", &file)?;
    view.has_token = true;
    view.token_hint = Some(auth.token_hint);
    Ok(view)
}

#[tauri::command(async)]
pub async fn egressos_dados_sync_download(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    source_url: Option<String>,
    repo: Option<String>,
    branch: Option<String>,
    path: Option<String>,
) -> Result<crate::egressos_dados_domain::EgressosDadosImportSummary, AppError> {
    use tauri::Emitter;
    let auth = state.require_egressos_dados_session()?;
    let data_dir = app_data_dir(&app)?;
    let file = crate::sync_domain::load_config(&data_dir)?;
    let cfg = crate::sync_domain::effective_project_config("egressos_dados", &file);

    let dir_source = if let Some(url) = source_url.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        crate::sync_domain::normalize_egressos_dados_source(url)?
    } else {
        let effective_repo = repo
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(&cfg.repo);
        let effective_branch = branch
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(&cfg.branch);
        let effective_path = path
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or(&cfg.path);
        let (owner, repo_name) = effective_repo
            .split_once('/')
            .ok_or_else(|| AppError::SyncConfig("Repositório deve estar no formato 'owner/name'.".into()))?;
        crate::sync_domain::EgressosDirSource {
            owner: owner.to_string(),
            repo: repo_name.to_string(),
            reference: effective_branch.to_string(),
            dir: effective_path.to_string(),
        }
    };

    let client = crate::sync_github::GitHubClient::new(
        crate::sync_github::GITHUB_API,
        Some(auth.token),
    );
    let mut conn = state.etl_connection()?;

    let result = crate::sync_github::download_egressos_dados_repo(
        &mut conn,
        &data_dir,
        &dir_source,
        &client,
        &|p| {
            let _ = app.emit("sync://progress", p);
        },
    )
    .await;

    let _ = app.emit(
        "sync://progress",
        crate::sync_domain::SyncProgress {
            operation: "download",
            project: "egressos_dados".into(),
            phase: if result.is_ok() { "done" } else { "failed" },
            bytes_done: 0,
            bytes_total: None,
        },
    );

    result
}

#[tauri::command(async)]
pub async fn egressos_dados_sync_list_branches(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    repo: Option<String>,
) -> Result<Vec<crate::sync_domain::RemoteBranchInfo>, AppError> {
    let auth = state.require_egressos_dados_session()?;
    let data_dir = app_data_dir(&app)?;
    let client = crate::sync_github::GitHubClient::new(
        crate::sync_github::GITHUB_API,
        Some(auth.token),
    );
    crate::sync_github::list_project_branches(
        &data_dir,
        "egressos_dados",
        repo.as_deref(),
        &client,
    )
    .await
}

#[tauri::command(async)]
#[allow(clippy::too_many_arguments)]
pub async fn egressos_dados_sync_upload(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    confirm_overwrite: Option<bool>,
    commit_message: Option<String>,
    create_new_branch: Option<bool>,
    base_branch: Option<String>,
    new_branch: Option<String>,
    target_branch: Option<String>,
) -> Result<crate::sync_github::UploadResult, AppError> {
    use tauri::Emitter;
    let auth = state.require_egressos_dados_session()?;
    let data_dir = app_data_dir(&app)?;
    let client = crate::sync_github::GitHubClient::new(
        crate::sync_github::GITHUB_API,
        Some(auth.token),
    );
    let mut conn = state.etl_connection()?;
    let branch_options = crate::sync_domain::UploadBranchOptions {
        create_new_branch: create_new_branch.unwrap_or(false),
        base_branch,
        new_branch,
        target_branch,
    };

    let result = crate::sync_github::upload_egressos_dados_repo_with_options(
        &mut conn,
        &data_dir,
        confirm_overwrite.unwrap_or(true),
        commit_message.as_deref(),
        &branch_options,
        &client,
        &|p| {
            let _ = app.emit("sync://progress", p);
        },
    )
    .await;

    let _ = app.emit(
        "sync://progress",
        crate::sync_domain::SyncProgress {
            operation: "upload",
            project: "egressos_dados".into(),
            phase: if result.is_ok() { "done" } else { "failed" },
            bytes_done: 0,
            bytes_total: None,
        },
    );

    result
}


// ----------------------------------------------------------- special ops (020)

#[tauri::command(async)]
pub fn merge_entities(
    state: State<'_, AppState>,
    entity: String,
    source_ids: Vec<i64>,
    resolved_data: crud::Record,
) -> Result<crud::Record, AppError> {
    state.require_session()?;
    let mut conn = state.etl_connection()?;
    crate::special::merge(&mut conn, &entity, &source_ids, &resolved_data)
}

#[tauri::command(async)]
pub fn link_entities(
    state: State<'_, AppState>,
    parent_type: String,
    parent_id: i64,
    child_type: String,
    child_id: i64,
) -> Result<crud::Record, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::special::link(&conn, &parent_type, parent_id, &child_type, child_id)
}

#[tauri::command(async)]
pub fn hv2_merge_entities(
    state: State<'_, AppState>,
    entity: String,
    source_ids: Vec<i64>,
    resolved_data: crud::Record,
) -> Result<crud::Record, AppError> {
    state.require_session()?;
    let mut conn = state.etl_connection()?;
    crate::special::hv2_merge(&mut conn, &entity, &source_ids, &resolved_data)
}

#[tauri::command(async)]
pub fn hv2_link_entities(
    state: State<'_, AppState>,
    parent_type: String,
    parent_id: i64,
    child_type: String,
    child_id: i64,
) -> Result<crud::Record, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::special::hv2_link(&conn, &parent_type, parent_id, &child_type, child_id)
}

#[tauri::command(async)]
pub fn logout(state: State<'_, AppState>) -> Result<(), AppError> {
    state.close_session()
}

#[derive(serde::Serialize)]
pub struct SessionStatus {
    pub authenticated: bool,
    pub username: Option<String>,
}

#[tauri::command(async)]
pub fn session_status(state: State<'_, AppState>) -> Result<SessionStatus, AppError> {
    let s = state.current_session()?;
    Ok(SessionStatus {
        authenticated: s.is_some(),
        username: s.map(|s| s.username),
    })
}

// --------------------------------------------------- SEP-040: Analytics & Status

#[tauri::command(async)]
pub fn projects_status(
    state: State<'_, AppState>,
) -> Result<Vec<crate::analytics::ProjectWorkspaceStatus>, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::analytics::compute_projects_status(&conn)
}

#[tauri::command(async)]
pub fn hv2_get_analytics(
    state: State<'_, AppState>,
    entity: Option<String>,
    filters: Option<String>,
) -> Result<crate::analytics::DashboardPayload, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::analytics::horizon_analytics_filtered(&conn, true, entity.as_deref(), filters.as_deref())
}

#[tauri::command(async)]
pub fn get_horizon_analytics(
    state: State<'_, AppState>,
    entity: Option<String>,
    filters: Option<String>,
) -> Result<crate::analytics::DashboardPayload, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::analytics::horizon_analytics_filtered(&conn, false, entity.as_deref(), filters.as_deref())
}

#[tauri::command(async)]
pub fn src_get_analytics(
    state: State<'_, AppState>,
    filters: Option<String>,
) -> Result<crate::analytics::DashboardPayload, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::analytics::src_analytics_filtered(&conn, filters.as_deref())
}

#[tauri::command(async)]
pub fn egressos_get_analytics(
    state: State<'_, AppState>,
    file_key: Option<String>,
    filters: Option<String>,
) -> Result<crate::analytics::DashboardPayload, AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::analytics::egressos_analytics_filtered(&conn, file_key.as_deref(), filters.as_deref())
}

#[tauri::command(async)]
pub fn egressos_dados_get_analytics(
    state: State<'_, AppState>,
    file_key: Option<String>,
    filters: Option<String>,
) -> Result<crate::analytics::DashboardPayload, AppError> {
    state.require_egressos_dados_session()?;
    let conn = state.db()?;
    crate::analytics::egressos_dados_analytics_filtered(&conn, file_key.as_deref(), filters.as_deref())
}

// --------------------------------------------------- SEP-041: Custom Charts

#[tauri::command(async)]
pub fn create_custom_chart(
    state: State<'_, AppState>,
    payload: crate::analytics::NewCustomChartInput,
) -> Result<crate::analytics::CustomDashboardChart, AppError> {
    if payload.scope.trim() == "egressos-dados" {
        state.require_egressos_dados_session()?;
    } else {
        state.require_session()?;
    }
    let conn = state.db()?;
    crate::analytics::create_custom_chart(&conn, &payload)
}

#[tauri::command(async)]
pub fn list_custom_charts(
    state: State<'_, AppState>,
    scope: String,
    entity: Option<String>,
) -> Result<Vec<crate::analytics::CustomDashboardChart>, AppError> {
    if scope.trim() == "egressos-dados" {
        state.require_egressos_dados_session()?;
    } else {
        state.require_session()?;
    }
    let conn = state.db()?;
    crate::analytics::list_custom_charts(&conn, &scope, entity.as_deref())
}

#[tauri::command(async)]
pub fn delete_custom_chart(
    state: State<'_, AppState>,
    id: i64,
) -> Result<(), AppError> {
    state.require_session()?;
    let conn = state.db()?;
    crate::analytics::delete_custom_chart(&conn, id)
}
