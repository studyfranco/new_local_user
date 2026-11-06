use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub enum CliInstances {
    Table,
    Id,
    Name,
    Kind,
    Transport,
    EndpointUrl,
    BridgeUrl,
    SecretToken,
    DefaultCwd,
    Role,
    RolePrompt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum OrchestrationChannels {
    Table,
    Id,
    Name,
    Description,
    ParticipantIdsJson,
    RetentionDays,
    MaxMessages,
    CreatedAt,
}

#[derive(DeriveIden)]
pub enum ChannelMessages {
    Table,
    Id,
    ChannelId,
    SenderInstanceId,
    SenderName,
    RecipientName,
    Content,
    CreatedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CliInstances::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CliInstances::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(CliInstances::Name).string().not_null())
                    .col(ColumnDef::new(CliInstances::Kind).string().not_null())
                    .col(ColumnDef::new(CliInstances::Transport).string().not_null())
                    .col(
                        ColumnDef::new(CliInstances::EndpointUrl)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(CliInstances::BridgeUrl).string().null())
                    .col(ColumnDef::new(CliInstances::SecretToken).string().null())
                    .col(ColumnDef::new(CliInstances::DefaultCwd).string().not_null())
                    .col(
                        ColumnDef::new(CliInstances::Role)
                            .string()
                            .not_null()
                            .default("developer"),
                    )
                    .col(
                        ColumnDef::new(CliInstances::RolePrompt)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(CliInstances::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CliInstances::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(OrchestrationChannels::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OrchestrationChannels::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(OrchestrationChannels::Name)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(OrchestrationChannels::Description)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(OrchestrationChannels::ParticipantIdsJson)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(OrchestrationChannels::RetentionDays)
                            .integer()
                            .not_null()
                            .default(30),
                    )
                    .col(
                        ColumnDef::new(OrchestrationChannels::MaxMessages)
                            .integer()
                            .not_null()
                            .default(200),
                    )
                    .col(
                        ColumnDef::new(OrchestrationChannels::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ChannelMessages::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ChannelMessages::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ChannelMessages::ChannelId).uuid().not_null())
                    .col(
                        ColumnDef::new(ChannelMessages::SenderInstanceId)
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ChannelMessages::SenderName)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ChannelMessages::RecipientName)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ChannelMessages::Content).text().not_null())
                    .col(
                        ColumnDef::new(ChannelMessages::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_channel_messages_channel")
                            .from(ChannelMessages::Table, ChannelMessages::ChannelId)
                            .to(OrchestrationChannels::Table, OrchestrationChannels::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_channel_messages_sender_instance")
                            .from(ChannelMessages::Table, ChannelMessages::SenderInstanceId)
                            .to(CliInstances::Table, CliInstances::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_channel_messages_channel_id")
                    .table(ChannelMessages::Table)
                    .col(ChannelMessages::ChannelId)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_channel_messages_sender_instance_id")
                    .table(ChannelMessages::Table)
                    .col(ChannelMessages::SenderInstanceId)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_channel_messages_channel_created")
                    .table(ChannelMessages::Table)
                    .col(ChannelMessages::ChannelId)
                    .col(ChannelMessages::CreatedAt)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_cli_instances_role")
                    .table(CliInstances::Table)
                    .col(CliInstances::Role)
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
                    .name("idx_cli_instances_role")
                    .table(CliInstances::Table)
                    .to_owned(),
            )
            .await;
        let _ = manager
            .drop_index(
                Index::drop()
                    .name("idx_channel_messages_channel_created")
                    .table(ChannelMessages::Table)
                    .to_owned(),
            )
            .await;
        let _ = manager
            .drop_index(
                Index::drop()
                    .name("idx_channel_messages_sender_instance_id")
                    .table(ChannelMessages::Table)
                    .to_owned(),
            )
            .await;
        let _ = manager
            .drop_index(
                Index::drop()
                    .name("idx_channel_messages_channel_id")
                    .table(ChannelMessages::Table)
                    .to_owned(),
            )
            .await;
        manager
            .drop_table(Table::drop().table(ChannelMessages::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(OrchestrationChannels::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(CliInstances::Table).to_owned())
            .await?;
        Ok(())
    }
}
