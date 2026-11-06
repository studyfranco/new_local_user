pub use sea_orm_migration::prelude::*;

mod m20261105_000001_initial_schema;
mod m20261106_000002_role_prompts_and_languages;
mod m20261106_000003_foreign_key_indexes;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20261105_000001_initial_schema::Migration),
            Box::new(m20261106_000002_role_prompts_and_languages::Migration),
            Box::new(m20261106_000003_foreign_key_indexes::Migration),
        ]
    }
}
