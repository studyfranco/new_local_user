use crate::{
    entities::{cli_instance, orchestration_channel},
    migration::{Migrator, MigratorTrait},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ConnectOptions, Database, DatabaseConnection, DbErr, EntityTrait,
    PaginatorTrait, Set,
};
use std::{path::Path, time::Duration};
use uuid::Uuid;

pub async fn connect_and_migrate(
    database_url: &str,
    default_codex_ws: &str,
    default_goose_ws: &str,
    default_goose_secret: Option<String>,
    default_claude_rpc: &str,
    default_agy_rpc: &str,
    default_cwd: &str,
) -> Result<DatabaseConnection, DbErr> {
    // Ensure parent directory exists for file-backed SQLite URLs
    if let Some(rest) = database_url.strip_prefix("sqlite://") {
        let path_part = rest.split('?').next().unwrap_or(rest);
        if path_part != ":memory:" && !path_part.is_empty() {
            if let Some(parent) = Path::new(path_part).parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = std::fs::create_dir_all(parent);
                }
            }
        }
    }

    let mut opt = ConnectOptions::new(database_url.to_string());
    opt.max_connections(16)
        .min_connections(1)
        .connect_timeout(Duration::from_secs(10))
        .acquire_timeout(Duration::from_secs(10))
        .sqlx_logging(false);

    if database_url.contains(":memory:") || database_url.contains("mode=memory") {
        opt.max_connections(1).min_connections(1);
    }

    let db = Database::connect(opt).await?;
    Migrator::up(&db, None).await?;

    // Seed initial CLI instances if table is empty
    let count = cli_instance::Entity::find().count(&db).await?;
    if count == 0 {
        let now = Utc::now();
        let codex_id = Uuid::new_v4();
        let goose_id = Uuid::new_v4();
        let claude_id = Uuid::new_v4();
        let agy_id = Uuid::new_v4();

        let seeds = vec![
            cli_instance::ActiveModel {
                id: Set(codex_id),
                name: Set("Codex #1 (Principal)".to_string()),
                kind: Set("codex".to_string()),
                transport: Set("ws".to_string()),
                endpoint_url: Set(default_codex_ws.to_string()),
                bridge_url: Set(Some("http://codex:3287/rpc".to_string())),
                secret_token: Set(None),
                default_cwd: Set(default_cwd.to_string()),
                created_at: Set(now),
                updated_at: Set(now),
            },
            cli_instance::ActiveModel {
                id: Set(antigravity_id(agy_id)),
                name: Set("Antigravity #1 (Principal)".to_string()),
                kind: Set("antigravity".to_string()),
                transport: Set("rpc".to_string()),
                endpoint_url: Set(default_agy_rpc.to_string()),
                bridge_url: Set(Some(default_agy_rpc.to_string())),
                secret_token: Set(None),
                default_cwd: Set(default_cwd.to_string()),
                created_at: Set(now),
                updated_at: Set(now),
            },
            cli_instance::ActiveModel {
                id: Set(claude_id),
                name: Set("Claude Code #1 (Principal)".to_string()),
                kind: Set("claude".to_string()),
                transport: Set("rpc".to_string()),
                endpoint_url: Set(default_claude_rpc.to_string()),
                bridge_url: Set(Some(default_claude_rpc.to_string())),
                secret_token: Set(None),
                default_cwd: Set(default_cwd.to_string()),
                created_at: Set(now),
                updated_at: Set(now),
            },
            cli_instance::ActiveModel {
                id: Set(goose_id),
                name: Set("Goose #1 (ACP Serve)".to_string()),
                kind: Set("goose".to_string()),
                transport: Set("ws".to_string()),
                endpoint_url: Set(default_goose_ws.to_string()),
                bridge_url: Set(Some("http://goose:3288/rpc".to_string())),
                secret_token: Set(default_goose_secret),
                default_cwd: Set(default_cwd.to_string()),
                created_at: Set(now),
                updated_at: Set(now),
            },
        ];

        for item in seeds {
            let _ = item.insert(&db).await;
        }

        let chan_count = orchestration_channel::Entity::find().count(&db).await?;
        if chan_count == 0 {
            let participants = serde_json::to_string(&vec![codex_id, agy_id, claude_id, goose_id])
                .unwrap_or_else(|_| "[]".to_string());
            let default_chan = orchestration_channel::ActiveModel {
                id: Set(Uuid::new_v4()),
                name: Set("general".to_string()),
                description: Set(Some(
                    "Canal principal d'orchestration inter-agents".to_string(),
                )),
                participant_ids_json: Set(participants),
                created_at: Set(now),
            };
            let _ = default_chan.insert(&db).await;
        }
    }

    Ok(db)
}

fn antigravity_id(id: Uuid) -> Uuid {
    id
}
