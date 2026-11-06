use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "available_languages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    /// ISO 639-1 language code (e.g. "en", "fr", "de", "es", "it")
    #[sea_orm(unique)]
    pub code: String,
    /// English name of the language (e.g. "English", "French", "German", "Spanish")
    pub name: String,
    /// Localized native name of the language (e.g. "English", "Français", "Deutsch", "Español")
    pub native_name: String,
    /// Whether this is the default fallback language of the system
    pub is_default: bool,
    /// Whether this language is active in the web interface
    pub is_active: bool,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        has_one = "super::language_pack::Entity",
        from = "Column::Code",
        to = "super::language_pack::Column::Lang"
    )]
    LanguagePack,
    #[sea_orm(
        has_many = "super::role_prompt::Entity",
        from = "Column::Code",
        to = "super::role_prompt::Column::Lang"
    )]
    RolePrompt,
}

impl Related<super::language_pack::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LanguagePack.def()
    }
}

impl Related<super::role_prompt::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RolePrompt.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

