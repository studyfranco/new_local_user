use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub enum ChannelMessages {
    Table,
    ChannelId,
    SenderInstanceId,
    CreatedAt,
}

#[derive(DeriveIden)]
pub enum CliInstances {
    Table,
    Role,
}

#[derive(DeriveIden)]
pub enum RolePrompts {
    Table,
    Role,
    Lang,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Foreign key index: channel_messages.channel_id -> orchestration_channels.id
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

        // 2. Foreign key index: channel_messages.sender_instance_id -> cli_instances.id
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

        // 3. Composite foreign key + ordering index: (channel_id, created_at)
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

        // 4. Join column index: cli_instances.role -> role_prompts.role
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

        // 5. Join column index: role_prompts.role -> cli_instances.role
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

        // 6. Join column index: role_prompts.lang -> language_packs.lang
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

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let _ = manager
            .drop_index(
                Index::drop()
                    .name("idx_role_prompts_lang")
                    .table(RolePrompts::Table)
                    .to_owned(),
            )
            .await;
        let _ = manager
            .drop_index(
                Index::drop()
                    .name("idx_role_prompts_role")
                    .table(RolePrompts::Table)
                    .to_owned(),
            )
            .await;
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
        Ok(())
    }
}

