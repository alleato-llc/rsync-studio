use std::sync::Arc;
use std::time::Duration;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager, WindowEvent};

use chrono::Utc;

use rsync_core::implementations::database::Database;
use rsync_core::implementations::sqlite_invocation_repository::SqliteInvocationRepository;
use rsync_core::implementations::sqlite_job_repository::SqliteJobRepository;
use rsync_core::implementations::sqlite_snapshot_repository::SqliteSnapshotRepository;
use rsync_core::implementations::sqlite_statistics_repository::SqliteStatisticsRepository;
use rsync_core::models::backup::InvocationTrigger;
use rsync_core::services::job_service::JobService;
use rsync_core::services::scheduler;
use rsync_core::services::statistics_service::StatisticsService;

mod commands;
mod execution;
mod state;

use state::AppState;

/// How often the scheduler checks for due jobs (in seconds).
const SCHEDULER_CHECK_INTERVAL_SECS: u64 = 300; // 5 minutes

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&data_dir)?;

            let db_path = data_dir.join("rsync-desktop.db");
            let database =
                Database::open(db_path.to_str().expect("invalid db path"))
                    .expect("failed to open database");

            let conn = database.conn();
            let jobs = Arc::new(SqliteJobRepository::new(conn.clone()));
            let invocations = Arc::new(SqliteInvocationRepository::new(conn.clone()));
            let snapshots = Arc::new(SqliteSnapshotRepository::new(conn.clone()));
            let statistics_repo = Arc::new(SqliteStatisticsRepository::new(conn));

            let job_service = Arc::new(JobService::new(jobs, invocations, snapshots));
            let statistics_service = Arc::new(StatisticsService::new(statistics_repo));

            app.manage(AppState {
                _database: database,
                job_service,
                statistics_service,
                running_jobs: execution::RunningJobs::new(),
            });

            // --- System tray ---
            setup_tray(app)?;

            // --- Close-to-tray behavior ---
            let app_handle = app.handle().clone();
            let main_window = app.get_webview_window("main")
                .expect("main window not found");
            main_window.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    if let Some(win) = app_handle.get_webview_window("main") {
                        let _ = win.hide();
                    }
                }
            });

            // --- Scheduler background thread ---
            spawn_scheduler(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_jobs,
            commands::get_job,
            commands::create_job,
            commands::update_job,
            commands::delete_job,
            commands::get_job_history,
            commands::execute_job,
            commands::execute_job_dry_run,
            commands::cancel_job,
            commands::get_running_jobs,
            commands::list_snapshots,
            commands::delete_snapshot,
            commands::explain_command,
            commands::parse_command_to_job,
            commands::export_jobs,
            commands::import_jobs,
            commands::run_preflight,
            commands::get_statistics,
            commands::get_statistics_for_job,
            commands::export_statistics,
            commands::reset_statistics,
            commands::reset_statistics_for_job,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let show = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide Window", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &hide, &quit])?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().cloned().expect("no app icon"))
        .icon_as_template(false) // full-color icon in menu bar
        .tooltip("Rsync Desktop")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
            }
            "hide" => {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.hide();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click { .. } = event {
                let app = tray.app_handle();
                if let Some(win) = app.get_webview_window("main") {
                    if win.is_visible().unwrap_or(false) {
                        let _ = win.hide();
                    } else {
                        let _ = win.show();
                        let _ = win.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

fn spawn_scheduler(app_handle: tauri::AppHandle) {
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(SCHEDULER_CHECK_INTERVAL_SECS));

            let state: tauri::State<'_, AppState> = app_handle.state();
            let job_service = Arc::clone(&state.job_service);
            let statistics_service = Arc::clone(&state.statistics_service);

            let jobs = match job_service.list_jobs() {
                Ok(j) => j,
                Err(e) => {
                    log::error!("Scheduler: failed to list jobs: {}", e);
                    continue;
                }
            };

            let now = Utc::now();

            for job in &jobs {
                // Skip disabled jobs or jobs without a schedule
                if !job.enabled {
                    continue;
                }
                let schedule = match &job.schedule {
                    Some(s) if s.enabled => s,
                    _ => continue,
                };

                // Skip jobs that are currently running
                if state.running_jobs.is_running(&job.id) {
                    continue;
                }

                // Determine the last run time from history
                let last_run = job_service
                    .get_job_history(&job.id, 1)
                    .ok()
                    .and_then(|h| h.first().map(|inv| inv.started_at));

                if scheduler::is_job_due(schedule, last_run, now) {
                    log::info!("Scheduler: job '{}' ({}) is due, executing", job.name, job.id);
                    let _ = app_handle.emit("job-scheduled", &job.id.to_string());

                    if let Err(e) = execution::run_job_internal(
                        job,
                        InvocationTrigger::Scheduled,
                        &state.running_jobs,
                        Arc::clone(&job_service),
                        Arc::clone(&statistics_service),
                        app_handle.clone(),
                    ) {
                        log::error!(
                            "Scheduler: failed to execute job '{}' ({}): {}",
                            job.name,
                            job.id,
                            e
                        );
                    }
                }
            }
        }
    });
}
