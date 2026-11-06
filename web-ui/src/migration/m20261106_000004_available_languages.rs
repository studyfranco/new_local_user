use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub enum AvailableLanguages {
    Table,
    Id,
    Code,
    Name,
    NativeName,
    IsDefault,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AvailableLanguages::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AvailableLanguages::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AvailableLanguages::Code)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(AvailableLanguages::Name).string().not_null())
                    .col(
                        ColumnDef::new(AvailableLanguages::NativeName)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AvailableLanguages::IsDefault)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(AvailableLanguages::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(AvailableLanguages::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AvailableLanguages::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_available_languages_code")
                    .table(AvailableLanguages::Table)
                    .col(AvailableLanguages::Code)
                    .unique()
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_available_languages_is_default")
                    .table(AvailableLanguages::Table)
                    .col(AvailableLanguages::IsDefault)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_available_languages_is_active")
                    .table(AvailableLanguages::Table)
                    .col(AvailableLanguages::IsActive)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .drop_index(
                Index::drop()
                    .name("idx_available_languages_is_active")
                    .table(AvailableLanguages::Table)
                    .to_owned(),
            )
            .await;
        let _ = manager
            .drop_index(
                Index::drop()
                    .name("idx_available_languages_is_default")
                    .table(AvailableLanguages::Table)
                    .to_owned(),
            )
            .await;
        let _ = manager
            .drop_index(
                Index::drop()
                    .name("idx_available_languages_code")
                    .table(AvailableLanguages::Table)
                    .to_owned(),
            )
            .await;
        manager
            .drop_table(Table::drop().table(AvailableLanguages::Table).to_owned())
            .await?;
        Ok(())
    }
}

