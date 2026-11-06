use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "role_prompts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    /// Role identifier (e.g. "architect", "developer", "reviewer", "tester", "security", "general")
    pub role: String,
    /// Language code (e.g. "en", "fr", "de")
    pub lang: String,
    /// Human-readable label with emoji (e.g. "🏗️ Software Architect")
    pub label: String,
    /// The system / role prompt text
    #[sea_orm(column_type = "Text")]
    pub prompt: String,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::available_language::Entity",
        from = "Column::Lang",
        to = "super::available_language::Column::Code"
    )]
    AvailableLanguage,
}

impl Related<super::available_language::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AvailableLanguage.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
