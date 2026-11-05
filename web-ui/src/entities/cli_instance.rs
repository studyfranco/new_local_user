use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "cli_instances")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub name: String,
    /// "codex" | "goose" | "claude" | "antigravity"
    pub kind: String,
    /// "ws" | "unix" | "http" | "rpc"
    pub transport: String,
    /// e.g. "ws://codex-1:4500", "/run/codex/app.sock", "ws://goose:3284/acp", "http://antigravity:3286/rpc"
    pub endpoint_url: String,
    /// Optional bridge port URL for Web UI authentication & tmux capture (e.g. "http://codex-1:3287/rpc")
    pub bridge_url: Option<String>,
    /// Optional secret token / API key
    pub secret_token: Option<String>,
    pub default_cwd: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
