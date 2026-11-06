use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "channel_messages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub channel_id: Uuid,
    pub sender_instance_id: Option<Uuid>,
    pub sender_name: String,
    pub recipient_name: String,
    #[sea_orm(column_type = "Text")]
    pub content: String,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::orchestration_channel::Entity",
        from = "Column::ChannelId",
        to = "super::orchestration_channel::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    OrchestrationChannel,
    #[sea_orm(
        belongs_to = "super::cli_instance::Entity",
        from = "Column::SenderInstanceId",
        to = "super::cli_instance::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    CliInstance,
}

impl Related<super::orchestration_channel::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OrchestrationChannel.def()
    }
}

impl Related<super::cli_instance::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CliInstance.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
