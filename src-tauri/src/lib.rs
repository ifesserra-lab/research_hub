pub mod auth;
pub mod commands;
pub mod crud;
pub mod egressos_domain;
pub mod export;
pub mod import;
pub mod json_io;
pub mod parquet_io;
pub mod db;
pub mod error;
pub mod registry;
pub mod special;
pub mod src_domain;
pub mod state;
pub mod sync_domain;
pub mod sync_github;

use tauri::Manager;

/// Em terminais abertos a partir de um pacote Snap (ex.: VS Code Snap no Ubuntu),
/// variáveis como `XDG_DATA_DIRS` e `GSETTINGS_SCHEMA_DIR` apontam para
/// `/snap/.../usr/share/glib-2.0/schemas`, cujo schema `org.gnome.settings-daemon.plugins.xsettings`
/// é incompatível com o GTK3/WebKit2GTK do sistema (`does not contain a key named 'antialiasing'`).
/// Limpamos os caminhos `/snap/` antes de inicializar o runtime gráfico do Tauri.
pub fn sanitize_snap_gtk_env() {
    for var in [
        "GSETTINGS_SCHEMA_DIR",
        "GTK_PATH",
        "GIO_MODULE_DIR",
        "GDK_PIXBUF_MODULE_FILE",
        "GDK_PIXBUF_MODULEDIR",
        "GTK_EXE_PREFIX",
        "GTK_DATA_PREFIX",
    ] {
        if let Ok(val) = std::env::var(var) {
            if val.contains("/snap/") {
                std::env::remove_var(var);
            }
        }
    }

    if let Ok(xdg) = std::env::var("XDG_DATA_DIRS") {
        let cleaned: Vec<&str> = xdg
            .split(':')
            .filter(|p| !p.is_empty() && !p.starts_with("/snap/"))
            .collect();
        if cleaned.is_empty() {
            std::env::set_var(
                "XDG_DATA_DIRS",
                "/usr/local/share:/usr/share:/var/lib/snapd/desktop",
            );
        } else {
            std::env::set_var("XDG_DATA_DIRS", cleaned.join(":"));
        }
    }

    let host_schemas = std::path::Path::new("/usr/share/glib-2.0/schemas");
    if host_schemas.join("gschemas.compiled").is_file()
        && std::env::var("GSETTINGS_SCHEMA_DIR").is_err()
    {
        std::env::set_var("GSETTINGS_SCHEMA_DIR", host_schemas);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    sanitize_snap_gtk_env();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // SQLite lives in the OS app-data dir, not next to the executable
            // (the Python version drops test.db in the current directory).
            let data_dir = app.path().app_data_dir()?;
            let db_path = db::database_path(&data_dir)?;
            let conn = db::initialize(&data_dir)?;
            app.manage(state::AppState::new(conn, db_path));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::login,
            commands::register,
            commands::logout,
            commands::session_status,
            commands::list_entities,
            commands::get_entity,
            commands::create_entity,
            commands::update_entity,
            commands::delete_entity,
            commands::import_canonical_zip,
            commands::export_canonical_zip,
            commands::import_src_json,
            commands::export_src_json,
            commands::src_list_acoes,
            commands::src_get_acao,
            commands::src_create_acao,
            commands::src_update_acao,
            commands::src_delete_acao,
            commands::src_list_participacoes,
            commands::src_create_participacao,
            commands::src_update_participacao,
            commands::src_delete_participacao,
            commands::src_get_meta,
            commands::src_update_meta,
            commands::import_egressos_dir,
            commands::export_egressos_dir,
            commands::egressos_get_catalog,
            commands::egressos_list_rows,
            commands::egressos_get_row,
            commands::egressos_create_row,
            commands::egressos_update_row,
            commands::egressos_delete_row,
            commands::egressos_get_envelope,
            commands::egressos_update_envelope,
            commands::merge_entities,
            commands::link_entities,
            commands::github_get_config,
            commands::github_set_config,
            commands::github_download,
            commands::github_check_destination,
            commands::github_upload,
            commands::github_save_token,
            commands::github_test_token,
            commands::github_remove_token,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Rotas de navegacao do painel, na ordem em que aparecem no menu lateral.
/// Fonte da verdade para o teste de roteamento; espelha `frontend/src/pages/`.
/// SEP-033: `/projects` (nova pagina principal), `/src` e `/src/acoes` (area SRC).
#[cfg(test)]
pub(crate) const ROUTES: &[&str] = &[
    "/",
    "/login",
    "/projects",
    "/dashboard",
    "/src",
    "/src/acoes",
    "/egressos",
    "/egressos/tabelas",
    "/dashboard/researchers",
    "/dashboard/articles",
    "/dashboard/groups",
    "/dashboard/initiatives",
    "/dashboard/advisorships",
    "/dashboard/awards",
    "/dashboard/students",
    "/dashboard/campuses",
    "/dashboard/organizations",
    "/dashboard/fellowships",
    "/dashboard/proficiencies",
    "/dashboard/professional_activities",
    "/dashboard/knowledge_areas",
    "/dashboard/languages",
    "/dashboard/research_productions",
];

#[cfg(test)]
mod tests {
    use super::ROUTES;

    fn app() -> tauri::App<tauri::test::MockRuntime> {
        tauri::test::mock_builder()
            .build(tauri::generate_context!())
            .expect("falha ao construir o app de teste")
    }

    /// SEP-015 / SC-002: os 18 destinos de navegacao devem resolver no binario
    /// compilado, com os hrefs exatamente como o frontend os escreve — sem barra
    /// final e sem sufixo .html.
    ///
    /// Este teste substitui a verificacao manual: o protocolo de asset do Tauri
    /// usa este mesmo resolvedor, entao uma rota que resolve aqui carrega na janela.
    #[test]
    fn todas_as_rotas_do_menu_resolvem_com_o_href_atual() {
        let app = app();
        let resolver = app.asset_resolver();

        let quebradas: Vec<&str> = ROUTES
            .iter()
            .copied()
            .filter(|r| resolver.get(r.to_string()).is_none())
            .collect();

        assert!(
            quebradas.is_empty(),
            "rotas que nao resolvem com o href atual: {quebradas:?}"
        );
    }

    /// O deep link de edicao (`?openId=N`, usado por EntityForm) precisa resolver
    /// para a mesma pagina que a rota sem parametro.
    #[test]
    fn rotas_com_query_string_resolvem() {
        let app = app();
        let resolver = app.asset_resolver();

        assert!(
            resolver.get("/dashboard/groups?openId=42".into()).is_some(),
            "rota com query string nao resolveu"
        );
    }

    /// Guarda contra regressao silenciosa: se uma pagina for adicionada em
    /// `frontend/src/pages/dashboard/` sem entrar em ROUTES, a contagem denuncia.
    /// SEP-033: raiz + login + projects + dashboard + 15 entidades + area SRC (2).
    #[test]
    fn contagem_de_rotas_bate_com_o_menu_lateral() {
        assert_eq!(
            ROUTES.len(),
            23,
            "ROUTES deve cobrir raiz + login + projects + dashboard + 15 entidades + /src + /src/acoes + /egressos + /egressos/tabelas"
        );
    }

    /// Rota de cadastro (/register) adicionada para registro de usuarios.
    #[test]
    fn rota_de_cadastro_resolve() {
        let app = app();
        let resolver = app.asset_resolver();
        assert!(
            resolver.get("/register".into()).is_some(),
            "rota /register nao resolveu"
        );
    }

    #[test]
    fn sanitize_snap_gtk_env_removes_snap_paths_from_xdg_and_gsettings() {
        std::env::set_var(
            "XDG_DATA_DIRS",
            "/snap/code/200/usr/share:/usr/share/ubuntu:/usr/share",
        );
        std::env::set_var(
            "GSETTINGS_SCHEMA_DIR",
            "/snap/code/200/usr/share/glib-2.0/schemas",
        );
        super::sanitize_snap_gtk_env();

        let xdg = std::env::var("XDG_DATA_DIRS").unwrap();
        assert!(!xdg.contains("/snap/"));
        assert!(xdg.contains("/usr/share"));

        if let Ok(gs) = std::env::var("GSETTINGS_SCHEMA_DIR") {
            assert!(!gs.contains("/snap/"));
        }
    }
}

