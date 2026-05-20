use clap::{Parser, Subcommand};
use warehouse_core::CoreResult;
use warehouse_core::auth::token_provider::CliTokenProvider;
use warehouse_core::config::CoreConfig;
use warehouse_core::facade::CoreHandle;

/// Warehouse Client Core CLI — dev tool for diagnostics and testing.
#[derive(Parser, Debug)]
#[command(name = "warehouse-cli")]
#[command(version = warehouse_core::CLIENT_VERSION)]
#[command(about = "Offline-first warehouse client core CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Check local environment health
    HealthLocal,
    /// Show current configuration
    ShowConfig,
    /// Manage local database
    Db {
        #[command(subcommand)]
        action: DbCommands,
    },
    /// Check remote SyncServer health
    HealthRemote,
    /// Show auth context from profile
    AuthContext,
    /// Manage active site
    Site {
        #[command(subcommand)]
        action: SiteCommands,
    },
    /// Bootstrap local profile and working set
    Bootstrap,
    /// Pull latest data from SyncServer
    SyncPull,
    /// Search local catalog
    Catalog {
        #[command(subcommand)]
        action: CatalogCommands,
    },
    /// List local balances
    Balances {
        #[command(subcommand)]
        action: BalanceCommands,
    },
    /// List operations from SyncServer
    Operations {
        #[command(subcommand)]
        action: OperationCommands,
    },
    /// List temporary items
    TempItems {
        #[command(subcommand)]
        action: TempItemCommands,
    },
    /// List asset registers
    Assets {
        #[command(subcommand)]
        action: AssetCommands,
    },
    /// Manage operation drafts
    Draft {
        #[command(subcommand)]
        action: DraftCommands,
    },
    /// Manage outbox events
    Outbox {
        #[command(subcommand)]
        action: OutboxCommands,
    },
    /// Run sync engine (push/pull/bootstrap)
    Sync {
        #[command(subcommand)]
        action: SyncCommands,
    },
}

#[derive(Subcommand, Debug)]
enum DbCommands {
    Init {
        #[arg(short, long, default_value = "warehouse.db")]
        path: String,
    },
    Info {
        #[arg(short, long, default_value = "warehouse.db")]
        path: String,
    },
}

#[derive(Subcommand, Debug)]
enum SiteCommands {
    /// List available sites from profile
    List,
    /// Set active site
    Set {
        #[arg(required = true)]
        site_id: i32,
    },
}

#[derive(Subcommand, Debug)]
enum CatalogCommands {
    /// Search items by name or SKU
    Search {
        #[arg(required = true)]
        query: String,
    },
}

#[derive(Subcommand, Debug)]
enum BalanceCommands {
    /// List balances for a site
    List {
        #[arg(required = true)]
        site_id: i32,
    },
}

#[derive(Subcommand, Debug)]
enum OperationCommands {
    /// List operations for a site
    List {
        #[arg(required = true)]
        site_id: i32,
    },
    /// Get operation detail
    Get {
        #[arg(required = true)]
        operation_id: String,
    },
}

#[derive(Subcommand, Debug)]
enum TempItemCommands {
    /// List active temporary items
    List,
}

#[derive(Subcommand, Debug)]
enum AssetCommands {
    /// List pending acceptance balances
    Pending,
    /// List lost assets
    Lost,
    /// List issued assets
    Issued,
}

#[derive(Subcommand, Debug)]
enum DraftCommands {
    /// Create a new draft
    Create {
        #[arg(required = true)]
        operation_type: String,
        #[arg(short, long)]
        site_id: Option<i32>,
    },
    /// List all drafts
    List,
    /// Show draft detail
    Get {
        #[arg(required = true)]
        draft_id: String,
    },
    /// Delete a draft
    Delete {
        #[arg(required = true)]
        draft_id: String,
    },
    /// Clone a draft
    Clone {
        #[arg(required = true)]
        draft_id: String,
    },
    /// Add item line to draft
    AddLine {
        #[arg(required = true)]
        draft_id: String,
        #[arg(required = true)]
        item_id: i32,
        #[arg(required = true)]
        qty: f64,
    },
    /// Validate a draft
    Validate {
        #[arg(required = true)]
        draft_id: String,
    },
    /// Queue draft for submission via outbox
    Submit {
        #[arg(required = true)]
        draft_id: String,
    },
}

#[derive(Subcommand, Debug)]
enum OutboxCommands {
    /// List outbox events
    List,
    /// Get outbox event detail
    Get {
        #[arg(required = true)]
        event_uuid: String,
    },
    /// Retry a failed outbox event
    Retry {
        #[arg(required = true)]
        event_uuid: String,
    },
    /// Cancel a pending outbox event
    Cancel {
        #[arg(required = true)]
        event_uuid: String,
    },
    /// Send pending outbox events
    Send,
}

#[derive(Subcommand, Debug)]
enum SyncCommands {
    /// Bootstrap: identity + catalog + sites
    Bootstrap,
    /// Push only: send pending outbox events
    Push,
    /// Pull only: refresh all data families
    Pull,
    /// Full: push then pull
    Full,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::HealthLocal => {
            println!("warehouse-cli v{}", warehouse_core::CLIENT_VERSION);
            println!("Rust version: {}", rustc_version());
            println!("Status: OK");
            println!("Core crate loaded: OK");
        }
        Commands::ShowConfig => {
            let config = warehouse_core::CoreConfig::default();
            println!(
                "{}",
                serde_json::to_string_pretty(&config).unwrap_or_default()
            );
        }
        Commands::Db { action } => match action {
            DbCommands::Init { path } => {
                println!("Initializing database at: {}", path);
                match warehouse_core::storage::Database::open(&path, true).await {
                    Ok(_db) => println!("Database initialized successfully."),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            DbCommands::Info { path } => {
                println!("Database path: {}", path);
                match warehouse_core::storage::Database::open(&path, false).await {
                    Ok(db) => println!("Database open OK. Path: {}", db.path()),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        },
        Commands::HealthRemote => cmd_health_remote().await,
        Commands::AuthContext => cmd_auth_context().await,
        Commands::Site { action } => cmd_site(action).await,
        Commands::Bootstrap => cmd_bootstrap().await,
        Commands::SyncPull => cmd_sync_pull().await,
        Commands::Catalog { action } => cmd_catalog(action).await,
        Commands::Balances { action } => cmd_balances(action).await,
        Commands::Operations { action } => cmd_operations(action).await,
        Commands::TempItems { action } => cmd_temp_items(action).await,
        Commands::Assets { action } => cmd_assets(action).await,
        Commands::Draft { action } => cmd_draft(action).await,
        Commands::Outbox { action } => cmd_outbox(action).await,
        Commands::Sync { action } => cmd_sync(action).await,
    }
}

async fn build_handle() -> CoreResult<CoreHandle> {
    let config = CoreConfig::default();
    let mut handle = CoreHandle::open(config).await?;
    handle.set_token_provider(Box::new(CliTokenProvider));
    handle.load_profile().await?;
    Ok(handle)
}

async fn cmd_health_remote() {
    let config = CoreConfig::default();
    let mut handle = match CoreHandle::open(config).await {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Error: {e}");
            return;
        }
    };
    handle.set_token_provider(Box::new(CliTokenProvider));
    match handle.health_remote().await {
        Ok(true) => println!("Remote health: OK"),
        Ok(false) => println!("Remote health: UNHEALTHY"),
        Err(e) => eprintln!("Error: {e}"),
    }
}

async fn cmd_auth_context() {
    let mut handle = match build_handle().await {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Error: {e}");
            return;
        }
    };
    if !handle.profile().is_authenticated() {
        match handle.refresh_identity().await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error refreshing identity: {e}");
                return;
            }
        }
    }
    match handle.get_auth_context() {
        Ok(ctx) => {
            println!("User: {} ({})", ctx.user_name, ctx.user_email);
            println!("Role: {} (root: {})", ctx.role, ctx.is_root);
            println!("Device: {}", ctx.device_id);
            println!("Sites:");
            for s in &ctx.available_sites {
                println!("  {} — {} ({})", s.site_id, s.name, s.code);
            }
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}

async fn cmd_site(action: SiteCommands) {
    let mut handle = match build_handle().await {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Error: {e}");
            return;
        }
    };
    match action {
        SiteCommands::List => match handle.list_available_sites() {
            Ok(sites) => {
                for s in &sites {
                    let active = handle
                        .get_active_site()
                        .ok()
                        .flatten()
                        .map(|a| a == s.site_id)
                        .unwrap_or(false);
                    println!(
                        "  {} — {} ({}) {}",
                        s.site_id,
                        s.name,
                        s.code,
                        if active { "[ACTIVE]" } else { "" }
                    );
                }
            }
            Err(e) => eprintln!("Error: {e}"),
        },
        SiteCommands::Set { site_id } => match handle.set_active_site(site_id).await {
            Ok(_) => println!("Active site set to {site_id}"),
            Err(e) => eprintln!("Error: {e}"),
        },
    }
}

async fn cmd_bootstrap() {
    let mut handle = match build_handle().await {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Error: {e}");
            return;
        }
    };
    match handle.bootstrap().await {
        Ok(result) => {
            println!(
                "Bootstrap: {}",
                if result.success { "SUCCESS" } else { "FAILED" }
            );
            println!("Protocol: {}", result.protocol_version);
            println!("Families synced: {:?}", result.families_synced);
            if !result.errors.is_empty() {
                println!("Errors:");
                for e in &result.errors {
                    println!("  - {e}");
                }
            }
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}

async fn cmd_sync_pull() {
    let mut handle = match build_handle().await {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Error: {e}");
            return;
        }
    };
    let summary = handle.pull_once().await;
    println!(
        "Sync pull: {} families, {} items, {} errors",
        summary.families.len(),
        summary.total_items,
        summary.errors_count
    );
    for family in &summary.families {
        let status = if family.success { "OK" } else { "FAIL" };
        println!("  {status} {} ({} items)", family.name, family.items_count);
    }
}

async fn cmd_catalog(action: CatalogCommands) {
    let handle = match build_handle().await {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Error: {e}");
            return;
        }
    };
    match action {
        CatalogCommands::Search { query } => match handle.search_items(&query).await {
            Ok(items) => {
                println!("Found {} items:", items.len());
                for item in &items {
                    println!(
                        "  [{}] {} — {}, cat {}, active: {}",
                        item.id,
                        item.name,
                        item.sku.as_deref().unwrap_or("no sku"),
                        item.category_id,
                        item.is_active
                    );
                }
            }
            Err(e) => eprintln!("Error: {e}"),
        },
    }
}

async fn cmd_balances(action: BalanceCommands) {
    let handle = match build_handle().await {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Error: {e}");
            return;
        }
    };
    match action {
        BalanceCommands::List { site_id } => match handle.list_balances(site_id).await {
            Ok(rows) => {
                println!("Balances for site {site_id}:");
                for row in &rows {
                    println!("  item {} — qty {}", row.item_id, row.qty);
                }
            }
            Err(e) => eprintln!("Error: {e}"),
        },
    }
}

async fn cmd_operations(action: OperationCommands) {
    let mut handle = match build_handle().await {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Error: {e}");
            return;
        }
    };
    match action {
        OperationCommands::List { site_id } => match handle.list_operations(site_id, 1, 50).await {
            Ok(resp) => {
                println!("Operations (page {}/{}):", resp.page, resp.total_count);
                for op in &resp.items {
                    println!(
                        "  [{}] {:?} — {:?}, {} lines, {}",
                        op.id, op.operation_type, op.status, op.line_count, op.created_at
                    );
                }
            }
            Err(e) => eprintln!("Error: {e}"),
        },
        OperationCommands::Get { operation_id } => {
            match handle.get_operation(&operation_id).await {
                Ok(op) => {
                    println!(
                        "Operation {}: {:?} / {:?}",
                        op.id, op.operation_type, op.status
                    );
                    println!("  Site: {} ({})", op.site_id, op.site_code);
                    println!("  Created by: {}", op.created_by_user_name);
                    println!("  Lines:");
                    for line in &op.lines {
                        println!(
                            "    item {} — {} qty {}",
                            line.item_id.unwrap_or_default(),
                            line.item_name,
                            line.qty
                        );
                    }
                }
                Err(e) => eprintln!("Error: {e}"),
            }
        }
    }
}

async fn cmd_temp_items(action: TempItemCommands) {
    let handle = match build_handle().await {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Error: {e}");
            return;
        }
    };
    match action {
        TempItemCommands::List => match handle.list_temporary_items().await {
            Ok(items) => {
                println!("Active temporary items:");
                for item in &items {
                    println!(
                        "  [{}] {} — unit {}, status: {:?}",
                        item.id, item.name, item.unit_id, item.status
                    );
                }
            }
            Err(e) => eprintln!("Error: {e}"),
        },
    }
}

async fn cmd_assets(action: AssetCommands) {
    let handle = match build_handle().await {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Error: {e}");
            return;
        }
    };
    match action {
        AssetCommands::Pending => match handle.list_pending_acceptance().await {
            Ok(rows) => {
                println!("Pending acceptance:");
                for row in &rows {
                    println!(
                        "  op {} line {} — qty {}",
                        row.operation_id, row.operation_line_id, row.qty
                    );
                }
            }
            Err(e) => eprintln!("Error: {e}"),
        },
        AssetCommands::Lost => match handle.list_lost_assets().await {
            Ok(rows) => {
                println!("Lost assets:");
                for row in &rows {
                    println!(
                        "  op {} line {} — lost qty {}",
                        row.operation_id, row.operation_line_id, row.lost_qty
                    );
                }
            }
            Err(e) => eprintln!("Error: {e}"),
        },
        AssetCommands::Issued => match handle.list_issued_assets().await {
            Ok(rows) => {
                println!("Issued assets:");
                for row in &rows {
                    println!(
                        "  op {} line {} — qty {} issued to {}",
                        row.operation_id, row.operation_line_id, row.qty, row.issued_to_name
                    );
                }
            }
            Err(e) => eprintln!("Error: {e}"),
        },
    }
}

async fn cmd_draft(action: DraftCommands) {
    let handle = match build_handle().await {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Error: {e}");
            return;
        }
    };
    match action {
        DraftCommands::Create {
            operation_type,
            site_id,
        } => {
            let op_type = match operation_type.to_uppercase().as_str() {
                "RECEIVE" => warehouse_core::domain::operation::OperationType::Receive,
                "EXPENSE" => warehouse_core::domain::operation::OperationType::Expense,
                "WRITE_OFF" | "WRITEOFF" => {
                    warehouse_core::domain::operation::OperationType::WriteOff
                }
                "MOVE" => warehouse_core::domain::operation::OperationType::Move,
                "ADJUSTMENT" => warehouse_core::domain::operation::OperationType::Adjustment,
                "ISSUE" => warehouse_core::domain::operation::OperationType::Issue,
                "ISSUE_RETURN" | "ISSUERETURN" => {
                    warehouse_core::domain::operation::OperationType::IssueReturn
                }
                _ => {
                    eprintln!(
                        "Unknown operation type: {operation_type}. Valid: RECEIVE, EXPENSE, WRITE_OFF, MOVE, ADJUSTMENT, ISSUE, ISSUE_RETURN"
                    );
                    return;
                }
            };
            match handle.create_draft(op_type, site_id).await {
                Ok(draft) => println!(
                    "Created draft {} ({:?})",
                    draft.draft_id, draft.operation_type
                ),
                Err(e) => eprintln!("Error: {e}"),
            }
        }
        DraftCommands::List => match handle.list_drafts().await {
            Ok(drafts) => {
                if drafts.is_empty() {
                    println!("No drafts.");
                } else {
                    for d in &drafts {
                        println!(
                            "[{}] {:?} site={:?} {} lines",
                            d.draft_id,
                            d.operation_type,
                            d.site_id,
                            d.lines.len()
                        );
                    }
                }
            }
            Err(e) => eprintln!("Error: {e}"),
        },
        DraftCommands::Get { draft_id } => match handle.get_draft(&draft_id).await {
            Ok(Some(d)) => {
                println!("Draft: {}", d.draft_id);
                println!("  Type: {:?}", d.operation_type);
                println!("  Site: {:?}", d.site_id);
                println!("  Lines: {}", d.lines.len());
                println!("  Created: {}", d.created_at);
                println!("  Updated: {}", d.updated_at);
                for line in &d.lines {
                    println!(
                        "    [{}] item={:?} qty={}",
                        line.line_id, line.item_id, line.qty
                    );
                }
            }
            Ok(None) => println!("Draft not found"),
            Err(e) => eprintln!("Error: {e}"),
        },
        DraftCommands::Delete { draft_id } => match handle.delete_draft(&draft_id).await {
            Ok(_) => println!("Draft deleted"),
            Err(e) => eprintln!("Error: {e}"),
        },
        DraftCommands::Clone { draft_id } => match handle.clone_draft(&draft_id).await {
            Ok(Some(cloned)) => println!("Cloned as {}", cloned.draft_id),
            Ok(None) => println!("Draft not found"),
            Err(e) => eprintln!("Error: {e}"),
        },
        DraftCommands::AddLine {
            draft_id,
            item_id,
            qty,
        } => {
            let qty_val = serde_json::json!(qty);
            match handle
                .add_draft_item_line(&draft_id, item_id, qty_val, None, None)
                .await
            {
                Ok(Some(_)) => println!("Line added to {draft_id}"),
                Ok(None) => println!("Draft not found"),
                Err(e) => eprintln!("Error: {e}"),
            }
        }
        DraftCommands::Validate { draft_id } => match handle.validate_draft(&draft_id).await {
            Ok(_) => println!("Draft is valid"),
            Err(errors) => {
                println!("Validation errors:");
                for e in &errors {
                    println!("  - {e}");
                }
            }
        },
        DraftCommands::Submit { draft_id } => match handle.queue_draft_submit(&draft_id).await {
            Ok(uuid) => println!("Queued outbox event: {uuid}"),
            Err(e) => eprintln!("Error: {e}"),
        },
    }
}

async fn cmd_outbox(action: OutboxCommands) {
    let handle = match build_handle().await {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Error: {e}");
            return;
        }
    };
    match action {
        OutboxCommands::List => match handle.list_outbox_events(None, None).await {
            Ok(events) => {
                if events.is_empty() {
                    println!("No outbox events.");
                } else {
                    for ev in &events {
                        println!(
                            "[{}] {} site={} status={} retry={}",
                            ev.event_uuid, ev.event_type, ev.site_id, ev.status, ev.retry_count
                        );
                    }
                }
            }
            Err(e) => eprintln!("Error: {e}"),
        },
        OutboxCommands::Get { event_uuid } => match handle.get_outbox_event(&event_uuid).await {
            Ok(Some(ev)) => {
                println!("Event: {}", ev.event_uuid);
                println!("  Type: {} / Command: {:?}", ev.event_type, ev.command_type);
                println!("  Site: {} Status: {}", ev.site_id, ev.status);
                println!("  Retries: {}/{}", ev.retry_count, ev.max_retries);
                println!("  Last error: {:?}", ev.last_error);
                println!("  Created: {}", ev.created_at);
            }
            Ok(None) => println!("Event not found"),
            Err(e) => eprintln!("Error: {e}"),
        },
        OutboxCommands::Retry { event_uuid } => {
            match handle.retry_outbox_event(&event_uuid).await {
                Ok(_) => println!("Event queued for retry"),
                Err(e) => eprintln!("Error: {e}"),
            }
        }
        OutboxCommands::Cancel { event_uuid } => {
            match handle.cancel_outbox_event(&event_uuid).await {
                Ok(_) => println!("Event cancelled"),
                Err(e) => eprintln!("Error: {e}"),
            }
        }
        OutboxCommands::Send => {
            let mut handle = handle;
            match handle.send_outbox().await {
                Ok(result) => println!(
                    "Sent: accepted={} failed={} conflicts={}",
                    result.accepted, result.failed, result.conflicts
                ),
                Err(e) => eprintln!("Error: {e}"),
            }
        }
    }
}

async fn cmd_sync(action: SyncCommands) {
    use warehouse_core::sync::SyncMode;

    let mode = match action {
        SyncCommands::Bootstrap => SyncMode::Bootstrap,
        SyncCommands::Push => SyncMode::PushOnly,
        SyncCommands::Pull => SyncMode::PullOnly,
        SyncCommands::Full => SyncMode::Full,
    };

    let mut handle = match build_handle().await {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Error: {e}");
            return;
        }
    };

    if !handle.profile().is_authenticated() {
        match handle.refresh_identity().await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error refreshing identity: {e}");
                return;
            }
        }
    }

    let result = handle.sync_once(mode).await;
    println!(
        "Sync mode: {:?} — {}",
        result.mode,
        if result.success { "SUCCESS" } else { "FAILED" }
    );
    println!(
        "  Push: {} accepted, {} failed, {} conflicts",
        result.push_accepted, result.push_failed, result.push_conflicts
    );
    println!(
        "  Pull: {} items, {} errors",
        result.pull_items, result.pull_errors
    );
    println!(
        "  Bootstrap: {}",
        if result.bootstrap_ok { "OK" } else { "N/A" }
    );
    if let Some(e) = &result.error {
        println!("  Error: {e}");
    }
}

fn rustc_version() -> String {
    option_env!("RUSTC_VERSION")
        .unwrap_or("unknown")
        .to_string()
}
