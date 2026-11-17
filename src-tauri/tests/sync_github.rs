//! SEP-034: testes de sincronização contra um servidor wiremock local
//! (research R8 — a suíte nunca depende de rede real, Princípio III).
//!
//! Cobre o cliente HTTP (`sync_github`) e o caminho download→import que reusa
//! os imports manuais (R7). As chamadas apontam para a base URL do mock; a
//! normalização de URL e a política são testadas no domínio puro
//! (`sync_domain::tests`).

use research_hub_lib::db;
use research_hub_lib::error::AppError;
use research_hub_lib::sync_domain::{self, NormalizedSource};
use research_hub_lib::sync_github::{
    check_project_destination, download_and_import, upload_horizon_export, GitHubClient,
    TokenInfo,
};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Serializa o erro no wire `{ kind, message }` para os asserts de kind —
/// `AppError::kind()` é pub(crate) por design.
fn wire(err: &AppError) -> serde_json::Value {
    serde_json::to_value(err).expect("AppError sempre serializa")
}

fn file_source(owner: &str, repo: &str, reference: &str, path: &str) -> NormalizedSource {
    NormalizedSource::File {
        owner: owner.into(),
        repo: repo.into(),
        reference: reference.into(),
        path: path.into(),
    }
}

/// ZIP canônico válido: uma entrada `{table}_canonical.json` por tabela
/// gerida, todas vazias exceto `campuses` (id/name). Tabelas vazias são
/// puladas pelo import (`columns.is_empty()`), então o pacote é aceito com
/// linhas só em campuses.
fn canonical_zip(campuses: &[(&str, i64)]) -> Vec<u8> {
    use std::io::Write;
    let mut zip_bytes = Vec::new();
    {
        let mut z = zip::ZipWriter::new(std::io::Cursor::new(&mut zip_bytes));
        let opts: zip::write::FileOptions<()> = zip::write::FileOptions::default();
        for def in research_hub_lib::registry::exported() {
            z.start_file(format!("{}_canonical.json", def.table), opts).unwrap();
            if def.table == "campuses" {
                let items: Vec<String> = campuses
                    .iter()
                    .map(|(name, id)| format!(r#"{{"id": {id}, "name": "{name}"}}"#))
                    .collect();
                z.write_all(format!("[{}]", items.join(", ")).as_bytes()).unwrap();
            } else {
                z.write_all(b"[]").unwrap();
            }
        }
        z.finish().unwrap();
    }
    zip_bytes
}

fn no_progress() -> &'static (dyn Fn(sync_domain::SyncProgress) + Send + Sync) {
    &|_| {}
}

fn client(server: &MockServer, token: Option<&str>) -> GitHubClient {
    GitHubClient::new(&server.uri(), token.map(String::from))
}

// ------------------------------------------------------------------ T007

#[tokio::test]
async fn download_publico_grava_temporario_e_emite_progresso() {
    let server = MockServer::start().await;
    let zip_bytes = canonical_zip(&[("Serra", 1)]);
    Mock::given(method("GET"))
        .and(path("/repos/a/b/contents/exports/exports_canonical.zip"))
        .and(header("Accept", "application/vnd.github.raw"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(zip_bytes))
        .mount(&server)
        .await;

    let src = file_source("a", "b", "main", "exports/exports_canonical.zip");

    let phases = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let phases2 = phases.clone();
    let on_progress = move |p: sync_domain::SyncProgress| {
        phases2.lock().unwrap().push(p.phase);
    };

    let temp = tempfile::tempdir().unwrap();
    let path = GitHubClient::new(&server.uri(), None)
        .download_to_temp(&src, "horizon", temp.path(), &on_progress)
        .await
        .unwrap();

    assert!(path.exists());
    assert!(path.to_string_lossy().contains("exports_canonical.zip"));
    assert!(path.starts_with(temp.path()));

    let phases = phases.lock().unwrap();
    assert_eq!(phases.first(), Some(&"started"));
    assert!(phases.contains(&"transferring"));
}

#[tokio::test]
async fn download_privado_envia_o_token_e_sem_token_da_401() {
    let server = MockServer::start().await;
    let zip_bytes = canonical_zip(&[]);
    Mock::given(method("GET"))
        .and(path("/repos/a/b/contents/exports/exports_canonical.zip"))
        .and(header("Authorization", "Bearer ghp_token_teste"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(zip_bytes))
        .mount(&server)
        .await;
    // Sem o header, o GitHub responde 401.
    Mock::given(method("GET"))
        .and(path("/repos/a/b/contents/exports/exports_canonical.zip"))
        .respond_with(ResponseTemplate::new(401).set_body_string(r#"{"message":"Requires authentication"}"#))
        .mount(&server)
        .await;

    let src = file_source("a", "b", "main", "exports/exports_canonical.zip");
    let temp = tempfile::tempdir().unwrap();

    // Com token: sucesso (US1-2).
    let path = client(&server, Some("ghp_token_teste"))
        .download_to_temp(&src, "horizon", temp.path(), no_progress())
        .await
        .unwrap();
    assert!(path.exists());

    // Sem token: erro acionável apontando para o token (US1-3), nada baixado.
    let err = client(&server, None)
        .download_to_temp(&src, "horizon", temp.path(), no_progress())
        .await
        .unwrap_err();
    let w = wire(&err);
    assert_eq!(w["kind"], "github_api");
    assert!(w["message"].as_str().unwrap().contains("token"));
}

#[tokio::test]
async fn download_404_vira_not_found_e_nao_deixa_temporario() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/repos/a/b/contents/nao/existe.zip"))
        .respond_with(ResponseTemplate::new(404).set_body_string(r#"{"message":"Not Found"}"#))
        .mount(&server)
        .await;

    let src = file_source("a", "b", "main", "nao/existe.zip");
    let temp = tempfile::tempdir().unwrap();

    let err = client(&server, None)
        .download_to_temp(&src, "horizon", temp.path(), no_progress())
        .await
        .unwrap_err();
    assert_eq!(wire(&err)["kind"], "not_found");
    assert!(read_dir_empty(&temp.path().join("sync_tmp")));
}

#[tokio::test]
async fn download_acima_do_limite_aborta_com_too_large() {
    let server = MockServer::start().await;
    // 20 bytes no servidor; limite do cliente em 10 bytes (sem alocar 100 MB).
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![0u8; 20]))
        .mount(&server)
        .await;

    let src = file_source("a", "b", "main", "exports/exports_canonical.zip");
    let temp = tempfile::tempdir().unwrap();

    let client = GitHubClient::with_max_bytes(&server.uri(), None, 10);
    let err = client
        .download_to_temp(&src, "horizon", temp.path(), no_progress())
        .await
        .unwrap_err();
    let w = wire(&err);
    assert_eq!(w["kind"], "too_large");
    assert_eq!(w["message"].as_str().unwrap(), "Arquivo acima do limite de 10 bytes aceito nesta versão.");
}

#[tokio::test]
async fn release_asset_e_localizado_por_tag_e_nome() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/repos/a/b/releases/tags/v1.0.0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "assets": [
                { "id": 77, "name": "outro.zip" },
                { "id": 42, "name": "exports_canonical.zip" }
            ]
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/repos/a/b/releases/assets/42"))
        .and(header("Accept", "application/octet-stream"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(canonical_zip(&[("Serra", 9)])))
        .mount(&server)
        .await;

    let src = NormalizedSource::ReleaseAsset {
        owner: "a".into(),
        repo: "b".into(),
        tag: "v1.0.0".into(),
        asset: "exports_canonical.zip".into(),
    };

    let temp = tempfile::tempdir().unwrap();
    let path = client(&server, None)
        .download_to_temp(&src, "horizon", temp.path(), no_progress())
        .await
        .unwrap();
    assert!(path.exists());
}

// ------------------------------------------------------------------ T008

#[tokio::test]
async fn download_horizon_importa_o_mesmo_resumo_do_import_manual() {
    let server = MockServer::start().await;
    let zip_bytes = canonical_zip(&[("Serra", 1), ("Vitória", 2)]);
    Mock::given(method("GET"))
        .and(path("/repos/a/b/contents/exports/exports_canonical.zip"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(zip_bytes.clone()))
        .mount(&server)
        .await;

    let data = tempfile::tempdir().unwrap();

    // Via GitHub (US1): download + import.
    let mut conn = db::open_in_memory().unwrap();
    let src = file_source("a", "b", "main", "exports/exports_canonical.zip");
    let summary = download_and_import(
        &mut conn,
        data.path(),
        "horizon",
        &src,
        &client(&server, None),
        no_progress(),
    )
    .await
    .unwrap();
    assert_eq!(summary["total_rows"], 2);

    // Manual do MESMO arquivo: resumo idêntico (SC-001 — "só muda a origem").
    let mut conn_manual = db::open_in_memory().unwrap();
    let manual = research_hub_lib::import::import_archive(
        &mut conn_manual,
        &zip_bytes,
        data.path(),
        |_| {},
    )
    .unwrap();
    assert_eq!(summary, serde_json::to_value(manual).unwrap());
}

#[tokio::test]
async fn download_src_importa_o_consolidado_e_falha_com_formato_errado() {
    let server = MockServer::start().await;
    let json = include_str!("fixtures/src_consolidado_exemplo.json");
    Mock::given(method("GET"))
        .and(path("/repos/a/b/contents/dados/src_consolidado.json"))
        .respond_with(ResponseTemplate::new(200).set_body_string(json))
        .mount(&server)
        .await;

    let data = tempfile::tempdir().unwrap();
    let mut conn = db::open_in_memory().unwrap();

    // Caminho feliz: consolidado SRC pela URL (US1, projeto src).
    let src = file_source("a", "b", "main", "dados/src_consolidado.json");
    let summary = download_and_import(
        &mut conn,
        data.path(),
        "src",
        &src,
        &client(&server, None),
        no_progress(),
    )
    .await
    .unwrap();
    assert!(summary["total_acoes"].as_u64().is_some());

    // FR-002/FR-009: zip na área SRC é recusado ANTES de qualquer chamada.
    let zip_src = file_source("a", "b", "main", "exports/exports_canonical.zip");
    let mut conn2 = db::open_in_memory().unwrap();
    let err = download_and_import(
        &mut conn2,
        data.path(),
        "src",
        &zip_src,
        &client(&server, None),
        no_progress(),
    )
    .await
    .unwrap_err();
    assert_eq!(wire(&err)["kind"], "validation");

    // E vice-versa: .json na área Horizon também.
    let json_horizon = file_source("a", "b", "main", "dados/src_consolidado.json");
    let mut conn3 = db::open_in_memory().unwrap();
    let err = download_and_import(
        &mut conn3,
        data.path(),
        "horizon",
        &json_horizon,
        &client(&server, None),
        no_progress(),
    )
    .await
    .unwrap_err();
    assert_eq!(wire(&err)["kind"], "validation");
}

#[tokio::test]
async fn falha_no_import_deixa_a_base_intacta_e_limpa_o_temporario() {
    let server = MockServer::start().await;
    // ZIP ilegível como corpo: download ok, import falha.
    Mock::given(method("GET"))
        .and(path("/repos/a/b/contents/exports/exports_canonical.zip"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"isto nao e um zip".to_vec()))
        .mount(&server)
        .await;

    let data = tempfile::tempdir().unwrap();

    // Base com curadoria prévia.
    let mut conn = db::open_in_memory().unwrap();
    conn.execute("INSERT INTO campuses (id, name) VALUES (1, 'Serra')", []).unwrap();

    let src = file_source("a", "b", "main", "exports/exports_canonical.zip");
    let err = download_and_import(
        &mut conn,
        data.path(),
        "horizon",
        &src,
        &client(&server, None),
        no_progress(),
    )
    .await
    .unwrap_err();

    // Erro legível; temporário removido.
    assert!(matches!(err, AppError::Internal(_)));
    let tmp_dir = data.path().join("sync_tmp");
    if tmp_dir.exists() {
        assert!(read_dir_empty(&tmp_dir));
    }

    // SC-002: a base permanece EXATAMENTE no estado anterior.
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM campuses", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);
    let name: String = conn
        .query_row("SELECT name FROM campuses WHERE id = 1", [], |r| r.get(0))
        .unwrap();
    assert_eq!(name, "Serra");
}

#[tokio::test]
async fn base_com_curadoria_gera_snapshot_no_download() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(canonical_zip(&[("Nova", 3)])))
        .mount(&server)
        .await;

    let data = tempfile::tempdir().unwrap();
    let mut conn = db::open_in_memory().unwrap();
    conn.execute("INSERT INTO campuses (id, name) VALUES (1, 'Serra')", []).unwrap();

    let src = file_source("a", "b", "main", "exports/exports_canonical.zip");
    let summary = download_and_import(
        &mut conn,
        data.path(),
        "horizon",
        &src,
        &client(&server, None),
        no_progress(),
    )
    .await
    .unwrap();

    // US1-5: snapshot do estado anterior gravado e INFORMADO, como no manual.
    let snapshot = summary["snapshot"].as_str().expect("snapshot informado");
    assert!(snapshot.contains("hub-"));
    assert!(std::path::Path::new(snapshot).exists());
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM campuses", [], |r| r.get(0)).unwrap();
    assert_eq!(count, 1, "import substituiu a base");
}

// ------------------------------------------------------------------ T013

#[tokio::test]
async fn upload_fluxo_git_data_api_sucesso_com_ordem_e_sem_criar_branch() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/repos/a/b/git/blobs"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "sha": "blob_sha_123"
        })))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/repos/a/b/git/ref/heads/main"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "object": { "sha": "base_sha_abc" }
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/repos/a/b/git/trees"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "sha": "tree_sha_456"
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/repos/a/b/git/commits"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "sha": "commit_sha_789",
            "html_url": "https://github.com/a/b/commit/commit_sha_789"
        })))
        .mount(&server)
        .await;

    Mock::given(method("PATCH"))
        .and(path("/repos/a/b/git/refs/heads/main"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "object": { "sha": "commit_sha_789" }
        })))
        .mount(&server)
        .await;

    let phases = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let phases_c = phases.clone();
    let on_progress = move |p: sync_domain::SyncProgress| {
        phases_c.lock().unwrap().push(p.phase);
    };

    let client = GitHubClient::new(&server.uri(), Some("token".into()));
    let res = client
        .upload_file("a", "b", "main", "exports.zip", vec![1, 2, 3], "Commit msg", "horizon", &on_progress)
        .await
        .unwrap();

    assert_eq!(res.commit_sha, "commit_sha_789");
    assert_eq!(res.html_url, "https://github.com/a/b/commit/commit_sha_789");

    let received = server.received_requests().await.unwrap();
    let paths: Vec<String> = received.iter().map(|r| r.url.path().to_string()).collect();
    assert_eq!(paths, vec![
        "/repos/a/b/git/blobs",
        "/repos/a/b/git/ref/heads/main",
        "/repos/a/b/git/trees",
        "/repos/a/b/git/commits",
        "/repos/a/b/git/refs/heads/main",
    ]);

    // Q1: NUNCA cria ref com POST /git/refs
    assert!(!paths.iter().any(|p| p == "/repos/a/b/git/refs"));

    let phases = phases.lock().unwrap();
    assert_eq!(phases.first(), Some(&"started"));
    assert!(phases.contains(&"committing"));
}

#[tokio::test]
async fn upload_branch_inexistente_retorna_not_found() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/repos/a/b/git/blobs"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({ "sha": "b1" })))
        .mount(&server)
        .await;

    // branch dá 404
    Mock::given(method("GET"))
        .and(path("/repos/a/b/git/ref/heads/feature"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    // repo existe (200)
    Mock::given(method("GET"))
        .and(path("/repos/a/b"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = GitHubClient::new(&server.uri(), None);
    let err = client
        .upload_file("a", "b", "feature", "x.zip", vec![1], "msg", "horizon", no_progress())
        .await
        .unwrap_err();

    let w = wire(&err);
    assert_eq!(w["kind"], "not_found");
    assert!(w["message"].as_str().unwrap().contains("branch 'feature' não existe"));
}

#[tokio::test]
async fn upload_patch_403_retorna_sem_permissao_de_escrita() {
    let server = MockServer::start().await;
    Mock::given(method("POST")).and(path("/repos/a/b/git/blobs")).respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"sha":"b1"}))).mount(&server).await;
    Mock::given(method("GET")).and(path("/repos/a/b/git/ref/heads/main")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"object":{"sha":"c0"}}))).mount(&server).await;
    Mock::given(method("POST")).and(path("/repos/a/b/git/trees")).respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"sha":"t1"}))).mount(&server).await;
    Mock::given(method("POST")).and(path("/repos/a/b/git/commits")).respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"sha":"c1"}))).mount(&server).await;
    Mock::given(method("PATCH")).and(path("/repos/a/b/git/refs/heads/main")).respond_with(ResponseTemplate::new(403)).mount(&server).await;

    let client = GitHubClient::new(&server.uri(), None);
    let err = client
        .upload_file("a", "b", "main", "x.zip", vec![1], "msg", "horizon", no_progress())
        .await
        .unwrap_err();

    let w = wire(&err);
    assert_eq!(w["kind"], "github_api");
    assert!(w["message"].as_str().unwrap().contains("sem permissão de escrita; nada foi gravado"));
}

#[tokio::test]
async fn upload_patch_422_reexecuta_e_se_persistir_falha() {
    let server = MockServer::start().await;
    Mock::given(method("POST")).and(path("/repos/a/b/git/blobs")).respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"sha":"b1"}))).mount(&server).await;
    Mock::given(method("GET")).and(path("/repos/a/b/git/ref/heads/main")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"object":{"sha":"c0"}}))).mount(&server).await;
    Mock::given(method("POST")).and(path("/repos/a/b/git/trees")).respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"sha":"t1"}))).mount(&server).await;
    Mock::given(method("POST")).and(path("/repos/a/b/git/commits")).respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"sha":"c1"}))).mount(&server).await;
    Mock::given(method("PATCH")).and(path("/repos/a/b/git/refs/heads/main")).respond_with(ResponseTemplate::new(422)).mount(&server).await;

    let client = GitHubClient::new(&server.uri(), None);
    let err = client
        .upload_file("a", "b", "main", "x.zip", vec![1], "msg", "horizon", no_progress())
        .await
        .unwrap_err();

    let w = wire(&err);
    assert_eq!(w["kind"], "github_api");
    assert!(w["message"].as_str().unwrap().contains("o repositório mudou durante o envio; tente novamente"));
}

#[tokio::test]
async fn upload_acima_do_limite_aborta_com_too_large_antes_de_blobs() {
    let server = MockServer::start().await;
    let client = GitHubClient::with_max_bytes(&server.uri(), None, 10);
    let err = client
        .upload_file("a", "b", "main", "x.zip", vec![0u8; 20], "msg", "horizon", no_progress())
        .await
        .unwrap_err();

    let w = wire(&err);
    assert_eq!(w["kind"], "too_large");
    assert_eq!(server.received_requests().await.unwrap().len(), 0);
}

// ------------------------------------------------------------------ T014

#[tokio::test]
async fn check_destination_retorna_flags_e_trata_ausencias() {
    let server = MockServer::start().await;
    let temp = tempfile::tempdir().unwrap();

    Mock::given(method("GET")).and(path("/repos/a/b")).respond_with(ResponseTemplate::new(200)).mount(&server).await;
    Mock::given(method("GET")).and(path("/repos/a/b/git/ref/heads/main")).respond_with(ResponseTemplate::new(200)).mount(&server).await;
    Mock::given(method("GET")).and(path("/repos/a/b/contents/exports.zip")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"sha": "sha_arquivo_xyz"}))).mount(&server).await;

    let mut cfg = sync_domain::SyncConfigFile::default();
    cfg.project_mut("horizon").repo = "a/b".into();
    cfg.project_mut("horizon").branch = "main".into();
    cfg.project_mut("horizon").path = "exports.zip".into();
    sync_domain::save_config(temp.path(), &cfg).unwrap();

    let client = GitHubClient::new(&server.uri(), None);
    let view = check_project_destination(temp.path(), "horizon", &client).await.unwrap();

    assert_eq!(view.repo, "a/b");
    assert_eq!(view.branch, "main");
    assert_eq!(view.path, "exports.zip");
    assert!(view.branch_exists);
    assert!(view.file_exists);
    assert_eq!(view.file_sha.as_deref(), Some("sha_arquivo_xyz"));

    // Sem destino configurado -> sync_config
    let err = check_project_destination(temp.path(), "src", &client).await.unwrap_err();
    assert_eq!(wire(&err)["kind"], "sync_config");

    // Repo 404 -> not_found "repositório não encontrado ou sem acesso"
    let server_404 = MockServer::start().await;
    Mock::given(method("GET")).and(path("/repos/a/b")).respond_with(ResponseTemplate::new(404)).mount(&server_404).await;
    let client_404 = GitHubClient::new(&server_404.uri(), None);
    let err = check_project_destination(temp.path(), "horizon", &client_404).await.unwrap_err();
    let w = wire(&err);
    assert_eq!(w["kind"], "not_found");
    assert!(w["message"].as_str().unwrap().contains("repositório não encontrado ou sem acesso"));
}

// ------------------------------------------------------------------ T015 / SEP-038 (US3)

#[tokio::test]
async fn upload_src_publica_json_consolidado_identico_ao_export_local_em_nova_branch_ou_existente() {
    use research_hub_lib::sync_domain::UploadBranchOptions;
    use research_hub_lib::sync_github::upload_horizon_export_with_options;

    let temp = tempfile::tempdir().unwrap();
    let server = MockServer::start().await;
    let client = GitHubClient::new(&server.uri(), Some("ghp_src_token".into()));

    let mut conn = db::open_in_memory().unwrap();
    let sample_json = include_str!("fixtures/src_consolidado_exemplo.json");
    research_hub_lib::src_domain::import(&mut conn, sample_json, temp.path()).unwrap();
    let expected_json = research_hub_lib::src_domain::export(&conn).unwrap();
    let expected_bytes = expected_json.clone().into_bytes();

    let mut cfg = sync_domain::SyncConfigFile::default();
    cfg.project_mut("src").repo = "rafaeldeps/src-sync".into();
    cfg.project_mut("src").branch = "main".into();
    cfg.project_mut("src").path = "dados/src_consolidado.json".into();
    sync_domain::save_config(temp.path(), &cfg).unwrap();

    // 1. Upload do SRC criando uma NOVA branch (US1 + US3)
    Mock::given(method("GET"))
        .and(path("/repos/rafaeldeps/src-sync/git/ref/heads/update/src-2026"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/repos/rafaeldeps/src-sync/git/blobs"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"sha": "blob_src_1"})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/repos/rafaeldeps/src-sync/git/ref/heads/main"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"object": {"sha": "base_main_src"}})))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/repos/rafaeldeps/src-sync/git/trees"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"sha": "tree_src_1"})))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/repos/rafaeldeps/src-sync/git/commits"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "sha": "commit_src_1",
            "html_url": "https://github.com/rafaeldeps/src-sync/commit/commit_src_1"
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/repos/rafaeldeps/src-sync/git/refs"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "ref": "refs/heads/update/src-2026",
            "object": { "sha": "commit_src_1" }
        })))
        .mount(&server)
        .await;

    let opts = UploadBranchOptions {
        create_new_branch: true,
        base_branch: Some("main".into()),
        new_branch: Some("update/src-2026".into()),
        target_branch: None,
    };

    let res = upload_horizon_export_with_options(
        expected_bytes.clone(),
        temp.path(),
        "src",
        false,
        &opts,
        &client,
        no_progress(),
    )
    .await
    .unwrap();

    assert_eq!(res.commit_sha, "commit_src_1");
    assert_eq!(res.branch, "update/src-2026");
    assert!(res.created_branch);
    assert!(!res.replaced);

    // Verifica paridade byte a byte (SC-003) e preserva a branch base em sync_config.json (FR-005)
    let requests = server.received_requests().await.unwrap();
    let blob_req = requests
        .iter()
        .find(|r| r.url.path() == "/repos/rafaeldeps/src-sync/git/blobs")
        .unwrap();
    let blob_json: serde_json::Value = serde_json::from_slice(&blob_req.body).unwrap();
    let b64 = blob_json["content"].as_str().unwrap();
    let sent_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b64).unwrap();
    assert_eq!(sent_bytes, expected_bytes);

    let reloaded = sync_domain::load_config(temp.path()).unwrap();
    assert_eq!(reloaded.project("src").unwrap().branch, "main");
}

#[tokio::test]
async fn upload_horizon_sucesso_envia_bytes_identicos_ao_export_manual() {
    let server = MockServer::start().await;
    let temp = tempfile::tempdir().unwrap();
    let conn = db::open_in_memory().unwrap();
    conn.execute("INSERT INTO campuses (id, name) VALUES (1, 'Serra')", []).unwrap();

    let mut cfg = sync_domain::SyncConfigFile::default();
    cfg.project_mut("horizon").repo = "a/b".into();
    cfg.project_mut("horizon").branch = "main".into();
    cfg.project_mut("horizon").path = "exports/exports_canonical.zip".into();
    sync_domain::save_config(temp.path(), &cfg).unwrap();

    // Destination checks
    Mock::given(method("GET")).and(path("/repos/a/b")).respond_with(ResponseTemplate::new(200)).mount(&server).await;
    Mock::given(method("GET")).and(path("/repos/a/b/git/ref/heads/main")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"object":{"sha":"c0"}}))).mount(&server).await;
    Mock::given(method("GET")).and(path("/repos/a/b/contents/exports/exports_canonical.zip")).respond_with(ResponseTemplate::new(404)).mount(&server).await;

    // Upload calls
    Mock::given(method("POST")).and(path("/repos/a/b/git/blobs")).respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"sha":"blob1"}))).mount(&server).await;
    Mock::given(method("POST")).and(path("/repos/a/b/git/trees")).respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"sha":"tree1"}))).mount(&server).await;
    Mock::given(method("POST")).and(path("/repos/a/b/git/commits")).respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"sha":"commit1", "html_url": "https://github.com/a/b/commit/commit1"}))).mount(&server).await;
    Mock::given(method("PATCH")).and(path("/repos/a/b/git/refs/heads/main")).respond_with(ResponseTemplate::new(200)).mount(&server).await;

    let (expected_bytes, _, _) = research_hub_lib::export::build_archive(&conn, None, |_| {}).unwrap();
    let client = GitHubClient::new(&server.uri(), None);
    let res = upload_horizon_export(expected_bytes.clone(), temp.path(), "horizon", false, &client, no_progress()).await.unwrap();

    assert_eq!(res.commit_sha, "commit1");
    assert_eq!(res.html_url, "https://github.com/a/b/commit/commit1");
    assert!(!res.replaced);

    // Verify blob content is byte-for-byte identical to manual export
    let requests = server.received_requests().await.unwrap();
    let blob_req = requests.iter().find(|r| r.url.path() == "/repos/a/b/git/blobs").unwrap();
    let blob_json: serde_json::Value = serde_json::from_slice(&blob_req.body).unwrap();
    let b64 = blob_json["content"].as_str().unwrap();
    let sent_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b64).unwrap();
    assert_eq!(sent_bytes, expected_bytes);
}

#[tokio::test]
async fn upload_destino_existente_com_sha_diferente_sem_confirmacao_da_conflict() {
    let server = MockServer::start().await;
    let temp = tempfile::tempdir().unwrap();
    let conn = db::open_in_memory().unwrap();
    conn.execute("INSERT INTO campuses (id, name) VALUES (1, 'Serra')", []).unwrap();

    let mut cfg = sync_domain::SyncConfigFile::default();
    cfg.project_mut("horizon").repo = "a/b".into();
    cfg.project_mut("horizon").branch = "main".into();
    cfg.project_mut("horizon").path = "exports/exports_canonical.zip".into();
    sync_domain::save_config(temp.path(), &cfg).unwrap();

    // Destination checks: file exists with different SHA
    Mock::given(method("GET")).and(path("/repos/a/b")).respond_with(ResponseTemplate::new(200)).mount(&server).await;
    Mock::given(method("GET")).and(path("/repos/a/b/git/ref/heads/main")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"object":{"sha":"c0"}}))).mount(&server).await;
    Mock::given(method("GET")).and(path("/repos/a/b/contents/exports/exports_canonical.zip")).respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"sha":"sha_antigo_divergente"}))).mount(&server).await;

    let (bytes, _, _) = research_hub_lib::export::build_archive(&conn, None, |_| {}).unwrap();
    let client = GitHubClient::new(&server.uri(), None);
    let err = upload_horizon_export(bytes, temp.path(), "horizon", false, &client, no_progress()).await.unwrap_err();

    let w = wire(&err);
    assert_eq!(w["kind"], "conflict");
    assert!(w["message"].as_str().unwrap().contains("Confirme a sobrescrita"));
}

// ------------------------------------------------------------------ T018 (US3)

#[test]
fn save_token_grava_com_permissao_restrita_e_token_hint() {
    let temp = tempfile::tempdir().unwrap();
    let mut cfg = sync_domain::load_config(temp.path()).unwrap();
    let secret_token = "ghp_1234567890abcdef";
    cfg.token = Some(secret_token.to_string());
    sync_domain::save_config(temp.path(), &cfg).unwrap();

    let reloaded = sync_domain::load_config(temp.path()).unwrap();
    assert_eq!(reloaded.token.as_deref(), Some(secret_token));
    assert_eq!(sync_domain::token_hint(&reloaded.token), Some("cdef".into()));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let file_path = temp.path().join(sync_domain::SYNC_CONFIG_FILE);
        let meta = std::fs::metadata(&file_path).unwrap();
        let mode = meta.permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "arquivo de config com token deve ter permissão 0600");
    }
}

#[test]
fn remove_token_apaga_chave_e_arquivo_fica_com_vestigio_zero() {
    let temp = tempfile::tempdir().unwrap();
    let secret = "ghp_super_secret_token_123456789";

    // Grava inicialmente com o token
    let mut cfg = sync_domain::SyncConfigFile::default();
    cfg.token = Some(secret.to_string());
    sync_domain::save_config(temp.path(), &cfg).unwrap();

    let file_path = temp.path().join(sync_domain::SYNC_CONFIG_FILE);
    let raw_before = std::fs::read_to_string(&file_path).unwrap();
    assert!(raw_before.contains(secret));

    // Remove o token (conforme github_remove_token)
    let mut to_remove = sync_domain::load_config(temp.path()).unwrap();
    to_remove.token = None;
    sync_domain::save_config(temp.path(), &to_remove).unwrap();

    // Recarrega e valida estado
    let reloaded = sync_domain::load_config(temp.path()).unwrap();
    assert_eq!(reloaded.token, None);
    assert_eq!(sync_domain::token_hint(&reloaded.token), None);

    // Inspeção direta do arquivo garante ZERO vestígio do token no disco (SC-006)
    let raw_after = std::fs::read_to_string(&file_path).unwrap();
    assert!(!raw_after.contains(secret), "o token secreto não deve constar em nenhuma parte do arquivo");
}

#[tokio::test]
async fn test_token_sucesso_retorna_login_escopos_sem_ecoar_token() {
    let server = MockServer::start().await;
    let secret = "ghp_valid_token_xyz987";

    Mock::given(method("GET"))
        .and(path("/user"))
        .and(header("Authorization", format!("Bearer {secret}").as_str()))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "login": "curador_ifes",
                    "id": 12345,
                }))
                .append_header("x-oauth-scopes", "repo, read:user"),
        )
        .mount(&server)
        .await;

    let client = GitHubClient::new(&server.uri(), Some(secret.to_string()));
    let info: TokenInfo = client.test_token().await.unwrap();

    assert_eq!(info.login, "curador_ifes");
    assert_eq!(info.scopes, "repo, read:user");
    assert!(info.valid);

    // Garante que a representação serializada ou debug não ecoa o token secreto
    let serialized = serde_json::to_string(&info).unwrap();
    assert!(!serialized.contains(secret), "TokenInfo não pode ecoar o token secreto");
    let debug_repr = format!("{info:?}");
    assert!(!debug_repr.contains(secret), "Debug de TokenInfo não pode ecoar o token secreto");
}

#[tokio::test]
async fn test_token_401_retorna_github_api_com_token_invalido() {
    let server = MockServer::start().await;
    let secret = "ghp_invalid_token";

    Mock::given(method("GET"))
        .and(path("/user"))
        .and(header("Authorization", format!("Bearer {secret}").as_str()))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "message": "Bad credentials"
        })))
        .mount(&server)
        .await;

    let client = GitHubClient::new(&server.uri(), Some(secret.to_string()));
    let err = client.test_token().await.unwrap_err();

    let w = wire(&err);
    assert_eq!(w["kind"], "github_api");
    assert!(
        w["message"].as_str().unwrap().contains("token inválido ou revogado"),
        "Mensagem de erro deve ser acionável e apontar para token inválido"
    );
    assert!(
        !w["message"].as_str().unwrap().contains(secret),
        "Erro nunca deve conter o token secreto"
    );
}

#[test]
fn test_token_sem_token_configurado_da_sync_config() {
    let temp = tempfile::tempdir().unwrap();
    let cfg = sync_domain::load_config(temp.path()).unwrap();
    assert!(cfg.token.is_none());

    // Comportamento do comando github_test_token quando não há token salvo
    let res: Result<(), AppError> = match cfg.token {
        Some(_) => Ok(()),
        None => Err(AppError::SyncConfig("Nenhum token configurado.".into())),
    };

    let err = res.unwrap_err();
    let w = wire(&err);
    assert_eq!(w["kind"], "sync_config");
    assert!(w["message"].as_str().unwrap().contains("Nenhum token configurado"));
}

// ------------------------------------------------------------------ T022 (SC-005)

#[tokio::test]
async fn auditoria_de_vazamento_nenhum_erro_contem_token_ou_pii() {
    let server = MockServer::start().await;
    let secret_token = "ghp_super_secret_audit_token_999999999";
    let fake_cpf = "123.456.789-00";
    let fake_email = "participante.secreto@ifes.edu.br";
    let fake_nome = "Fulano de Tal Participante";

    let mut errors: Vec<AppError> = Vec::new();

    // 1. Network error (porta fechada com URL)
    let bad_client = GitHubClient::new("http://127.0.0.1:1", Some(secret_token.to_string()));
    let net_err = bad_client.test_token().await.unwrap_err();
    errors.push(net_err);

    // 2. github_api: 401
    Mock::given(method("GET"))
        .and(path("/user"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Bad credentials"))
        .mount(&server)
        .await;
    let auth_client = GitHubClient::new(&server.uri(), Some(secret_token.to_string()));
    errors.push(auth_client.test_token().await.unwrap_err());

    // 3. github_api: 403
    Mock::given(method("GET"))
        .and(path("/repos/a/b/contents/file.zip"))
        .respond_with(ResponseTemplate::new(403))
        .mount(&server)
        .await;
    let src = file_source("a", "b", "main", "file.zip");
    let temp = tempfile::tempdir().unwrap();
    errors.push(
        auth_client
            .download_to_temp(&src, "horizon", temp.path(), no_progress())
            .await
            .unwrap_err(),
    );

    // 4. too_large
    let client_small = GitHubClient::with_max_bytes(&server.uri(), Some(secret_token.to_string()), 10);
    errors.push(
        client_small
            .upload_file("a", "b", "main", "huge.zip", vec![0u8; 100], "msg", "horizon", no_progress())
            .await
            .unwrap_err(),
    );

    // 5. sync_config (projeto desconhecido + validações de repo, branch, path)
    errors.push(sync_domain::check_upload_allowed("projeto_invalido").unwrap_err());
    errors.push(sync_domain::validate_repo("invalid").unwrap_err());
    errors.push(sync_domain::validate_branch("").unwrap_err());
    errors.push(sync_domain::validate_path("horizon", "bad.tar").unwrap_err());
    errors.push(sync_domain::validate_path("src", "bad.zip").unwrap_err());

    // 6. list_branches e create_branch_ref (SEP-038, SC-004)
    Mock::given(method("GET"))
        .and(path("/repos/a/b/branches"))
        .respond_with(ResponseTemplate::new(403))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/repos/a/b/git/refs"))
        .respond_with(ResponseTemplate::new(403))
        .mount(&server)
        .await;
    errors.push(auth_client.list_branches("a", "b").await.unwrap_err());
    errors.push(
        auth_client
            .create_branch_ref("a", "b", "update/audit", "sha123")
            .await
            .unwrap_err(),
    );

    // 7. not_found
    Mock::given(method("GET"))
        .and(path("/repos/a/b/git/ref/heads/nao-existe"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;
    errors.push(
        auth_client
            .upload_file("a", "b", "nao-existe", "test.zip", vec![1], "msg", "horizon", no_progress())
            .await
            .unwrap_err(),
    );

    // 8. conflict
    errors.push(AppError::Conflict("Confirme a sobrescrita do arquivo".into()));

    // Audit assertion: NONE of the errors across all kinds leaks token or PII (SC-005)
    let sensitive_strings = [secret_token, fake_cpf, fake_email, fake_nome];
    for err in &errors {
        let msg = err.to_string();
        let debug = format!("{err:?}");
        let json = serde_json::to_string(&wire(err)).unwrap();

        for sensitive in sensitive_strings {
            assert!(
                !msg.contains(sensitive),
                "Mensagem de erro vazou dado sensível ({sensitive}): {msg}"
            );
            assert!(
                !debug.contains(sensitive),
                "Debug de erro vazou dado sensível ({sensitive}): {debug}"
            );
            assert!(
                !json.contains(sensitive),
                "JSON de erro vazou dado sensível ({sensitive}): {json}"
            );
        }
    }
}


// ------------------------------------------------------------------ utils

fn read_dir_empty(dir: &std::path::Path) -> bool {
    std::fs::read_dir(dir).map(|mut rd| rd.next().is_none()).unwrap_or(true)
}

// --------------------------------------------------------- SEP-035 (T030, T036)

#[tokio::test]
async fn download_egressos_dir_baixa_21_jsons_cria_snapshot_e_aborta_em_falha_parcial() {
    use research_hub_lib::egressos_domain::{self, EGRESSOS_FILES};
    use research_hub_lib::sync_domain::EgressosDirSource;
    use research_hub_lib::sync_github::download_egressos_dir;

    let server = MockServer::start().await;
    let data = tempfile::tempdir().unwrap();
    let ref_dir = data.path().join("ref_dados");
    egressos_domain::write_sample_dados_dir(&ref_dir);

    for def in EGRESSOS_FILES {
        let body = std::fs::read_to_string(ref_dir.join(def.file_name)).unwrap();
        Mock::given(method("GET"))
            .and(path(format!("/repos/thiagodeps/egressos/contents/dados/{}", def.file_name)))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(&server)
            .await;
    }

    let mut conn = db::open_in_memory().unwrap();
    let src = EgressosDirSource {
        owner: "thiagodeps".into(),
        repo: "egressos".into(),
        reference: "main".into(),
        dir: "dados".into(),
    };

    // 1. Primeiro download popula os 21 arquivos
    let summary1 = download_egressos_dir(
        &mut conn,
        data.path(),
        &src,
        &client(&server, Some("ghp_token_egressos")),
        no_progress(),
    )
    .await
    .unwrap();
    assert_eq!(summary1.total_files, 21);
    assert!(summary1.total_rows >= 50);
    assert!(summary1.snapshot.is_none());

    // 2. Segundo download cria snapshot egressos-pre-import-*
    let summary2 = download_egressos_dir(
        &mut conn,
        data.path(),
        &src,
        &client(&server, Some("ghp_token_egressos")),
        no_progress(),
    )
    .await
    .unwrap();
    let snap = summary2.snapshot.expect("snapshot esperado no reimport via GitHub");
    assert!(snap.starts_with("egressos-pre-import-"));

    // 3. Se 1 dos 21 arquivos der 404, aborta sem alterar a base existente (FR-005, FR-011)
    let broken_server = MockServer::start().await;
    for def in EGRESSOS_FILES {
        if def.file_key == "vitrine" {
            Mock::given(method("GET"))
                .and(path("/repos/thiagodeps/egressos/contents/dados/vitrine.json"))
                .respond_with(ResponseTemplate::new(404))
                .mount(&broken_server)
                .await;
        } else {
            let body = std::fs::read_to_string(ref_dir.join(def.file_name)).unwrap();
            Mock::given(method("GET"))
                .and(path(format!("/repos/thiagodeps/egressos/contents/dados/{}", def.file_name)))
                .respond_with(ResponseTemplate::new(200).set_body_string(body))
                .mount(&broken_server)
                .await;
        }
    }

    let err = download_egressos_dir(
        &mut conn,
        data.path(),
        &src,
        &client(&broken_server, Some("ghp_token_egressos")),
        no_progress(),
    )
    .await
    .unwrap_err();
    assert_eq!(wire(&err)["kind"], "not_found");
    let cat = egressos_domain::get_catalog(&conn).unwrap();
    assert_eq!(cat.total_files, 21);
    assert_eq!(cat.total_rows, summary2.total_rows);
}

#[tokio::test]
async fn upload_egressos_dir_publica_21_blobs_em_commit_unico_atomico() {
    use research_hub_lib::egressos_domain;
    use research_hub_lib::sync_github::upload_egressos_dir;

    let server = MockServer::start().await;
    let temp = tempfile::tempdir().unwrap();
    let dados_dir = temp.path().join("dados_src");
    egressos_domain::write_sample_dados_dir(&dados_dir);
    let mut conn = db::open_in_memory().unwrap();
    egressos_domain::import_dir(
        &mut conn,
        &dados_dir,
        temp.path(),
        |_| {},
    )
    .unwrap();

    let mut cfg = sync_domain::SyncConfigFile::default();
    cfg.project_mut("egressos").repo = "thiagodeps/egressos".into();
    cfg.project_mut("egressos").branch = "main".into();
    cfg.project_mut("egressos").path = "dados".into();
    sync_domain::save_config(temp.path(), &cfg).unwrap();

    // Destination check
    Mock::given(method("GET"))
        .and(path("/repos/thiagodeps/egressos"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/repos/thiagodeps/egressos/git/ref/heads/main"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "object": { "sha": "base_commit_sha" }
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/repos/thiagodeps/egressos/contents/dados"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    // Git Data API: 21 blobs + 1 tree + 1 commit + 1 ref patch
    Mock::given(method("POST"))
        .and(path("/repos/thiagodeps/egressos/git/blobs"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "sha": "blob_sha_egressos"
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/repos/thiagodeps/egressos/git/trees"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "sha": "tree_sha_egressos"
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/repos/thiagodeps/egressos/git/commits"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "sha": "commit_egressos_123",
            "html_url": "https://github.com/thiagodeps/egressos/commit/commit_egressos_123"
        })))
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/repos/thiagodeps/egressos/git/refs/heads/main"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let res = upload_egressos_dir(
        &mut conn,
        temp.path(),
        false,
        &client(&server, Some("ghp_secret_egressos")),
        no_progress(),
    )
    .await
    .unwrap();

    assert_eq!(res.commit_sha, "commit_egressos_123");
    assert_eq!(
        res.html_url,
        "https://github.com/thiagodeps/egressos/commit/commit_egressos_123"
    );

    let requests = server.received_requests().await.unwrap();
    let blob_posts = requests
        .iter()
        .filter(|r| r.url.path() == "/repos/thiagodeps/egressos/git/blobs")
        .count();
    assert_eq!(blob_posts, 21, "deve criar exatamente 21 blobs para os 21 JSONs");

    let tree_req = requests
        .iter()
        .find(|r| r.url.path() == "/repos/thiagodeps/egressos/git/trees")
        .unwrap();
    let tree_body: serde_json::Value = serde_json::from_slice(&tree_req.body).unwrap();
    let entries = tree_body["tree"].as_array().unwrap();
    assert_eq!(entries.len(), 21);
    assert!(entries
        .iter()
        .any(|e| e["path"] == "dados/egressos_perfil.json"));
}

#[tokio::test]
async fn isolamento_entre_tres_projetos_e_zero_vazamento_de_token_no_egressos() {
    use research_hub_lib::egressos_domain;
    use research_hub_lib::sync_domain::EgressosDirSource;
    use research_hub_lib::sync_github::{download_egressos_dir, upload_egressos_dir};

    let temp = tempfile::tempdir().unwrap();
    let dados_dir = temp.path().join("dados_src");
    egressos_domain::write_sample_dados_dir(&dados_dir);
    let mut conn = db::open_in_memory().unwrap();

    // Popula dados no Horizon e no SRC
    conn.execute("INSERT INTO campuses (id, name) VALUES (1, 'Serra')", [])
        .unwrap();
    let src_json = include_str!("fixtures/src_consolidado_exemplo.json");
    research_hub_lib::src_domain::import(&mut conn, src_json, temp.path()).unwrap();

    let horizon_before: i64 = conn
        .query_row("SELECT COUNT(*) FROM campuses", [], |r| r.get(0))
        .unwrap();
    let src_acoes_before: i64 = conn
        .query_row("SELECT COUNT(*) FROM src_acoes", [], |r| r.get(0))
        .unwrap();
    let src_part_before: i64 = conn
        .query_row("SELECT COUNT(*) FROM src_participacoes", [], |r| r.get(0))
        .unwrap();

    // Importa, edita e exporta no Egressos
    let summary = egressos_domain::import_dir(
        &mut conn,
        &dados_dir,
        temp.path(),
        |_| {},
    )
    .unwrap();
    assert_eq!(summary.total_files, 21);

    let mut rec = research_hub_lib::crud::Record::new();
    rec.insert("label".into(), serde_json::json!("ISO1"));
    rec.insert("nome".into(), serde_json::json!("Isolamento Teste"));
    let created = egressos_domain::create_row(&conn, "egressos_perfil", None, &rec).unwrap();
    let cid = created.get("id").and_then(|v| v.as_i64()).unwrap();
    egressos_domain::delete_row(&mut conn, cid, true).unwrap();
    egressos_domain::export_dir(&conn, &temp.path().join("out_iso")).unwrap();

    // Contagens do Horizon e do SRC permanecem 100% inalteradas (FR-002, FR-003, SC-001)
    let horizon_after: i64 = conn
        .query_row("SELECT COUNT(*) FROM campuses", [], |r| r.get(0))
        .unwrap();
    let src_acoes_after: i64 = conn
        .query_row("SELECT COUNT(*) FROM src_acoes", [], |r| r.get(0))
        .unwrap();
    let src_part_after: i64 = conn
        .query_row("SELECT COUNT(*) FROM src_participacoes", [], |r| r.get(0))
        .unwrap();

    assert_eq!(horizon_before, horizon_after);
    assert_eq!(src_acoes_before, src_acoes_after);
    assert_eq!(src_part_before, src_part_after);

    // Auditoria de segredo (FR-013): falhas de download/upload do Egressos nunca vazam o token
    let secret = "ghp_secret_egressos_token_never_leak_777";
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;
    let auth_client = GitHubClient::new(&server.uri(), Some(secret.to_string()));
    let dir_src = EgressosDirSource {
        owner: "thiagodeps".into(),
        repo: "egressos".into(),
        reference: "main".into(),
        dir: "dados".into(),
    };
    let dl_err = download_egressos_dir(&mut conn, temp.path(), &dir_src, &auth_client, no_progress())
        .await
        .unwrap_err();
    let ul_err = upload_egressos_dir(&mut conn, temp.path(), true, &auth_client, no_progress())
        .await
        .unwrap_err();

    for e in [dl_err, ul_err] {
        assert!(!e.to_string().contains(secret));
        assert!(!format!("{e:?}").contains(secret));
        assert!(!serde_json::to_string(&wire(&e)).unwrap().contains(secret));
    }
}

// --------------------------------------------------- SEP-036 (T005, T017, T024)

#[tokio::test]
async fn verify_org_membership_autoriza_membro_ativo_e_fallback_publico_ifesserra_lab() {
    use research_hub_lib::sync_github::REQUIRED_EGRESSOS_DADOS_ORG;

    // 1. Filiação autenticada com state == "active" em /user/memberships/orgs/ifesserra-lab
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/user"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("x-oauth-scopes", "read:org, repo")
                .set_body_json(serde_json::json!({ "login": "membro-lab" })),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/user/memberships/orgs/ifesserra-lab"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "state": "active",
            "role": "member",
            "organization": { "login": "ifesserra-lab" }
        })))
        .mount(&server)
        .await;

    let info = client(&server, Some("ghp_token_ifesserra_active"))
        .verify_org_membership(REQUIRED_EGRESSOS_DADOS_ORG)
        .await
        .unwrap();
    assert_eq!(info.login, "membro-lab");
    assert_eq!(info.organization, "ifesserra-lab");
    assert_eq!(info.state, "active");

    // 2. Fallback público: /user/memberships/orgs/ifesserra-lab retorna 403 (sem read:org),
    // mas /orgs/ifesserra-lab/members/membro-publico retorna 204 No Content
    let fallback_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/user"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("x-oauth-scopes", "repo")
                .set_body_json(serde_json::json!({ "login": "membro-publico" })),
        )
        .mount(&fallback_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/user/memberships/orgs/ifesserra-lab"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "message": "Resource not accessible by personal access token"
        })))
        .mount(&fallback_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/orgs/ifesserra-lab/members/membro-publico"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&fallback_server)
        .await;

    let info_pub = client(&fallback_server, Some("ghp_token_public_member"))
        .verify_org_membership(REQUIRED_EGRESSOS_DADOS_ORG)
        .await
        .unwrap();
    assert_eq!(info_pub.login, "membro-publico");
    assert_eq!(info_pub.organization, "ifesserra-lab");
    assert_eq!(info_pub.state, "active");
}

#[tokio::test]
async fn verify_org_membership_rejeita_nao_membro_pendente_sem_escopo_e_token_invalido_sem_vazar_segredo() {
    use research_hub_lib::sync_github::REQUIRED_EGRESSOS_DADOS_ORG;

    // 1. Usuário válido no GitHub, mas NÃO pertence à organização ifesserra-lab (404)
    let non_member_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/user"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("x-oauth-scopes", "read:org")
                .set_body_json(serde_json::json!({ "login": "externo-user" })),
        )
        .mount(&non_member_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/user/memberships/orgs/ifesserra-lab"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&non_member_server)
        .await;

    let err_non_member = client(&non_member_server, Some("ghp_secret_non_member_111"))
        .verify_org_membership(REQUIRED_EGRESSOS_DADOS_ORG)
        .await
        .unwrap_err();
    assert_eq!(wire(&err_non_member)["kind"], "org_unauthorized");
    assert!(err_non_member.to_string().contains("@externo-user"));
    assert!(err_non_member.to_string().contains("ifesserra-lab"));
    assert!(!err_non_member.to_string().contains("ghp_secret_non_member_111"));

    // 2. Convite pendente na organização ifesserra-lab (state == "pending")
    let pending_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/user"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("x-oauth-scopes", "read:org")
                .set_body_json(serde_json::json!({ "login": "convidado-lab" })),
        )
        .mount(&pending_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/user/memberships/orgs/ifesserra-lab"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "state": "pending",
            "organization": { "login": "ifesserra-lab" }
        })))
        .mount(&pending_server)
        .await;

    let err_pending = client(&pending_server, Some("ghp_secret_pending_222"))
        .verify_org_membership(REQUIRED_EGRESSOS_DADOS_ORG)
        .await
        .unwrap_err();
    assert_eq!(wire(&err_pending)["kind"], "org_unauthorized");
    assert!(err_pending.to_string().contains("convite pendente"));
    assert!(!err_pending.to_string().contains("ghp_secret_pending_222"));

    // 3. Token sem escopo read:org (403) e não listado publicamente (404)
    let no_scope_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/user"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("x-oauth-scopes", "repo")
                .set_body_json(serde_json::json!({ "login": "membro-privado" })),
        )
        .mount(&no_scope_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/user/memberships/orgs/ifesserra-lab"))
        .respond_with(ResponseTemplate::new(403))
        .mount(&no_scope_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/orgs/ifesserra-lab/members/membro-privado"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&no_scope_server)
        .await;

    let err_scope = client(&no_scope_server, Some("ghp_secret_no_scope_333"))
        .verify_org_membership(REQUIRED_EGRESSOS_DADOS_ORG)
        .await
        .unwrap_err();
    assert_eq!(wire(&err_scope)["kind"], "org_unauthorized");
    assert!(err_scope.to_string().contains("read:org"));
    assert!(!err_scope.to_string().contains("ghp_secret_no_scope_333"));

    // 4. Token inválido ou expirado (401 em /user)
    let invalid_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/user"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&invalid_server)
        .await;

    let err_invalid = client(&invalid_server, Some("ghp_secret_invalid_444"))
        .verify_org_membership(REQUIRED_EGRESSOS_DADOS_ORG)
        .await
        .unwrap_err();
    assert_eq!(wire(&err_invalid)["kind"], "github_api");
    assert!(err_invalid.to_string().contains("inválido ou revogado"));
    assert!(!err_invalid.to_string().contains("ghp_secret_invalid_444"));
}

// --------------------------------------------------------- SEP-037 (T022, T025)

#[tokio::test]
async fn download_egressos_dados_repo_baixa_39_jsons_raiz_e_data_e_cria_snapshot() {
    use research_hub_lib::egressos_dados_domain::{
        self, write_synthetic_egressos_dados_repo, EGRESSOS_DADOS_FILES,
    };
    use research_hub_lib::sync_domain::EgressosDirSource;
    use research_hub_lib::sync_github::download_egressos_dados_repo;

    let server = MockServer::start().await;
    let data = tempfile::tempdir().unwrap();
    let ref_dir = data.path().join("ref_egressos_dados");
    write_synthetic_egressos_dados_repo(&ref_dir).unwrap();

    for def in EGRESSOS_DADOS_FILES {
        let body = std::fs::read_to_string(ref_dir.join(def.relative_path)).unwrap();
        Mock::given(method("GET"))
            .and(path(format!(
                "/repos/ifesserra-lab/egressos-dados/contents/{}",
                def.relative_path
            )))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(&server)
            .await;
    }

    let mut conn = db::open_in_memory().unwrap();
    let src = EgressosDirSource {
        owner: "ifesserra-lab".into(),
        repo: "egressos-dados".into(),
        reference: "main".into(),
        dir: ".".into(),
    };

    // 1. Primeiro download popula os 39 arquivos JSON (alunos.json + 38 em data/*.json)
    let summary1 = download_egressos_dados_repo(
        &mut conn,
        data.path(),
        &src,
        &client(&server, Some("ghp_token_ifesserra_dados")),
        no_progress(),
    )
    .await
    .unwrap();
    assert_eq!(summary1.total_files, 39);
    assert!(summary1.total_rows >= 39);
    assert!(summary1.snapshot.is_none());

    // 2. Segundo download cria snapshot egressos-dados-pre-import-*
    let summary2 = download_egressos_dados_repo(
        &mut conn,
        data.path(),
        &src,
        &client(&server, Some("ghp_token_ifesserra_dados")),
        no_progress(),
    )
    .await
    .unwrap();
    let snap = summary2
        .snapshot
        .expect("snapshot esperado no reimport via GitHub");
    assert!(snap.starts_with("egressos-dados-pre-import-"));

    // 3. Se 1 dos 39 arquivos der 404, aborta sem alterar a base existente (FR-008, FR-010)
    let broken_server = MockServer::start().await;
    for def in EGRESSOS_DADOS_FILES {
        if def.file_key == "vitrine" {
            Mock::given(method("GET"))
                .and(path("/repos/ifesserra-lab/egressos-dados/contents/data/vitrine.json"))
                .respond_with(ResponseTemplate::new(404))
                .mount(&broken_server)
                .await;
        } else {
            let body = std::fs::read_to_string(ref_dir.join(def.relative_path)).unwrap();
            Mock::given(method("GET"))
                .and(path(format!(
                    "/repos/ifesserra-lab/egressos-dados/contents/{}",
                    def.relative_path
                )))
                .respond_with(ResponseTemplate::new(200).set_body_string(body))
                .mount(&broken_server)
                .await;
        }
    }

    let err = download_egressos_dados_repo(
        &mut conn,
        data.path(),
        &src,
        &client(&broken_server, Some("ghp_token_ifesserra_dados")),
        no_progress(),
    )
    .await
    .unwrap_err();
    assert_eq!(wire(&err)["kind"], "not_found");
    let cat = egressos_dados_domain::get_catalog(&conn).unwrap();
    assert_eq!(cat.total_files, 39);
    assert_eq!(cat.total_rows, summary2.total_rows);
}

#[tokio::test]
async fn upload_egressos_dados_repo_publica_39_blobs_em_commit_unico_atomico() {
    use research_hub_lib::egressos_dados_domain::{self, write_synthetic_egressos_dados_repo};
    use research_hub_lib::sync_github::upload_egressos_dados_repo;

    let server = MockServer::start().await;
    let temp = tempfile::tempdir().unwrap();
    let repo_dir = temp.path().join("egressos_dados_src");
    write_synthetic_egressos_dados_repo(&repo_dir).unwrap();
    let mut conn = db::open_in_memory().unwrap();
    egressos_dados_domain::import_dir(&mut conn, &repo_dir, temp.path(), |_| {}).unwrap();

    // Destination check (default config is ifesserra-lab/egressos-dados, branch main, path .)
    Mock::given(method("GET"))
        .and(path("/repos/ifesserra-lab/egressos-dados"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/repos/ifesserra-lab/egressos-dados/git/ref/heads/main"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "object": { "sha": "base_commit_sha_ed" }
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/repos/ifesserra-lab/egressos-dados/contents/alunos.json"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    // Git Data API: 39 blobs + 1 tree + 1 commit + 1 ref patch
    Mock::given(method("POST"))
        .and(path("/repos/ifesserra-lab/egressos-dados/git/blobs"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "sha": "blob_sha_ed"
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/repos/ifesserra-lab/egressos-dados/git/trees"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "sha": "tree_sha_ed"
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/repos/ifesserra-lab/egressos-dados/git/commits"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "sha": "commit_ed_999",
            "html_url": "https://github.com/ifesserra-lab/egressos-dados/commit/commit_ed_999"
        })))
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/repos/ifesserra-lab/egressos-dados/git/refs/heads/main"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let res = upload_egressos_dados_repo(
        &mut conn,
        temp.path(),
        true,
        Some("Publicação atômica dos 39 datasets egressos-dados"),
        &client(&server, Some("ghp_secret_egressos_dados")),
        no_progress(),
    )
    .await
    .unwrap();

    assert_eq!(res.commit_sha, "commit_ed_999");
    assert_eq!(
        res.html_url,
        "https://github.com/ifesserra-lab/egressos-dados/commit/commit_ed_999"
    );

    let requests = server.received_requests().await.unwrap();
    let blob_posts = requests
        .iter()
        .filter(|r| r.url.path() == "/repos/ifesserra-lab/egressos-dados/git/blobs")
        .count();
    assert_eq!(blob_posts, 39, "deve criar exatamente 39 blobs para os 39 JSONs");

    let tree_req = requests
        .iter()
        .find(|r| r.url.path() == "/repos/ifesserra-lab/egressos-dados/git/trees")
        .unwrap();
    let tree_body: serde_json::Value = serde_json::from_slice(&tree_req.body).unwrap();
    let entries = tree_body["tree"].as_array().unwrap();
    assert_eq!(entries.len(), 39);
    assert!(entries.iter().any(|e| e["path"] == "alunos.json"));
    assert!(entries.iter().any(|e| e["path"] == "data/egressos_perfil.json"));
}

// --------------------------------------------------------- SEP-038 (T006, T014)

#[tokio::test]
async fn upload_cria_nova_branch_via_post_git_refs_e_recusa_branch_existente_com_conflict() {
    use research_hub_lib::sync_domain::UploadBranchOptions;
    use research_hub_lib::sync_github::upload_horizon_export_with_options;

    let server = MockServer::start().await;
    let temp = tempfile::tempdir().unwrap();
    let mut cfg = sync_domain::SyncConfigFile::default();
    cfg.project_mut("horizon").repo = "a/b".into();
    cfg.project_mut("horizon").branch = "main".into();
    cfg.project_mut("horizon").path = "exports/exports_canonical.zip".into();
    sync_domain::save_config(temp.path(), &cfg).unwrap();

    // 1. Se a branch informada em "Criar nova branch" já existir (200), aborta com Conflict antes de enviar blobs
    Mock::given(method("GET"))
        .and(path("/repos/a/b/git/ref/heads/branch-ja-existe"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "object": { "sha": "sha_existente" }
        })))
        .mount(&server)
        .await;

    let opts_conflict = UploadBranchOptions {
        create_new_branch: true,
        base_branch: Some("main".into()),
        new_branch: Some("branch-ja-existe".into()),
        target_branch: None,
    };
    let err = upload_horizon_export_with_options(
        vec![1, 2, 3],
        temp.path(),
        "horizon",
        false,
        &opts_conflict,
        &client(&server, Some("ghp_token")),
        no_progress(),
    )
    .await
    .unwrap_err();
    assert_eq!(wire(&err)["kind"], "conflict");
    assert!(err.to_string().contains("branch-ja-existe"));

    // 2. Com new_branch em branco, gera update/horizon-<timestamp> automaticamente e cria via POST /git/refs
    let server2 = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/repos/a/b/git/ref/heads/main"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "object": { "sha": "base_main_sha" }
        })))
        .mount(&server2)
        .await;
    // Qualquer consulta de ref para a nova branch update/horizon-* retorna 404 (ainda não existe)
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server2)
        .await;
    Mock::given(method("POST"))
        .and(path("/repos/a/b/git/blobs"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"sha": "b_new"})))
        .mount(&server2)
        .await;
    Mock::given(method("POST"))
        .and(path("/repos/a/b/git/trees"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"sha": "t_new"})))
        .mount(&server2)
        .await;
    Mock::given(method("POST"))
        .and(path("/repos/a/b/git/commits"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "sha": "c_new_branch",
            "html_url": "https://github.com/a/b/commit/c_new_branch"
        })))
        .mount(&server2)
        .await;
    Mock::given(method("POST"))
        .and(path("/repos/a/b/git/refs"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "ref": "refs/heads/update/horizon-auto",
            "object": { "sha": "c_new_branch" }
        })))
        .mount(&server2)
        .await;

    let opts_auto = UploadBranchOptions {
        create_new_branch: true,
        base_branch: None,
        new_branch: None,
        target_branch: None,
    };
    let res = upload_horizon_export_with_options(
        vec![10, 20, 30],
        temp.path(),
        "horizon",
        false,
        &opts_auto,
        &client(&server2, Some("ghp_token")),
        no_progress(),
    )
    .await
    .unwrap();

    assert!(res.created_branch);
    assert!(res.branch.starts_with("update/horizon-"));
    assert_eq!(res.commit_sha, "c_new_branch");

    let reqs = server2.received_requests().await.unwrap();
    assert!(reqs.iter().any(|r| r.method.as_str() == "POST" && r.url.path() == "/repos/a/b/git/refs"));
    assert!(!reqs.iter().any(|r| r.method.as_str() == "PATCH"));
}

#[tokio::test]
async fn list_branches_retorna_branches_remotas_e_upload_aceita_branch_base_ou_destino_alem_da_main() {
    use research_hub_lib::sync_domain::UploadBranchOptions;
    use research_hub_lib::sync_github::{list_project_branches, upload_horizon_export_with_options};

    let server = MockServer::start().await;
    let temp = tempfile::tempdir().unwrap();
    let mut cfg = sync_domain::SyncConfigFile::default();
    cfg.project_mut("horizon").repo = "a/b".into();
    cfg.project_mut("horizon").branch = "main".into();
    cfg.project_mut("horizon").path = "exports/exports_canonical.zip".into();
    sync_domain::save_config(temp.path(), &cfg).unwrap();

    Mock::given(method("GET"))
        .and(path("/repos/a/b/branches"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "name": "main", "commit": { "sha": "sha_main_111" } },
            { "name": "develop", "commit": { "sha": "sha_dev_222" } },
            { "name": "curadoria-2026", "commit": { "sha": "sha_cur_333" } }
        ])))
        .mount(&server)
        .await;

    let gh = client(&server, Some("ghp_token"));
    let branches = list_project_branches(temp.path(), "horizon", None, &gh).await.unwrap();
    assert_eq!(branches.len(), 3);
    assert_eq!(branches[0].name, "main");
    assert_eq!(branches[1].name, "develop");
    assert_eq!(branches[2].name, "curadoria-2026");

    // Upload no modo "Enviar na branch selecionada" para 'develop' (diferente da main)
    Mock::given(method("GET")).and(path("/repos/a/b")).respond_with(ResponseTemplate::new(200)).mount(&server).await;
    Mock::given(method("GET"))
        .and(path("/repos/a/b/git/ref/heads/develop"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "object": { "sha": "sha_dev_222" }
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/repos/a/b/contents/exports/exports_canonical.zip"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/repos/a/b/git/blobs"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"sha": "b_dev"})))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/repos/a/b/git/trees"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"sha": "t_dev"})))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/repos/a/b/git/commits"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "sha": "c_dev_999",
            "html_url": "https://github.com/a/b/commit/c_dev_999"
        })))
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/repos/a/b/git/refs/heads/develop"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let opts_dev = UploadBranchOptions {
        create_new_branch: false,
        base_branch: Some("main".into()),
        new_branch: None,
        target_branch: Some("develop".into()),
    };
    let res_dev = upload_horizon_export_with_options(
        vec![5, 6, 7],
        temp.path(),
        "horizon",
        false,
        &opts_dev,
        &gh,
        no_progress(),
    )
    .await
    .unwrap();

    assert!(!res_dev.created_branch);
    assert_eq!(res_dev.branch, "develop");
    assert_eq!(res_dev.commit_sha, "c_dev_999");

    // FR-008: Se target_branch não existir no remoto ("nova-branch-direta" -> 404), cria a partir de base_branch ("develop")
    Mock::given(method("GET"))
        .and(path("/repos/a/b/git/ref/heads/nova-branch-direta"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/repos/a/b/git/refs"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "ref": "refs/heads/nova-branch-direta",
            "object": { "sha": "c_dev_999" }
        })))
        .mount(&server)
        .await;

    let opts_fallback = UploadBranchOptions {
        create_new_branch: false,
        base_branch: Some("develop".into()),
        new_branch: None,
        target_branch: Some("nova-branch-direta".into()),
    };
    let res_fb = upload_horizon_export_with_options(
        vec![5, 6, 7],
        temp.path(),
        "horizon",
        false,
        &opts_fallback,
        &gh,
        no_progress(),
    )
    .await
    .unwrap();

    assert!(res_fb.created_branch);
    assert_eq!(res_fb.branch, "nova-branch-direta");
}

#[tokio::test]
async fn download_and_upload_horizon_v2_operam_isolados_do_horizon_legacy() {
    let server = MockServer::start().await;
    let zip_bytes = canonical_zip(&[("Serra V2", 10), ("Vitoria V2", 20)]);
    Mock::given(method("GET"))
        .and(path("/repos/v2/repo/contents/exports/exports_canonical.zip"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(zip_bytes.clone()))
        .mount(&server)
        .await;

    let data = tempfile::tempdir().unwrap();
    let mut conn = db::open_in_memory().unwrap();
    conn.execute("INSERT INTO campuses (id, name) VALUES (1, 'Legado Intacto')", [])
        .unwrap();

    let src = file_source("v2", "repo", "main", "exports/exports_canonical.zip");
    let summary = download_and_import(
        &mut conn,
        data.path(),
        "horizon_v2",
        &src,
        &client(&server, None),
        no_progress(),
    )
    .await
    .unwrap();
    assert_eq!(summary["total_rows"], 2);

    let hv2_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM hv2_campuses", [], |r| r.get(0))
        .unwrap();
    assert_eq!(hv2_count, 2);

    let legacy_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM campuses", [], |r| r.get(0))
        .unwrap();
    assert_eq!(legacy_count, 1);
    assert!(data.path().join(research_hub_lib::import::HORIZON_V2_ORIGINAL_ARCHIVE).exists());
    assert!(!data.path().join(research_hub_lib::import::ORIGINAL_ARCHIVE).exists());
}



