//! Managed state handed to Tauri commands (SEP-016, FR-012).

use crate::error::AppError;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

/// Authenticated admin. Populated in SEP-021; the slot exists here so commands
/// can already be written against a real gate.
#[derive(Clone, Debug)]
pub struct Session {
    pub username: String,
}

/// Organização GitHub obrigatória para acesso à aba `egressos-dados` (SEP-036).
pub const REQUIRED_EGRESSOS_DADOS_ORG: &str = "ifesserra-lab";

/// Sessão de autorização especial da aba `egressos-dados` (SEP-036, FR-008/FR-009).
/// Mantida exclusivamente em memória durante a sessão ativa; o campo `token`
/// NUNCA é serializado para fora do processo Rust.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EgressosDadosAuth {
    pub login: String,
    pub organization: String,
    pub token: String,
    pub token_hint: String,
}

/// Visão pública serializada via IPC para o front-end (`data-model.md` §1.2).
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct EgressosDadosAuthStatus {
    pub authorized: bool,
    pub login: Option<String>,
    pub organization: String,
    pub token_hint: Option<String>,
}

impl Default for EgressosDadosAuthStatus {
    fn default() -> Self {
        Self {
            authorized: false,
            login: None,
            organization: REQUIRED_EGRESSOS_DADOS_ORG.to_string(),
            token_hint: None,
        }
    }
}

pub struct AppState {
    /// Interactive connection, used by CRUD commands. Short-lived locks only.
    db: Mutex<Connection>,
    /// Kept so the ETL can open its own connection instead of holding this lock.
    pub db_path: PathBuf,
    pub session: Mutex<Option<Session>>,
    pub egressos_dados_auth: Mutex<Option<EgressosDadosAuth>>,
}

impl AppState {
    pub fn new(db: Connection, db_path: PathBuf) -> Self {
        Self {
            db: Mutex::new(db),
            db_path,
            session: Mutex::new(None),
            egressos_dados_auth: Mutex::new(None),
        }
    }

    pub fn db(&self) -> Result<MutexGuard<'_, Connection>, AppError> {
        Ok(self.db.lock()?)
    }

    /// Own connection for long operations, so imports never block the UI.
    pub fn etl_connection(&self) -> Result<Connection, AppError> {
        crate::db::open(&self.db_path)
    }
}

impl AppState {
    /// Reject the call unless a session is open (SEP-021).
    ///
    /// This makes authentication real for the first time: today the JWT is
    /// created at login and never validated by any endpoint, so any caller can
    /// read every table without credentials.
    pub fn require_session(&self) -> Result<Session, AppError> {
        self.session.lock()?.clone().ok_or(AppError::Unauthenticated)
    }

    pub fn open_session(&self, username: &str) -> Result<(), AppError> {
        *self.session.lock()? = Some(Session { username: username.to_string() });
        Ok(())
    }

    pub fn close_session(&self) -> Result<(), AppError> {
        *self.session.lock()? = None;
        *self.egressos_dados_auth.lock()? = None;
        Ok(())
    }

    pub fn current_session(&self) -> Result<Option<Session>, AppError> {
        Ok(self.session.lock()?.clone())
    }

    pub fn open_egressos_dados_session(
        &self,
        login: &str,
        organization: &str,
        token: &str,
    ) -> Result<EgressosDadosAuthStatus, AppError> {
        self.require_session()?;
        let hint = crate::sync_domain::token_hint(&Some(token.to_string())).unwrap_or_default();
        let auth = EgressosDadosAuth {
            login: login.to_string(),
            organization: organization.to_string(),
            token: token.to_string(),
            token_hint: hint.clone(),
        };
        *self.egressos_dados_auth.lock()? = Some(auth);
        Ok(EgressosDadosAuthStatus {
            authorized: true,
            login: Some(login.to_string()),
            organization: organization.to_string(),
            token_hint: Some(hint),
        })
    }

    pub fn close_egressos_dados_session(&self) -> Result<EgressosDadosAuthStatus, AppError> {
        *self.egressos_dados_auth.lock()? = None;
        Ok(EgressosDadosAuthStatus::default())
    }

    pub fn current_egressos_dados_status(&self) -> Result<EgressosDadosAuthStatus, AppError> {
        self.require_session()?;
        let guard = self.egressos_dados_auth.lock()?;
        match guard.as_ref() {
            Some(auth) if auth.organization == REQUIRED_EGRESSOS_DADOS_ORG => {
                Ok(EgressosDadosAuthStatus {
                    authorized: true,
                    login: Some(auth.login.clone()),
                    organization: auth.organization.clone(),
                    token_hint: Some(auth.token_hint.clone()),
                })
            }
            _ => Ok(EgressosDadosAuthStatus::default()),
        }
    }

    pub fn require_egressos_dados_session(&self) -> Result<EgressosDadosAuth, AppError> {
        self.require_session()?;
        let guard = self.egressos_dados_auth.lock()?;
        match guard.as_ref() {
            Some(auth) if auth.organization == REQUIRED_EGRESSOS_DADOS_ORG => Ok(auth.clone()),
            _ => Err(AppError::OrgUnauthorized(
                "Acesso restrito: autentique-se com um Token do GitHub de membro ativo da organização 'ifesserra-lab' para acessar egressos-dados.".into(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn state() -> AppState {
        AppState::new(db::open_in_memory().unwrap(), PathBuf::from(":memory:"))
    }

    #[test]
    fn commands_are_refused_until_login() {
        let s = state();
        assert!(matches!(s.require_session(), Err(AppError::Unauthenticated)));

        s.open_session("admin@admin.com").unwrap();
        assert_eq!(s.require_session().unwrap().username, "admin@admin.com");

        s.close_session().unwrap();
        assert!(matches!(s.require_session(), Err(AppError::Unauthenticated)));
    }

    #[test]
    fn session_status_reflects_login_state() {
        let s = state();
        assert!(s.current_session().unwrap().is_none());
        s.open_session("admin@admin.com").unwrap();
        assert!(s.current_session().unwrap().is_some());
    }

    #[test]
    fn egressos_dados_session_is_gated_in_memory_and_never_exposes_full_token() {
        let s = state();
        // Sem sessão local aberta -> Unauthenticated
        assert!(matches!(
            s.require_egressos_dados_session(),
            Err(AppError::Unauthenticated)
        ));

        s.open_session("admin@admin.com").unwrap();

        // Sessão local aberta, mas login especial ainda não feito -> OrgUnauthorized
        let default_status = s.current_egressos_dados_status().unwrap();
        assert!(!default_status.authorized);
        assert_eq!(default_status.login, None);
        assert_eq!(default_status.organization, "ifesserra-lab");
        assert_eq!(default_status.token_hint, None);
        assert!(matches!(
            s.require_egressos_dados_session(),
            Err(AppError::OrgUnauthorized(_))
        ));

        // Abre sessão especial da organização ifesserra-lab
        let status = s
            .open_egressos_dados_session("thiagodeps", "ifesserra-lab", "ghp_secret_token_9f2a")
            .unwrap();
        assert!(status.authorized);
        assert_eq!(status.login.as_deref(), Some("thiagodeps"));
        assert_eq!(status.organization, "ifesserra-lab");
        assert_eq!(status.token_hint.as_deref(), Some("9f2a"));

        // Serialização pública nunca contém o valor completo do token
        let serialized = serde_json::to_string(&status).unwrap();
        assert!(!serialized.contains("ghp_secret_token_9f2a"));

        // Guarda retorna a sessão em memória com o token para uso interno
        let active = s.require_egressos_dados_session().unwrap();
        assert_eq!(active.login, "thiagodeps");
        assert_eq!(active.organization, "ifesserra-lab");
        assert_eq!(active.token, "ghp_secret_token_9f2a");

        // Logout da aba limpa imediatamente o estado em memória
        let after_logout = s.close_egressos_dados_session().unwrap();
        assert!(!after_logout.authorized);
        assert!(matches!(
            s.require_egressos_dados_session(),
            Err(AppError::OrgUnauthorized(_))
        ));
    }

    #[test]
    fn general_logout_clears_egressos_dados_session_automatically() {
        let s = state();
        s.open_session("admin@admin.com").unwrap();
        s.open_egressos_dados_session("thiagodeps", "ifesserra-lab", "ghp_secret_token_9f2a")
            .unwrap();
        assert!(s.current_egressos_dados_status().unwrap().authorized);

        // Logout geral do aplicativo deve limpar também a autorização de egressos-dados
        s.close_session().unwrap();
        s.open_session("outro@admin.com").unwrap();
        assert!(!s.current_egressos_dados_status().unwrap().authorized);
        assert!(matches!(
            s.require_egressos_dados_session(),
            Err(AppError::OrgUnauthorized(_))
        ));
    }

    #[test]
    fn egressos_dados_gate_is_isolated_from_horizon_src_and_egressos_local_sessions() {
        let s = state();
        s.open_session("admin@admin.com").unwrap();

        // Enquanto egressos-dados está bloqueado (None), a sessão padrão para Horizon/SRC/Egressos
        // permanece 100% válida e acessível (FR-010, SC-004)
        assert!(s.require_session().is_ok());
        assert!(matches!(
            s.require_egressos_dados_session(),
            Err(AppError::OrgUnauthorized(_))
        ));

        // Desbloquear e depois bloquear egressos-dados não encerra a sessão principal
        s.open_egressos_dados_session("thiagodeps", "ifesserra-lab", "ghp_token_1234")
            .unwrap();
        s.close_egressos_dados_session().unwrap();
        assert_eq!(s.require_session().unwrap().username, "admin@admin.com");
    }
}
