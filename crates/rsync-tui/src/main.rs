mod app;
mod event;
mod handler;
mod theme;
mod ui;

use std::io;
use std::sync::Arc;
use std::time::Duration;

use clap::{Parser, Subcommand};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use rsync_core::database::sqlite::Database;
use rsync_core::models::backup::InvocationTrigger;
use rsync_core::repository::sqlite::invocation::SqliteInvocationRepository;
use rsync_core::repository::sqlite::job::SqliteJobRepository;
use rsync_core::repository::sqlite::settings::SqliteSettingsRepository;
use rsync_core::repository::sqlite::snapshot::SqliteSnapshotRepository;
use rsync_core::repository::sqlite::statistics::SqliteStatisticsRepository;
use rsync_core::services::execution_handler::ExecutionEventHandler;
use rsync_core::services::job_executor::JobExecutor;
use rsync_core::services::job_service::JobService;
use rsync_core::services::retention_runner;
use rsync_core::services::running_jobs::RunningJobs;
use rsync_core::services::scheduler_backend::{InProcessScheduler, SchedulerBackend, SchedulerConfig};
use rsync_core::services::settings_service::SettingsService;
use rsync_core::services::statistics_service::StatisticsService;

use app::App;
use event::{AppEvent, EventLoop};
use handler::TuiEventHandler;

#[derive(Parser)]
#[command(name = "rsync-tui", about = "Terminal UI for Rsync Studio")]
struct Cli {
    /// Custom database path
    #[arg(long)]
    db_path: Option<String>,

    /// Custom log directory
    #[arg(long)]
    log_dir: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a single job non-interactively
    Run {
        /// Job ID to execute
        job_id: String,
    },
    /// List all jobs
    List,
}

fn main() -> io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    let cli = Cli::parse();

    // Determine paths — try the GUI's Tauri data directory first so both
    // frontends share the same database by default.
    let default_data_dir = resolve_data_dir();

    let db_path = cli.db_path.unwrap_or_else(|| {
        let dir = &default_data_dir;
        std::fs::create_dir_all(dir).ok();
        dir.join("rsync-studio.db")
            .to_str()
            .unwrap_or("rsync-studio.db")
            .to_string()
    });

    let default_log_dir = cli.log_dir.unwrap_or_else(|| {
        let dir = default_data_dir.join("logs");
        std::fs::create_dir_all(&dir).ok();
        dir.to_str().unwrap_or("logs").to_string()
    });

    // Open database and create services
    let database = Database::open(&db_path).expect("Failed to open database");
    let conn = database.conn();

    let jobs = Arc::new(SqliteJobRepository::new(conn.clone()));
    let invocations = Arc::new(SqliteInvocationRepository::new(conn.clone()));
    let snapshots = Arc::new(SqliteSnapshotRepository::new(conn.clone()));
    let statistics_repo = Arc::new(SqliteStatisticsRepository::new(conn.clone()));
    let settings_repo = Arc::new(SqliteSettingsRepository::new(conn));

    let job_service = Arc::new(JobService::new(jobs, invocations, snapshots));
    let statistics_service = Arc::new(StatisticsService::new(statistics_repo));
    let settings_service = Arc::new(SettingsService::new(settings_repo));
    let running_jobs = Arc::new(RunningJobs::new());

    let job_executor = Arc::new(JobExecutor::new(
        Arc::clone(&job_service),
        Arc::clone(&statistics_service),
        Arc::clone(&settings_service),
        Arc::clone(&running_jobs),
        default_log_dir,
    ));

    // Run retention on startup
    retention_runner::run_history_retention(&job_service, &settings_service);

    // Handle subcommands
    match cli.command {
        Some(Commands::Run { job_id }) => {
            run_single_job(&job_id, &job_executor, &job_service)?;
        }
        Some(Commands::List) => {
            list_jobs(&job_service)?;
        }
        None => {
            run_tui(
                job_executor,
                job_service,
                statistics_service,
                settings_service,
            )?;
        }
    }

    Ok(())
}

fn run_single_job(
    job_id_str: &str,
    job_executor: &Arc<JobExecutor>,
    job_service: &Arc<JobService>,
) -> io::Result<()> {
    let job_uuid = job_id_str
        .parse::<uuid::Uuid>()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid job ID: {}", e)))?;

    let job = job_service
        .get_job(&job_uuid)
        .map_err(|e| io::Error::new(io::ErrorKind::NotFound, format!("Job not found: {}", e)))?;

    println!("Running job '{}' ({})", job.name, job.id);

    // Create a channel-based handler that prints to stdout
    let (tx, rx) = std::sync::mpsc::channel();
    let handler: Arc<dyn ExecutionEventHandler> = Arc::new(TuiEventHandler::new(tx));

    let invocation_id = job_executor
        .execute(&job, InvocationTrigger::Manual, handler)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    println!("Invocation ID: {}", invocation_id);

    // Drain events until completion
    use handler::TuiEvent;
    loop {
        match rx.recv() {
            Ok(TuiEvent::LogLine(ll)) => {
                if ll.is_stderr {
                    eprintln!("{}", ll.line);
                } else {
                    println!("{}", ll.line);
                }
            }
            Ok(TuiEvent::Progress(p)) => {
                eprint!(
                    "\r{:.1}% | {} files | {} ",
                    p.percentage, p.files_transferred, p.transfer_rate
                );
            }
            Ok(TuiEvent::StatusChange(status)) => {
                eprintln!();
                match status.status {
                    rsync_core::models::job::JobStatus::Completed => {
                        println!("Job completed successfully.");
                    }
                    rsync_core::models::job::JobStatus::Failed => {
                        let msg = status
                            .error_message
                            .unwrap_or_else(|| "Unknown error".to_string());
                        eprintln!("Job failed: {}", msg);
                        std::process::exit(status.exit_code.unwrap_or(1));
                    }
                    rsync_core::models::job::JobStatus::Cancelled => {
                        eprintln!("Job cancelled.");
                        std::process::exit(130);
                    }
                    _ => {}
                }
                break;
            }
            Err(_) => break,
        }
    }

    Ok(())
}

fn list_jobs(job_service: &Arc<JobService>) -> io::Result<()> {
    let jobs = job_service
        .list_jobs()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    if jobs.is_empty() {
        println!("No jobs configured.");
        return Ok(());
    }

    println!("{:<38} {:<30} {:<10} {}", "ID", "Name", "Enabled", "Mode");
    println!("{}", "-".repeat(90));

    for job in &jobs {
        let mode = match &job.backup_mode {
            rsync_core::models::job::BackupMode::Mirror => "Mirror",
            rsync_core::models::job::BackupMode::Versioned { .. } => "Versioned",
            rsync_core::models::job::BackupMode::Snapshot { .. } => "Snapshot",
        };
        println!(
            "{:<38} {:<30} {:<10} {}",
            job.id,
            truncate(&job.name, 28),
            if job.enabled { "Yes" } else { "No" },
            mode
        );
    }

    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max.saturating_sub(3)])
    } else {
        s.to_string()
    }
}

fn run_tui(
    job_executor: Arc<JobExecutor>,
    job_service: Arc<JobService>,
    statistics_service: Arc<StatisticsService>,
    settings_service: Arc<SettingsService>,
) -> io::Result<()> {
    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Event loop
    let event_loop = EventLoop::new(Duration::from_millis(50));
    let job_sender = event_loop.job_sender();

    // Start scheduler
    let scheduler_sender = event_loop.job_sender();
    let handler_factory: Arc<dyn Fn() -> Arc<dyn ExecutionEventHandler> + Send + Sync> =
        Arc::new(move || Arc::new(TuiEventHandler::new(scheduler_sender.clone())));

    let scheduler = InProcessScheduler::new(
        SchedulerConfig::default(),
        Arc::clone(&job_executor),
        Arc::clone(&job_service),
        Arc::clone(&settings_service),
        handler_factory,
    );
    let _scheduler_handle = scheduler.start();

    // App state
    let mut app = App::new(
        job_executor,
        job_service,
        statistics_service,
        settings_service,
        job_sender,
    );

    // Main loop
    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        match event_loop.next() {
            AppEvent::Key(key) => {
                app.handle_key(key);
            }
            AppEvent::Job(event) => {
                app.handle_job_event(event);
            }
            AppEvent::Resize(_, _) => {
                // Terminal handles resize automatically
            }
            AppEvent::Tick => {
                // Periodic refresh could go here
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Terminal teardown
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

/// Resolve the data directory, preferring the GUI's Tauri directory if it exists
/// so both frontends share the same database.
fn resolve_data_dir() -> std::path::PathBuf {
    if let Some(data_dir) = dirs::data_dir() {
        // Tauri uses the bundle identifier as the directory name.
        // On macOS: ~/Library/Application Support/com.rsync-studio.app/
        // On Linux: ~/.local/share/com.rsync-studio.app/  (or similar)
        let tauri_dir = data_dir.join("com.rsync-studio.app");
        if tauri_dir.join("rsync-studio.db").exists() {
            return tauri_dir;
        }

        // Fallback: standalone TUI directory
        data_dir.join("rsync-studio")
    } else {
        std::path::PathBuf::from(".rsync-studio")
    }
}
