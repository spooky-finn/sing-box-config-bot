use std::sync::Arc;

use serde::{Deserialize, Serialize};
use teloxide::types::CallbackQuery;
use tracing::{error, info};

use crate::{db::enums::UserStatus, ports::user::IUserRepo};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvitationCmd {
    #[serde(rename = "i")]
    pub user_id: i64,
    #[serde(rename = "s")]
    pub status: UserStatus,
}

impl InvitationCmd {
    pub fn new(user_id: i64, status: UserStatus) -> Self {
        Self { user_id, status }
    }

    pub fn parse(text: &str) -> Result<Self, String> {
        let cmd: InvitationCmd =
            serde_json::from_str(text).map_err(|e| format!("Failed to parse JSON: {}", e))?;
        Ok(cmd)
    }

    pub fn to_callback_data(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

pub struct AdminService {
    user_repo: Arc<dyn IUserRepo>,
    pub admin_id: i64,
}

impl AdminService {
    pub fn new(user_repo: Arc<dyn IUserRepo>, admin_id: i64) -> Self {
        Self {
            user_repo,
            admin_id,
        }
    }

    pub fn is_admin_callback(&self, msg: &CallbackQuery) -> Option<InvitationCmd> {
        if msg.from.id.0 as i64 != self.admin_id {
            return None;
        }

        let data = msg.data.as_ref()?;

        match InvitationCmd::parse(data) {
            Ok(cmd) => Some(cmd),
            Err(e) => {
                error!(error = %e, "Failed to parse admin command");
                None
            }
        }
    }

    pub async fn handle_admin_callback(
        &self,
        cmd: &InvitationCmd,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.user_repo.set_status(cmd.user_id, cmd.status)?;
        info!(user_id = cmd.user_id, status = ?cmd.status, "Admin callback handled");
        Ok(())
    }
}
