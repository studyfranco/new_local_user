use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "language_packs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    /// ISO 639-1 or short language identifier (e.g. "en", "fr", "de", "es")
    #[sea_orm(unique)]
    pub lang: String,
    /// Human readable name of the language (e.g. "English", "Français", "Deutsch", "Español")
    pub name: String,
    /// JSON string storing key-value pairs of UI translation strings
    #[sea_orm(column_type = "Text")]
    pub dictionary_json: String,
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
