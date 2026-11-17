//! Cliente GitHub da sincronização (SEP-034) — o único lugar que fala HTTP.
//!
//! Base URL injetável: produção usa `https://api.github.com`, os testes
//! apontam para um servidor wiremock local (research R8 — a suíte nunca
//! depende de rede real). O token viaja apenas no header `Authorization`;
//! nenhuma mensagem de erro o inclui (FR-006).
//!
//! Download: Contents API raw / Release Assets API (research R2), em chunks,
//! respeitando o limite de transferência (Q3). Upload: Git Data API
//! blob→tree→commit→ref (research R3) — a branch NUNCA é criada (Q1).

use crate::error::AppError;
use crate::sync_domain::{NormalizedSource, SyncProgress, MAX_TRANSFER_BYTES};
use base64::Engine;
use std::path::{Path, PathBuf};

/// Base URL padrão da API pública do GitHub.
pub const GITHUB_API: &str = "https://api.github.com";

/// Organização obrigatória para desbloqueio da aba `egressos-dados` (SEP-036).
pub use crate::state::REQUIRED_EGRESSOS_DADOS_ORG;

/// Callback de progresso; o comando converte em evento Tauri `sync://progress`
/// (research R6). O módulo em si não conhece Tauri.
pub type OnProgress<'a> = &'a (dyn Fn(SyncProgress) + Send + Sync);

#[derive(Clone)]
pub struct GitHubClient {
    base_url: String,
    token: Option<String>,
    max_bytes: usize,
    http: reqwest::Client,
}

/// Resultado do envio (`US2` / `SEP-038`): confirmação com o commit gerado e a branch publicada.
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub struct UploadResult {
    pub commit_sha: String,
    pub html_url: String,
    pub replaced: bool,
    pub branch: String,
    pub created_branch: bool,
}

/// Resultado da checagem prévia do destino (US2-2 / T014).
#[derive(serde::Serialize, Clone, Debug, Default)]
pub struct DestinationInfo {
    pub branch_exists: bool,
    pub file_exists: bool,
    pub file_sha: Option<String>,
}

/// Informações do token testado (US3 / T018).
#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub struct TokenInfo {
    pub login: String,
    pub scopes: String,
    pub valid: bool,
}

/// Resultado da verificação de membresia ativa em organização GitHub (SEP-036, FR-003).
#[derive(serde::Serialize, Clone, Debug, PartialEq, Eq)]
pub struct OrgMembershipInfo {
    pub login: String,
    pub organization: String,
    pub state: String,
}

impl GitHubClient {
    pub fn new(base_url: &str, token: Option<String>) -> Self {
        Self::with_max_bytes(base_url, token, MAX_TRANSFER_BYTES)
    }

    /// Limite ajustável para os testes (T007 usa um limite pequeno em vez de
    /// alocar 100 MB no servidor de mentira).
    pub fn with_max_bytes(base_url: &str, token: Option<String>, max_bytes: usize) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            token,
            max_bytes,
            http: reqwest::Client::builder()
                .connect_timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("cliente HTTP padrão deve ser construtível"),
        }
    }

    fn auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let req = req
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header("User-Agent", "research-hub");
        match &self.token {
            Some(t) => req.bearer_auth(t),
            None => req,
        }
    }

    /// Traduz uma resposta de erro do GitHub para `AppError` acionável
    /// (FR-007). O corpo do GitHub pode citar o path, mas nunca o token —
    /// que não é parte de nenhuma URL (R4).
    async fn api_error(&self, resp: reqwest::Response) -> AppError {
        let status = resp.status().as_u16();
        let hint = match status {
            401 => "o GitHub rejeitou o acesso: configure um token de acesso válido \
                    (ou use uma URL de repositório público)."
                .to_string(),
            403 => "acesso negado pelo GitHub: verifique as permissões do token \
                    (leitura para download; leitura e escrita para envio) ou o \
                    limite de requisições — tente mais tarde."
                .to_string(),
            _ => {
                let detail = resp
                    .json::<serde_json::Value>()
                    .await
                    .ok()
                    .and_then(|v| v.get("message").and_then(|m| m.as_str()).map(String::from))
                    .unwrap_or_default();
                if detail.is_empty() {
                    format!("o GitHub respondeu {status}; tente novamente")
                } else {
                    format!("o GitHub respondeu {status}: {detail}")
                }
            }
        };
        AppError::GithubApi { status, hint }
    }

    // ------------------------------------------------------------ download

    /// Baixa a origem normalizada para um arquivo temporário, em chunks
    /// (FR-008/SC-001). Em qualquer falha o temporário é removido antes de
    /// retornar — a base local nunca é tocada aqui.
    pub async fn download_to_temp(
        &self,
        source: &NormalizedSource,
        project: &str,
        temp_dir: &Path,
        on_progress: OnProgress<'_>,
    ) -> Result<PathBuf, AppError> {
        std::fs::create_dir_all(temp_dir)?;

        on_progress(SyncProgress {
            operation: "download",
            project: project.into(),
            phase: "started",
            bytes_done: 0,
            bytes_total: None,
        });

        // Localiza o asset de release (2 chamadas) ou monta a URL de contents.
        let (url, accept) = match source {
            NormalizedSource::File { path, reference, .. } => {
                let url = format!(
                    "{}/repos/{}/{}/contents/{}?ref={}",
                    self.base_url,
                    source.owner(),
                    source.repo(),
                    path,
                    reference
                );
                (url, "application/vnd.github.raw".to_string())
            }
            NormalizedSource::ReleaseAsset { tag, asset, .. } => {
                let releases_url = format!(
                    "{}/repos/{}/{}/releases/tags/{}",
                    self.base_url,
                    source.owner(),
                    source.repo(),
                    tag
                );
                let resp = self
                    .auth(self.http.get(&releases_url))
                    .send()
                    .await
                    .map_err(network_error)?;
                if !resp.status().is_success() {
                    return Err(self.api_error(resp).await);
                }
                let release: serde_json::Value = resp
                    .json()
                    .await
                    .map_err(|e| AppError::Network(format!("resposta ilegível do GitHub: {e}")))?;
                let assets = release
                    .get("assets")
                    .and_then(|a| a.as_array())
                    .cloned()
                    .unwrap_or_default();
                let id = assets
                    .iter()
                    .find(|a| a.get("name").and_then(|n| n.as_str()) == Some(asset))
                    .and_then(|a| a.get("id").and_then(|i| i.as_u64()))
                    .ok_or_else(|| {
                        AppError::GithubApi {
                            status: 404,
                            hint: format!("o arquivo '{asset}' não existe na release '{tag}' deste repositório."),
                        }
                    })?;
                (
                    format!("{}/repos/{}/{}/releases/assets/{}", self.base_url, source.owner(), source.repo(), id),
                    "application/octet-stream".to_string(),
                )
            }
        };

        let resp = self
            .auth(self.http.get(&url))
            .header("Accept", accept)
            .send()
            .await
            .map_err(network_error)?;
        if !resp.status().is_success() {
            return Err(self.api_error(resp).await);
        }

        let total = resp.content_length();
        let file_name = format!(
            "sync-{}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0),
            source.file_name()
        );
        let dest = temp_dir.join(file_name);
        let mut file = std::fs::File::create(&dest)?;

        let mut resp = resp;
        let mut done: u64 = 0;
        loop {
            let chunk = match resp.chunk().await {
                Ok(Some(c)) => c,
                Ok(None) => break,
                Err(e) => {
                    let _ = std::fs::remove_file(&dest);
                    return Err(AppError::Network(format!(
                        "conexão interrompida durante o download: {e}"
                    )));
                }
            };
            done += chunk.len() as u64;
            if done as usize > self.max_bytes {
                let _ = std::fs::remove_file(&dest);
                return Err(AppError::TooLarge { limit_bytes: self.max_bytes });
            }
            use std::io::Write;
            file.write_all(&chunk)?;
            on_progress(SyncProgress {
                operation: "download",
                project: project.into(),
                phase: "transferring",
                bytes_done: done,
                bytes_total: total,
            });
        }

        Ok(dest)
    }

    // -------------------------------------------------------------- upload

    /// Lê a ponta atual da branch. 404 com repositório existente →
    /// `NotFound` (kind not_found) com contexto de branch — NUNCA cria a ref.
    async fn head_ref(&self, owner: &str, repo: &str, branch: &str) -> Result<String, AppError> {
        let url = format!("{}/repos/{}/{}/git/ref/heads/{}", self.base_url, owner, repo, branch);
        let resp = self.auth(self.http.get(&url)).send().await.map_err(network_error)?;
        match resp.status().as_u16() {
            200 => {
                let v: serde_json::Value = resp
                    .json()
                    .await
                    .map_err(|e| AppError::Network(format!("resposta ilegível do GitHub: {e}")))?;
                Ok(v.pointer("/object/sha")
                    .and_then(|s| s.as_str())
                    .unwrap_or_default()
                    .to_string())
            }
            404 => {
                // Repositório existe? Distingue repo ausente de branch ausente.
                let repo_url = format!("{}/repos/{}/{}", self.base_url, owner, repo);
                let probe = self
                    .auth(self.http.get(&repo_url))
                    .send()
                    .await
                    .map_err(network_error)?;
                if probe.status().as_u16() == 404 {
                    Err(AppError::GithubApi {
                        status: 404,
                        hint: "repositório não encontrado ou sem acesso".into(),
                    })
                } else {
                    Err(AppError::GithubApi {
                        status: 404,
                        hint: format!(
                            "a branch '{branch}' não existe no repositório {owner}/{repo}. \
                             Crie a branch no GitHub antes de enviar — o aplicativo \
                             nunca cria branches automaticamente."
                        ),
                    })
                }
            }
            s => Err(AppError::GithubApi {
                status: s,
                hint: format!("o GitHub respondeu {s} ao ler a branch; tente novamente"),
            }),
        }
    }

    /// Lista as branches existentes em um repositório GitHub (`SEP-038`, `FR-006`).
    pub async fn list_branches(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<crate::sync_domain::RemoteBranchInfo>, AppError> {
        let url = format!(
            "{}/repos/{}/{}/branches?per_page=100",
            self.base_url, owner, repo
        );
        let resp = self
            .auth(self.http.get(&url))
            .header(reqwest::header::CACHE_CONTROL, "no-cache")
            .send()
            .await
            .map_err(network_error)?;
        if resp.status().as_u16() == 404 {
            return Err(AppError::GithubApi {
                status: 404,
                hint: "repositório não encontrado ou sem acesso".into(),
            });
        }
        if !resp.status().is_success() {
            return Err(self.api_error(resp).await);
        }
        let items: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| AppError::Network(format!("resposta ilegível do GitHub: {e}")))?;
        let arr = items
            .as_array()
            .ok_or_else(|| AppError::Network("lista de branches inválida do GitHub".into()))?;
        let mut out = Vec::with_capacity(arr.len());
        for item in arr {
            if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                let commit_sha = item
                    .pointer("/commit/sha")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                out.push(crate::sync_domain::RemoteBranchInfo {
                    name: name.to_string(),
                    commit_sha,
                });
            }
        }
        Ok(out)
    }

    /// Verifica se uma referência `refs/heads/{branch}` já existe no repositório (`SEP-038`).
    pub async fn branch_exists(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
    ) -> Result<bool, AppError> {
        let url = format!(
            "{}/repos/{}/{}/git/ref/heads/{}",
            self.base_url, owner, repo, branch
        );
        let resp = self
            .auth(self.http.get(&url))
            .header(reqwest::header::CACHE_CONTROL, "no-cache")
            .send()
            .await
            .map_err(network_error)?;
        match resp.status().as_u16() {
            200 => Ok(true),
            404 => Ok(false),
            _ => Err(self.api_error(resp).await),
        }
    }

    /// Cria uma nova referência remota `refs/heads/{new_branch}` via Git Data API (`SEP-038`, `FR-007`).
    pub async fn create_branch_ref(
        &self,
        owner: &str,
        repo: &str,
        new_branch: &str,
        commit_sha: &str,
    ) -> Result<(), AppError> {
        let refs_url = format!("{}/repos/{}/{}/git/refs", self.base_url, owner, repo);
        let resp = self
            .auth(self.http.post(&refs_url))
            .json(&serde_json::json!({
                "ref": format!("refs/heads/{new_branch}"),
                "sha": commit_sha,
            }))
            .send()
            .await
            .map_err(network_error)?;
        let status = resp.status().as_u16();
        if status == 422 {
            return Err(AppError::Conflict(format!(
                "A branch '{new_branch}' já existe no repositório {owner}/{repo}. Escolha outro nome ou utilize o modo 'Enviar na branch selecionada'."
            )));
        }
        if status == 403 {
            return Err(AppError::GithubApi {
                status: 403,
                hint: "sem permissão de escrita; nada foi gravado".into(),
            });
        }
        if !resp.status().is_success() {
            return Err(self.api_error(resp).await);
        }
        Ok(())
    }

    /// Fluxo Git Data API completo (research R3). `confirm_overwrite` chega
    /// validado pelo comando; aqui é só a mecânica.
    pub async fn upload_file(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
        path: &str,
        bytes: Vec<u8>,
        message: &str,
        project: &str,
        on_progress: OnProgress<'_>,
    ) -> Result<UploadResult, AppError> {
        self.upload_file_with_branch(
            owner,
            repo,
            branch,
            branch,
            false,
            path,
            bytes,
            message,
            project,
            on_progress,
        )
        .await
    }

    /// Fluxo Git Data API completo com suporte a criação de nova branch (`SEP-038`, `FR-007`).
    #[allow(clippy::too_many_arguments)]
    pub async fn upload_file_with_branch(
        &self,
        owner: &str,
        repo: &str,
        base_branch: &str,
        publish_branch: &str,
        create_new_branch: bool,
        path: &str,
        bytes: Vec<u8>,
        message: &str,
        project: &str,
        on_progress: OnProgress<'_>,
    ) -> Result<UploadResult, AppError> {
        on_progress(SyncProgress {
            operation: "upload",
            project: project.into(),
            phase: "started",
            bytes_done: 0,
            bytes_total: Some(bytes.len() as u64),
        });

        // Limite ANTES de qualquer transferência (Q3/FR-010).
        if bytes.len() > self.max_bytes {
            return Err(AppError::TooLarge { limit_bytes: self.max_bytes });
        }

        if create_new_branch && self.branch_exists(owner, repo, publish_branch).await? {
            return Err(AppError::Conflict(format!(
                "A branch '{publish_branch}' já existe no repositório {owner}/{repo}. Escolha outro nome ou utilize o modo 'Enviar na branch selecionada'."
            )));
        }

        on_progress(SyncProgress {
            operation: "upload",
            project: project.into(),
            phase: "committing",
            bytes_done: bytes.len() as u64,
            bytes_total: Some(bytes.len() as u64),
        });

        // 1. blob
        let blob_url = format!("{}/repos/{}/{}/git/blobs", self.base_url, owner, repo);
        let content = base64::engine::general_purpose::STANDARD.encode(&bytes);
        let resp = self
            .auth(self.http.post(&blob_url))
            .json(&serde_json::json!({ "content": content, "encoding": "base64" }))
            .send()
            .await
            .map_err(network_error)?;
        if !resp.status().is_success() {
            return Err(self.api_error(resp).await);
        }
        let blob: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| AppError::Network(format!("resposta ilegível do GitHub: {e}")))?;
        let blob_sha = blob
            .get("sha")
            .and_then(|s| s.as_str())
            .ok_or_else(|| AppError::Network("resposta do blob sem sha".into()))?
            .to_string();

        // 2..5 com uma retomada: a ponta pode mudar entre a leitura e o PATCH
        // (422). Segunda falha → erro acionável, nada gravado pelo app.
        let mut attempt = 0;
        loop {
            let base_sha = self.head_ref(owner, repo, base_branch).await?;

            // tree
            let tree_url = format!("{}/repos/{}/{}/git/trees", self.base_url, owner, repo);
            let resp = self
                .auth(self.http.post(&tree_url))
                .json(&serde_json::json!({
                    "base_tree": base_sha,
                    "tree": [{ "path": path, "mode": "100644", "type": "blob", "sha": blob_sha }],
                }))
                .send()
                .await
                .map_err(network_error)?;
            if !resp.status().is_success() {
                return Err(self.api_error(resp).await);
            }
            let tree: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| AppError::Network(format!("resposta ilegível do GitHub: {e}")))?;
            let tree_sha = tree
                .get("sha")
                .and_then(|s| s.as_str())
                .ok_or_else(|| AppError::Network("resposta da tree sem sha".into()))?
                .to_string();

            // commit
            let commit_url = format!("{}/repos/{}/{}/git/commits", self.base_url, owner, repo);
            let resp = self
                .auth(self.http.post(&commit_url))
                .json(&serde_json::json!({ "message": message, "tree": tree_sha, "parents": [base_sha] }))
                .send()
                .await
                .map_err(network_error)?;
            if !resp.status().is_success() {
                return Err(self.api_error(resp).await);
            }
            let commit: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| AppError::Network(format!("resposta ilegível do GitHub: {e}")))?;
            let commit_sha = commit
                .get("sha")
                .and_then(|s| s.as_str())
                .unwrap_or_default()
                .to_string();
            let html_url = commit
                .get("html_url")
                .and_then(|s| s.as_str())
                .unwrap_or_default()
                .to_string();

            if create_new_branch {
                self.create_branch_ref(owner, repo, publish_branch, &commit_sha)
                    .await?;
                return Ok(UploadResult {
                    commit_sha,
                    html_url,
                    replaced: false,
                    branch: publish_branch.to_string(),
                    created_branch: true,
                });
            }

            // avanço da ref existente
            let ref_url = format!("{}/repos/{}/{}/git/refs/heads/{}", self.base_url, owner, repo, publish_branch);
            let resp = self
                .auth(self.http.patch(&ref_url))
                .json(&serde_json::json!({ "sha": commit_sha, "force": false }))
                .send()
                .await
                .map_err(network_error)?;
            let status = resp.status().as_u16();
            if status == 422 && attempt == 0 {
                attempt += 1;
                continue; // ponta mudou; re-ler e refazer tree/commit/ref
            }
            if status == 422 {
                return Err(AppError::GithubApi {
                    status: 422,
                    hint: "o repositório mudou durante o envio; tente novamente".into(),
                });
            }
            if status == 403 {
                return Err(AppError::GithubApi {
                    status: 403,
                    hint: "sem permissão de escrita; nada foi gravado".into(),
                });
            }
            if !resp.status().is_success() {
                return Err(self.api_error(resp).await);
            }

            return Ok(UploadResult {
                commit_sha,
                html_url,
                replaced: false,
                branch: publish_branch.to_string(),
                created_branch: false,
            });
        }
    }

    /// Publica múltiplos arquivos de um diretório (ex.: os 21 `.json` de `dados/`
    /// do projeto Egressos) em um único commit atômico via Git Data API (`FR-012`).
    pub async fn upload_directory(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
        dir: &str,
        files: &[(String, Vec<u8>)],
        message: &str,
        project: &str,
        on_progress: OnProgress<'_>,
    ) -> Result<UploadResult, AppError> {
        self.upload_directory_with_branch(
            owner,
            repo,
            branch,
            branch,
            false,
            dir,
            files,
            message,
            project,
            on_progress,
        )
        .await
    }

    /// Publica múltiplos arquivos de um diretório com suporte a criação de nova branch (`SEP-038`).
    #[allow(clippy::too_many_arguments)]
    pub async fn upload_directory_with_branch(
        &self,
        owner: &str,
        repo: &str,
        base_branch: &str,
        publish_branch: &str,
        create_new_branch: bool,
        dir: &str,
        files: &[(String, Vec<u8>)],
        message: &str,
        project: &str,
        on_progress: OnProgress<'_>,
    ) -> Result<UploadResult, AppError> {
        let total_bytes: usize = files.iter().map(|(_, b)| b.len()).sum();

        on_progress(SyncProgress {
            operation: "upload",
            project: project.into(),
            phase: "started",
            bytes_done: 0,
            bytes_total: Some(total_bytes as u64),
        });

        if total_bytes > self.max_bytes {
            return Err(AppError::TooLarge {
                limit_bytes: self.max_bytes,
            });
        }

        if create_new_branch && self.branch_exists(owner, repo, publish_branch).await? {
            return Err(AppError::Conflict(format!(
                "A branch '{publish_branch}' já existe no repositório {owner}/{repo}. Escolha outro nome ou utilize o modo 'Enviar na branch selecionada'."
            )));
        }

        let clean_dir = match dir.trim().trim_matches('/') {
            "." => "",
            other => other,
        };
        let blob_url = format!("{}/repos/{}/{}/git/blobs", self.base_url, owner, repo);
        let mut tree_entries = Vec::with_capacity(files.len());
        let mut bytes_sent: u64 = 0;

        for (file_name, bytes) in files {
            let content = base64::engine::general_purpose::STANDARD.encode(bytes);
            let resp = self
                .auth(self.http.post(&blob_url))
                .json(&serde_json::json!({ "content": content, "encoding": "base64" }))
                .send()
                .await
                .map_err(network_error)?;
            if !resp.status().is_success() {
                return Err(self.api_error(resp).await);
            }
            let blob: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| AppError::Network(format!("resposta ilegível do GitHub: {e}")))?;
            let blob_sha = blob
                .get("sha")
                .and_then(|s| s.as_str())
                .ok_or_else(|| AppError::Network("resposta do blob sem sha".into()))?
                .to_string();

            let entry_path = if clean_dir.is_empty() {
                file_name.clone()
            } else {
                format!("{clean_dir}/{file_name}")
            };
            tree_entries.push(serde_json::json!({
                "path": entry_path,
                "mode": "100644",
                "type": "blob",
                "sha": blob_sha,
            }));

            bytes_sent += bytes.len() as u64;
            on_progress(SyncProgress {
                operation: "upload",
                project: project.into(),
                phase: "committing",
                bytes_done: bytes_sent,
                bytes_total: Some(total_bytes as u64),
            });
        }

        let mut attempt = 0;
        loop {
            let base_sha = self.head_ref(owner, repo, base_branch).await?;

            let tree_url = format!("{}/repos/{}/{}/git/trees", self.base_url, owner, repo);
            let resp = self
                .auth(self.http.post(&tree_url))
                .json(&serde_json::json!({
                    "base_tree": base_sha,
                    "tree": tree_entries,
                }))
                .send()
                .await
                .map_err(network_error)?;
            if !resp.status().is_success() {
                return Err(self.api_error(resp).await);
            }
            let tree: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| AppError::Network(format!("resposta ilegível do GitHub: {e}")))?;
            let tree_sha = tree
                .get("sha")
                .and_then(|s| s.as_str())
                .ok_or_else(|| AppError::Network("resposta da tree sem sha".into()))?
                .to_string();

            let commit_url = format!("{}/repos/{}/{}/git/commits", self.base_url, owner, repo);
            let resp = self
                .auth(self.http.post(&commit_url))
                .json(&serde_json::json!({
                    "message": message,
                    "tree": tree_sha,
                    "parents": [base_sha],
                }))
                .send()
                .await
                .map_err(network_error)?;
            if !resp.status().is_success() {
                return Err(self.api_error(resp).await);
            }
            let commit: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| AppError::Network(format!("resposta ilegível do GitHub: {e}")))?;
            let commit_sha = commit
                .get("sha")
                .and_then(|s| s.as_str())
                .unwrap_or_default()
                .to_string();
            let html_url = commit
                .get("html_url")
                .and_then(|s| s.as_str())
                .unwrap_or_default()
                .to_string();

            if create_new_branch {
                self.create_branch_ref(owner, repo, publish_branch, &commit_sha)
                    .await?;
                return Ok(UploadResult {
                    commit_sha,
                    html_url,
                    replaced: false,
                    branch: publish_branch.to_string(),
                    created_branch: true,
                });
            }

            let ref_url = format!(
                "{}/repos/{}/{}/git/refs/heads/{}",
                self.base_url, owner, repo, publish_branch
            );
            let resp = self
                .auth(self.http.patch(&ref_url))
                .json(&serde_json::json!({ "sha": commit_sha, "force": false }))
                .send()
                .await
                .map_err(network_error)?;
            let status = resp.status().as_u16();
            if status == 422 && attempt == 0 {
                attempt += 1;
                continue;
            }
            if status == 422 {
                return Err(AppError::GithubApi {
                    status: 422,
                    hint: "o repositório mudou durante o envio; tente novamente".into(),
                });
            }
            if status == 403 {
                return Err(AppError::GithubApi {
                    status: 403,
                    hint: "sem permissão de escrita; nada foi gravado".into(),
                });
            }
            if !resp.status().is_success() {
                return Err(self.api_error(resp).await);
            }

            return Ok(UploadResult {
                commit_sha,
                html_url,
                replaced: false,
                branch: publish_branch.to_string(),
                created_branch: false,
            });
        }
    }

    /// Checagem prévia do destino (T014): flags em vez de erro para branch/
    /// arquivo ausentes — quem falha é o upload. Repo ausente → NotFound.
    pub async fn check_destination(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
        path: &str,
    ) -> Result<DestinationInfo, AppError> {
        let mut info = DestinationInfo::default();

        let repo_url = format!("{}/repos/{}/{}", self.base_url, owner, repo);
        let repo_resp = self
            .auth(self.http.get(&repo_url))
            .send()
            .await
            .map_err(network_error)?;
        if repo_resp.status().as_u16() == 404 {
            return Err(AppError::GithubApi {
                status: 404,
                hint: "repositório não encontrado ou sem acesso".into(),
            });
        }
        if !repo_resp.status().is_success() {
            return Err(self.api_error(repo_resp).await);
        }

        let ref_url = format!("{}/repos/{}/{}/git/ref/heads/{}", self.base_url, owner, repo, branch);
        let ref_resp = self.auth(self.http.get(&ref_url)).send().await.map_err(network_error)?;
        info.branch_exists = ref_resp.status().is_success();
        if !info.branch_exists && ref_resp.status().as_u16() != 404 {
            return Err(self.api_error(ref_resp).await);
        }

        if info.branch_exists {
            let check_path = match path.trim().trim_matches('/') {
                "" | "." => "alunos.json",
                other => other,
            };
            let contents_url = format!(
                "{}/repos/{}/{}/contents/{}?ref={}",
                self.base_url, owner, repo, check_path, branch
            );
            let resp = self
                .auth(self.http.get(&contents_url))
                .send()
                .await
                .map_err(network_error)?;
            match resp.status().as_u16() {
                200 => {
                    let v: serde_json::Value = resp
                        .json()
                        .await
                        .map_err(|e| AppError::Network(format!("resposta ilegível do GitHub: {e}")))?;
                    info.file_exists = true;
                    info.file_sha = if let Some(arr) = v.as_array() {
                        arr.first()
                            .and_then(|item| item.get("sha"))
                            .and_then(|s| s.as_str())
                            .map(String::from)
                    } else {
                        v.get("sha").and_then(|s| s.as_str()).map(String::from)
                    };
                }
                404 => {}
                s => {
                    let _ = &contents_url;
                    return Err(AppError::GithubApi {
                        status: s,
                        hint: format!("o GitHub respondeu {s} ao ler o arquivo de destino; tente novamente"),
                    });
                }
            }
        }

        Ok(info)
    }

    // ---------------------------------------------------------------- token

    /// Valida o token contra `GET /user` (US3). O valor nunca é ecoado.
    pub async fn test_token(&self) -> Result<TokenInfo, AppError> {
        let url = format!("{}/user", self.base_url);
        let resp = self.auth(self.http.get(&url)).send().await.map_err(network_error)?;
        if resp.status().as_u16() == 401 {
            return Err(AppError::GithubApi {
                status: 401,
                hint: "token inválido ou revogado; salve um token atualizado".into(),
            });
        }
        if !resp.status().is_success() {
            return Err(self.api_error(resp).await);
        }
        let scopes = resp
            .headers()
            .get("x-oauth-scopes")
            .and_then(|h| h.to_str().ok())
            .unwrap_or_default()
            .to_string();
        let v: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| AppError::Network(format!("resposta ilegível do GitHub: {e}")))?;
        Ok(TokenInfo {
            login: v.get("login").and_then(|l| l.as_str()).unwrap_or_default().to_string(),
            scopes,
            valid: true,
        })
    }

    /// Verifica se o token autentica um usuário válido no GitHub E se esse usuário
    /// é membro ativo (`state == "active"`) da organização `org` (ex.: `ifesserra-lab`).
    ///
    /// Fluxo (SEP-036, FR-003..FR-005, Clarificação Q3):
    /// 1. `GET /user` -> identifica `login` e `scopes` (401 -> token inválido/expirado).
    /// 2. `GET /user/memberships/orgs/{org}` -> verifica filiação autenticada (pública ou privada):
    ///    - `200 OK` com `state == "active"` -> autorizado.
    ///    - `200 OK` com `state == "pending"` -> recusado (convite pendente).
    ///    - `403 Forbidden` (sem escopo `read:org`) -> tenta fallback público `GET /orgs/{org}/members/{login}`:
    ///      - `204 No Content` -> autorizado (membro público ativo).
    ///      - Caso contrário -> recusado orientando habilitar o escopo `read:org`.
    ///    - `404 Not Found` -> tenta fallback público `GET /orgs/{org}/members/{login}` caso o token
    ///      não declare `read:org`/`admin:org`; se não for `204`, recusa informando que `@{login}`
    ///      não pertence à organização `{org}`.
    pub async fn verify_org_membership(&self, org: &str) -> Result<OrgMembershipInfo, AppError> {
        let token_info = self.test_token().await?;
        let login = token_info.login;
        if login.trim().is_empty() {
            return Err(AppError::GithubApi {
                status: 401,
                hint: "Não foi possível identificar o usuário do token no GitHub.".into(),
            });
        }

        let membership_url = format!("{}/user/memberships/orgs/{}", self.base_url, org);
        let resp = self
            .auth(self.http.get(&membership_url))
            .send()
            .await
            .map_err(network_error)?;

        match resp.status().as_u16() {
            200 => {
                let body: serde_json::Value = resp
                    .json()
                    .await
                    .map_err(|e| AppError::Network(format!("resposta ilegível do GitHub: {e}")))?;
                let state = body
                    .get("state")
                    .and_then(|s| s.as_str())
                    .unwrap_or_default()
                    .to_string();
                if state == "active" {
                    Ok(OrgMembershipInfo {
                        login,
                        organization: org.to_string(),
                        state: "active".into(),
                    })
                } else if state == "pending" {
                    Err(AppError::OrgUnauthorized(format!(
                        "Acesso negado: a conta '@{login}' possui um convite pendente na organização '{org}'. Aceite o convite em https://github.com/orgs/{org} antes de acessar egressos-dados."
                    )))
                } else {
                    Err(AppError::OrgUnauthorized(format!(
                        "Acesso negado: a conta '@{login}' não possui participação ativa na organização '{org}'."
                    )))
                }
            }
            403 => {
                // Token sem escopo read:org: tenta verificar membresia pública como fallback
                if self.check_public_org_member(org, &login).await? {
                    return Ok(OrgMembershipInfo {
                        login,
                        organization: org.to_string(),
                        state: "active".into(),
                    });
                }
                Err(AppError::OrgUnauthorized(format!(
                    "Não foi possível verificar sua filiação privada na organização '{org}' para '@{login}'. Habilite a permissão 'read:org' no seu Token do GitHub (ou torne sua participação pública em https://github.com/orgs/{org}/people)."
                )))
            }
            404 => {
                let has_org_scope = token_info
                    .scopes
                    .split(',')
                    .map(|s| s.trim())
                    .any(|s| s == "read:org" || s == "admin:org" || s == "write:org" || s == "user");

                if !has_org_scope && self.check_public_org_member(org, &login).await? {
                    return Ok(OrgMembershipInfo {
                        login,
                        organization: org.to_string(),
                        state: "active".into(),
                    });
                }

                Err(AppError::OrgUnauthorized(format!(
                    "Acesso negado: a conta GitHub '@{login}' não pertence à organização '{org}'. Somente membros ativos de https://github.com/orgs/{org}/dashboard têm acesso à aba egressos-dados."
                )))
            }
            401 => Err(AppError::GithubApi {
                status: 401,
                hint: "token inválido ou revogado; informe um token atualizado".into(),
            }),
            status => Err(AppError::GithubApi {
                status,
                hint: format!(
                    "o GitHub respondeu {status} ao verificar a organização '{org}'; tente novamente"
                ),
            }),
        }
    }

    async fn check_public_org_member(&self, org: &str, login: &str) -> Result<bool, AppError> {
        let public_url = format!("{}/orgs/{}/members/{}", self.base_url, org, login);
        let resp = self
            .auth(self.http.get(&public_url))
            .send()
            .await
            .map_err(network_error)?;
        Ok(resp.status().as_u16() == 204)
    }
}

/// reqwest::Error → `network` com dica acionável (FR-007). O Display do erro
/// não contém o token (ele só existe no header, que o Display não imprime).
fn network_error(e: reqwest::Error) -> AppError {
    AppError::Network(format!(
        "sem acesso a github.com — verifique sua conexão com a internet e tente de novo ({e})"
    ))
}

// --------------------------------------------------------------- importação

/// Download + import em um passo (T008): origem validada por projeto, bytes em
/// temporário, import EXISTENTE reutilizado (R7 — mesmas validações, snapshot
/// e tudo-ou-nada), temporário removido em qualquer desfecho.
///
/// Reusar o import manual por caminho de arquivo garante SC-004 por construção.
pub async fn download_and_import(
    conn: &mut rusqlite::Connection,
    data_dir: &Path,
    project: &str,
    source: &NormalizedSource,
    client: &GitHubClient,
    on_progress: OnProgress<'_>,
) -> Result<serde_json::Value, AppError> {
    crate::sync_domain::ensure_known_project(project)?;

    // FR-002/FR-009: cada projeto aceita só o seu formato.
    let expected = if project == "horizon" || project == "horizon_v2" {
        "zip"
    } else {
        "json"
    };
    let name = source.file_name().to_ascii_lowercase();
    if !name.ends_with(&format!(".{expected}")) {
        return Err(AppError::Validation(format!(
            "O projeto {project} espera um export .{expected}; a URL aponta para '{}'.",
            source.file_name()
        )));
    }

    let temp_dir = data_dir.join("sync_tmp");
    let temp_path = client.download_to_temp(source, project, &temp_dir, on_progress).await?;

    on_progress(SyncProgress {
        operation: "download",
        project: project.into(),
        phase: "importing",
        bytes_done: 0,
        bytes_total: None,
    });

    let result: Result<serde_json::Value, AppError> = match project {
        "horizon" => {
            let bytes = std::fs::read(&temp_path)?;
            let mut conn_ref = conn;
            crate::import::import_archive(&mut conn_ref, &bytes, data_dir, |_t| {})
                .and_then(|summary| {
                    serde_json::to_value(summary)
                        .map_err(|e| AppError::Internal(format!("resumo ilegível: {e}")))
                })
        }
        "horizon_v2" => {
            let bytes = std::fs::read(&temp_path)?;
            let mut conn_ref = conn;
            crate::import::import_horizon_v2_archive(&mut conn_ref, &bytes, data_dir, |_t| {})
                .and_then(|summary| {
                    serde_json::to_value(summary)
                        .map_err(|e| AppError::Internal(format!("resumo ilegível: {e}")))
                })
        }
        "src" => {
            let text = std::fs::read_to_string(&temp_path)?;
            let mut conn_ref = conn;
            crate::src_domain::import(&mut conn_ref, &text, data_dir)
                .and_then(|summary| {
                    serde_json::to_value(summary)
                        .map_err(|e| AppError::Internal(format!("resumo ilegível: {e}")))
                })
        }
        _ => unreachable!("projeto validado acima"),
    };

    let _ = std::fs::remove_file(&temp_path);
    result
}

/// Checagem do destino configurado para um projeto (T014).
pub async fn check_project_destination(
    data_dir: &Path,
    project: &str,
    client: &GitHubClient,
) -> Result<crate::sync_domain::CheckDestinationView, AppError> {
    crate::sync_domain::ensure_known_project(project)?;
    let file = crate::sync_domain::load_config(data_dir)?;
    let cfg = crate::sync_domain::effective_project_config(project, &file);
    if cfg.repo.is_empty() || cfg.branch.is_empty() || cfg.path.is_empty() {
        return Err(AppError::SyncConfig(
            "Destino não configurado para este projeto.".into(),
        ));
    }
    let (owner, repo) = cfg
        .repo
        .split_once('/')
        .ok_or_else(|| AppError::SyncConfig("Repositório deve estar no formato 'owner/name'.".into()))?;

    let info = client.check_destination(owner, repo, &cfg.branch, &cfg.path).await?;
    Ok(crate::sync_domain::CheckDestinationView {
        repo: cfg.repo.clone(),
        branch: cfg.branch.clone(),
        path: cfg.path.clone(),
        branch_exists: info.branch_exists,
        file_exists: info.file_exists,
        file_sha: info.file_sha,
    })
}

/// Lista as branches existentes do repositório configurado (ou informado em `repo_override`) para um projeto (`SEP-038`, `FR-006`).
pub async fn list_project_branches(
    data_dir: &Path,
    project: &str,
    repo_override: Option<&str>,
    client: &GitHubClient,
) -> Result<Vec<crate::sync_domain::RemoteBranchInfo>, AppError> {
    crate::sync_domain::ensure_known_project(project)?;
    let file = crate::sync_domain::load_config(data_dir)?;
    let cfg = crate::sync_domain::effective_project_config(project, &file);
    let effective_repo = repo_override
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(&cfg.repo);
    if effective_repo.is_empty() {
        return Err(AppError::SyncConfig(
            "Informe o repositório no formato 'owner/name' antes de carregar as branches.".into(),
        ));
    }
    crate::sync_domain::validate_repo(effective_repo)?;
    let (owner, repo) = effective_repo
        .split_once('/')
        .ok_or_else(|| AppError::SyncConfig("Repositório deve estar no formato 'owner/name'.".into()))?;
    client.list_branches(owner, repo).await
}

/// Envio do export canônico (Horizon `.zip` ou SRC `.json`) para o GitHub (T015 / `SEP-038`).
pub async fn upload_horizon_export(
    bytes: Vec<u8>,
    data_dir: &Path,
    project: &str,
    confirm_overwrite: bool,
    client: &GitHubClient,
    on_progress: OnProgress<'_>,
) -> Result<UploadResult, AppError> {
    upload_horizon_export_with_options(
        bytes,
        data_dir,
        project,
        confirm_overwrite,
        &crate::sync_domain::UploadBranchOptions::default(),
        client,
        on_progress,
    )
    .await
}

/// Envio do export de arquivo único (`horizon` ou `src`) com opções de criação/seleção de branch (`SEP-038`).
pub async fn upload_horizon_export_with_options(
    bytes: Vec<u8>,
    data_dir: &Path,
    project: &str,
    confirm_overwrite: bool,
    branch_options: &crate::sync_domain::UploadBranchOptions,
    client: &GitHubClient,
    on_progress: OnProgress<'_>,
) -> Result<UploadResult, AppError> {
    crate::sync_domain::check_upload_allowed(project)?;

    let file = crate::sync_domain::load_config(data_dir)?;
    let cfg = crate::sync_domain::effective_project_config(project, &file);
    if cfg.repo.is_empty() || cfg.branch.is_empty() || cfg.path.is_empty() {
        return Err(AppError::SyncConfig(
            "Destino não configurado para este projeto.".into(),
        ));
    }
    crate::sync_domain::validate_path(project, &cfg.path)?;
    let (owner, repo) = cfg
        .repo
        .split_once('/')
        .ok_or_else(|| AppError::SyncConfig("Repositório deve estar no formato 'owner/name'.".into()))?;

    let plan = branch_options.resolve(project, &cfg.branch)?;

    // Limite antes de qualquer chamada HTTP (Q3)
    if bytes.len() > client.max_bytes {
        return Err(AppError::TooLarge { limit_bytes: client.max_bytes });
    }

    let message = if project == "src" {
        "Export consolidado SRC ResearchHub"
    } else {
        "Export canônico ResearchHub"
    };

    if plan.create_new_branch {
        return client
            .upload_file_with_branch(
                owner,
                repo,
                &plan.base_branch,
                &plan.publish_branch,
                true,
                &cfg.path,
                bytes,
                message,
                project,
                on_progress,
            )
            .await;
    }

    // Modo enviar na branch selecionada
    let dest_info = client
        .check_destination(owner, repo, &plan.publish_branch, &cfg.path)
        .await?;

    // FR-008: se o curador informou uma target_branch que ainda não existe no remoto,
    // cria-a a partir da branch base (ex.: main) em vez de falhar.
    if !dest_info.branch_exists && branch_options.target_branch.is_some() {
        return client
            .upload_file_with_branch(
                owner,
                repo,
                &plan.base_branch,
                &plan.publish_branch,
                true,
                &cfg.path,
                bytes,
                message,
                project,
                on_progress,
            )
            .await;
    }

    if dest_info.file_exists {
        let current_sha = dest_info.file_sha.as_deref().unwrap_or_default();
        let new_sha = crate::sync_domain::git_blob_sha(&bytes);
        if current_sha != new_sha && !confirm_overwrite {
            return Err(AppError::Conflict(
                "O arquivo de destino já existe no repositório com conteúdo diferente. Confirme a sobrescrita para prosseguir.".into(),
            ));
        }
    }

    let mut res = client
        .upload_file_with_branch(
            owner,
            repo,
            &plan.publish_branch,
            &plan.publish_branch,
            false,
            &cfg.path,
            bytes,
            message,
            project,
            on_progress,
        )
        .await?;
    res.replaced = dest_info.file_exists;
    Ok(res)
}

/// Baixa os 21 arquivos `.json` da pasta `dados/` do Egressos no GitHub,
/// valida-os atomicamente, cria snapshot `egressos-pre-import-*` e importa
/// na base local (`SEP-035`, `FR-011`).
pub async fn download_egressos_dir(
    conn: &mut rusqlite::Connection,
    data_dir: &Path,
    source: &crate::sync_domain::EgressosDirSource,
    client: &GitHubClient,
    on_progress: OnProgress<'_>,
) -> Result<crate::egressos_domain::EgressosImportSummary, AppError> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_dir = data_dir.join("sync_tmp").join(format!("egressos-{stamp}"));
    std::fs::create_dir_all(&temp_dir)?;

    let clean_dir = source.dir.trim_matches('/');
    let mut download_err: Option<AppError> = None;

    for def in crate::egressos_domain::EGRESSOS_FILES {
        let remote_path = if clean_dir.is_empty() {
            def.file_name.to_string()
        } else {
            format!("{clean_dir}/{}", def.file_name)
        };
        let file_src = NormalizedSource::File {
            owner: source.owner.clone(),
            repo: source.repo.clone(),
            reference: source.reference.clone(),
            path: remote_path,
        };
        match client
            .download_to_temp(&file_src, "egressos", &temp_dir, on_progress)
            .await
        {
            Ok(downloaded_tmp) => {
                let canonical_dest = temp_dir.join(def.file_name);
                if let Err(e) = std::fs::rename(&downloaded_tmp, &canonical_dest) {
                    download_err = Some(AppError::Io(e));
                    break;
                }
            }
            Err(e) => {
                download_err = Some(e);
                break;
            }
        }
    }

    if let Some(err) = download_err {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Err(err);
    }

    on_progress(SyncProgress {
        operation: "download",
        project: "egressos".into(),
        phase: "importing",
        bytes_done: 0,
        bytes_total: None,
    });

    let import_res = crate::egressos_domain::import_dir(conn, &temp_dir, data_dir, |_| {});
    let _ = std::fs::remove_dir_all(&temp_dir);

    let mut summary = import_res?;
    summary.path = format!(
        "https://github.com/{}/{}/tree/{}/{}",
        source.owner, source.repo, source.reference, clean_dir
    );
    Ok(summary)
}

/// Gera os 21 arquivos `.json` em memória a partir da base Egressos atual e
/// envia-os ao GitHub em um único commit na branch configurada (`SEP-035`, `FR-012`).
pub async fn upload_egressos_dir(
    conn: &mut rusqlite::Connection,
    data_dir: &Path,
    confirm_overwrite: bool,
    client: &GitHubClient,
    on_progress: OnProgress<'_>,
) -> Result<UploadResult, AppError> {
    upload_egressos_dir_with_options(
        conn,
        data_dir,
        confirm_overwrite,
        &crate::sync_domain::UploadBranchOptions::default(),
        client,
        on_progress,
    )
    .await
}

/// Gera os 21 arquivos `.json` em memória a partir da base Egressos atual e
/// envia-os ao GitHub com suporte a criação de nova branch ou escolha de branch (`SEP-038`).
pub async fn upload_egressos_dir_with_options(
    conn: &mut rusqlite::Connection,
    data_dir: &Path,
    confirm_overwrite: bool,
    branch_options: &crate::sync_domain::UploadBranchOptions,
    client: &GitHubClient,
    on_progress: OnProgress<'_>,
) -> Result<UploadResult, AppError> {
    crate::sync_domain::check_upload_allowed("egressos")?;
    let files = crate::egressos_domain::export_files_in_memory(conn)?;

    let file = crate::sync_domain::load_config(data_dir)?;
    let cfg = crate::sync_domain::effective_project_config("egressos", &file);
    if cfg.repo.is_empty() || cfg.branch.is_empty() || cfg.path.is_empty() {
        return Err(AppError::SyncConfig(
            "Destino não configurado para o projeto Egressos.".into(),
        ));
    }
    let (owner, repo) = cfg
        .repo
        .split_once('/')
        .ok_or_else(|| AppError::SyncConfig("Repositório deve estar no formato 'owner/name'.".into()))?;

    let plan = branch_options.resolve("egressos", &cfg.branch)?;

    let total_bytes: usize = files.iter().map(|(_, b)| b.len()).sum();
    if total_bytes > client.max_bytes {
        return Err(AppError::TooLarge {
            limit_bytes: client.max_bytes,
        });
    }

    let message = "Export dados Egressos ResearchHub (21 arquivos JSON)";

    if plan.create_new_branch {
        return client
            .upload_directory_with_branch(
                owner,
                repo,
                &plan.base_branch,
                &plan.publish_branch,
                true,
                &cfg.path,
                &files,
                message,
                "egressos",
                on_progress,
            )
            .await;
    }

    let dest_info = client
        .check_destination(owner, repo, &plan.publish_branch, &cfg.path)
        .await?;

    if !dest_info.branch_exists && branch_options.target_branch.is_some() {
        return client
            .upload_directory_with_branch(
                owner,
                repo,
                &plan.base_branch,
                &plan.publish_branch,
                true,
                &cfg.path,
                &files,
                message,
                "egressos",
                on_progress,
            )
            .await;
    }

    if dest_info.file_exists && !confirm_overwrite {
        return Err(AppError::Conflict(
            "A pasta de destino já existe no repositório. Confirme a sobrescrita dos 21 arquivos JSON para prosseguir.".into(),
        ));
    }

    let mut res = client
        .upload_directory_with_branch(
            owner,
            repo,
            &plan.publish_branch,
            &plan.publish_branch,
            false,
            &cfg.path,
            &files,
            message,
            "egressos",
            on_progress,
        )
        .await?;
    res.replaced = dest_info.file_exists;
    Ok(res)
}

/// Baixa os 39 arquivos `.json` do repositório `egressos-dados` (`alunos.json` na raiz +
/// 38 arquivos em `data/*.json`), valida-os atomicamente, cria snapshot
/// `egressos-dados-pre-import-*` e importa na base dedicada (`SEP-037`, `FR-010`).
pub async fn download_egressos_dados_repo(
    conn: &mut rusqlite::Connection,
    data_dir: &Path,
    source: &crate::sync_domain::EgressosDirSource,
    client: &GitHubClient,
    on_progress: OnProgress<'_>,
) -> Result<crate::egressos_dados_domain::EgressosDadosImportSummary, AppError> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_dir = data_dir
        .join("sync_tmp")
        .join(format!("egressos-dados-{stamp}"));
    std::fs::create_dir_all(temp_dir.join("data"))?;

    let clean_dir = match source.dir.trim().trim_matches('/') {
        "." => "",
        other => other,
    };
    let mut download_err: Option<AppError> = None;

    for def in crate::egressos_dados_domain::EGRESSOS_DADOS_FILES {
        let remote_path = if clean_dir.is_empty() {
            def.relative_path.to_string()
        } else {
            format!("{clean_dir}/{}", def.relative_path)
        };
        let file_src = NormalizedSource::File {
            owner: source.owner.clone(),
            repo: source.repo.clone(),
            reference: source.reference.clone(),
            path: remote_path,
        };
        match client
            .download_to_temp(&file_src, "egressos_dados", &temp_dir, on_progress)
            .await
        {
            Ok(downloaded_tmp) => {
                let canonical_dest = temp_dir.join(def.relative_path);
                if let Some(parent) = canonical_dest.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if let Err(e) = std::fs::rename(&downloaded_tmp, &canonical_dest) {
                    download_err = Some(AppError::Io(e));
                    break;
                }
            }
            Err(e) => {
                download_err = Some(e);
                break;
            }
        }
    }

    if let Some(err) = download_err {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Err(err);
    }

    on_progress(SyncProgress {
        operation: "download",
        project: "egressos_dados".into(),
        phase: "importing",
        bytes_done: 0,
        bytes_total: None,
    });

    let import_res =
        crate::egressos_dados_domain::import_dir(conn, &temp_dir, data_dir, |_| {});
    let _ = std::fs::remove_dir_all(&temp_dir);

    let mut summary = import_res?;
    summary.path = if clean_dir.is_empty() {
        format!(
            "https://github.com/{}/{}/tree/{}",
            source.owner, source.repo, source.reference
        )
    } else {
        format!(
            "https://github.com/{}/{}/tree/{}/{}",
            source.owner, source.repo, source.reference, clean_dir
        )
    };
    Ok(summary)
}

/// Gera os 39 arquivos `.json` (`alunos.json` + 38 em `data/*.json`) em memória a partir
/// da base `egressos-dados` atual e envia-os ao GitHub em um único commit atômico (`SEP-037`, `FR-010`).
pub async fn upload_egressos_dados_repo(
    conn: &mut rusqlite::Connection,
    data_dir: &Path,
    confirm_overwrite: bool,
    commit_message: Option<&str>,
    client: &GitHubClient,
    on_progress: OnProgress<'_>,
) -> Result<UploadResult, AppError> {
    upload_egressos_dados_repo_with_options(
        conn,
        data_dir,
        confirm_overwrite,
        commit_message,
        &crate::sync_domain::UploadBranchOptions::default(),
        client,
        on_progress,
    )
    .await
}

/// Gera os 39 arquivos `.json` em memória a partir da base `egressos-dados` atual e
/// envia-os ao GitHub com suporte a criação de nova branch ou seleção de branch (`SEP-038`).
#[allow(clippy::too_many_arguments)]
pub async fn upload_egressos_dados_repo_with_options(
    conn: &mut rusqlite::Connection,
    data_dir: &Path,
    confirm_overwrite: bool,
    commit_message: Option<&str>,
    branch_options: &crate::sync_domain::UploadBranchOptions,
    client: &GitHubClient,
    on_progress: OnProgress<'_>,
) -> Result<UploadResult, AppError> {
    crate::sync_domain::check_upload_allowed("egressos_dados")?;
    let files = crate::egressos_dados_domain::export_files_in_memory(conn)?;

    let file = crate::sync_domain::load_config(data_dir)?;
    let cfg = crate::sync_domain::effective_project_config("egressos_dados", &file);
    if cfg.repo.is_empty() || cfg.branch.is_empty() || cfg.path.is_empty() {
        return Err(AppError::SyncConfig(
            "Destino não configurado para o projeto Egressos — Dados.".into(),
        ));
    }
    let (owner, repo) = cfg
        .repo
        .split_once('/')
        .ok_or_else(|| {
            AppError::SyncConfig("Repositório deve estar no formato 'owner/name'.".into())
        })?;

    let plan = branch_options.resolve("egressos_dados", &cfg.branch)?;

    let total_bytes: usize = files.iter().map(|(_, b)| b.len()).sum();
    if total_bytes > client.max_bytes {
        return Err(AppError::TooLarge {
            limit_bytes: client.max_bytes,
        });
    }

    let message = commit_message
        .map(str::trim)
        .filter(|m| !m.is_empty())
        .unwrap_or("Export dados Egressos-Dados ResearchHub (39 arquivos JSON)");

    if plan.create_new_branch {
        return client
            .upload_directory_with_branch(
                owner,
                repo,
                &plan.base_branch,
                &plan.publish_branch,
                true,
                &cfg.path,
                &files,
                message,
                "egressos_dados",
                on_progress,
            )
            .await;
    }

    let dest_info = client
        .check_destination(owner, repo, &plan.publish_branch, &cfg.path)
        .await?;

    if !dest_info.branch_exists && branch_options.target_branch.is_some() {
        return client
            .upload_directory_with_branch(
                owner,
                repo,
                &plan.base_branch,
                &plan.publish_branch,
                true,
                &cfg.path,
                &files,
                message,
                "egressos_dados",
                on_progress,
            )
            .await;
    }

    if dest_info.file_exists && !confirm_overwrite {
        return Err(AppError::Conflict(
            "O repositório de destino já possui arquivos. Confirme a sobrescrita dos 39 arquivos JSON para prosseguir.".into(),
        ));
    }

    let mut res = client
        .upload_directory_with_branch(
            owner,
            repo,
            &plan.publish_branch,
            &plan.publish_branch,
            false,
            &cfg.path,
            &files,
            message,
            "egressos_dados",
            on_progress,
        )
        .await?;
    res.replaced = dest_info.file_exists;
    Ok(res)
}

