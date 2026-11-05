use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "orchestration_channels")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub name: String,
    pub description: Option<String>,
    /// JSON array of CLI instance UUIDs participating in this channel
    pub participant_ids_json: String,
    /// Configurable retention in days (0 = unlimited)
    pub retention_days: i32,
    /// Configurable maximum number of messages retained in DB for this channel (0 = unlimited)
    pub max_messages: i32,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::channel_message::Entity")]
    ChannelMessage,
}

impl Related<super::channel_message::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ChannelMessage.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
