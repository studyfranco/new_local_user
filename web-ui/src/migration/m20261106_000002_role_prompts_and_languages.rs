use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub enum RolePrompts {
    Table,
    Id,
    Role,
    Lang,
    Label,
    Prompt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum LanguagePacks {
    Table,
    Id,
    Lang,
    Name,
    DictionaryJson,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Role Prompts table
        manager
            .create_table(
                Table::create()
                    .table(RolePrompts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RolePrompts::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(RolePrompts::Role).string().not_null())
                    .col(
                        ColumnDef::new(RolePrompts::Lang)
                            .string()
                            .not_null()
                            .default("en"),
                    )
                    .col(ColumnDef::new(RolePrompts::Label).string().not_null())
                    .col(ColumnDef::new(RolePrompts::Prompt).text().not_null())
                    .col(
                        ColumnDef::new(RolePrompts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RolePrompts::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_role_prompts_role_lang")
                    .table(RolePrompts::Table)
                    .col(RolePrompts::Role)
                    .col(RolePrompts::Lang)
                    .unique()
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_role_prompts_lang")
                    .table(RolePrompts::Table)
                    .col(RolePrompts::Lang)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_role_prompts_role")
                    .table(RolePrompts::Table)
                    .col(RolePrompts::Role)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        // 2. Language Packs table
        manager
            .create_table(
                Table::create()
                    .table(LanguagePacks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(LanguagePacks::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(LanguagePacks::Lang)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(LanguagePacks::Name).string().not_null())
                    .col(
                        ColumnDef::new(LanguagePacks::DictionaryJson)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(LanguagePacks::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(LanguagePacks::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(LanguagePacks::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(RolePrompts::Table).to_owned())
            .await?;
        Ok(())
    }
}
