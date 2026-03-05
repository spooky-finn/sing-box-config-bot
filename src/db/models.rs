use crate::db::enums::UserStatus;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::db::{user, vpn_uuid};

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize, Insertable)]
#[diesel(table_name = user)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub status: i32,
    pub created_at: String,
}

impl User {
    pub fn status_enum(&self) -> UserStatus {
        match self.status {
            0 => UserStatus::New,
            1 => UserStatus::Accepted,
            2 => UserStatus::Rejected,
            _ => UserStatus::New,
        }
    }
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize, Insertable)]
#[diesel(table_name = vpn_uuid)]
pub struct VpnUuid {
    pub uuid: String,
    pub user_id: i64,
}
