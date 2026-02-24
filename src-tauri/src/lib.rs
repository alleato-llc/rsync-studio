use std::sync::Arc;
use std::time::Duration;

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager, WindowEvent};

use chrono::Utc;

use rsync_core::database::sqlite::Database;
use rsync_core::repository::sqlite::invocation::SqliteInvocationRepository;
use rsync_core::repository::sqlite::job::SqliteJobRepository;
use rsync_core::repository::sqlite::settings::SqliteSettingsRepository;
use rsync_core::repository::sqlite::snapshot::SqliteSnapshotRepository;
use rsync_core::repository::sqlite::statistics::SqliteStatisticsRepository;
use rsync_core::models::backup::InvocationTrigger;
use rsync_core::services::history_retention;
use rsync_core::services::job_service::JobService;
use rsync_core::services::scheduler;
use rsync_core::services::settings_service::SettingsService;
use rsync_core::services::statistics_service::StatisticsService;

mod commands;
mod execution;
mod state;

use state::AppState;

/// How often the scheduler checks for due jobs (in seconds).
const SCHEDULER_CHECK_INTERVAL_SECS: u64 = 300; // 5 minutes

/// How many scheduler cycles between retention checks (~1 hour at 5min intervals).
const RETENTION_CHECK_EVERY_N_CYCLES: u64 = 12;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .menu(|handle| {
            let about = MenuItem::with_id(handle, "about", "About Rsync Studio", true, None::<&str>)?;
            let app_submenu = Submenu::with_items(
                handle,
                "Rsync Studio",
                true,
                &[
                    &about,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::services(handle, None::<&str>)?,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::hide(handle, None::<&str>)?,
                    &PredefinedMenuItem::hide_others(handle, None::<&str>)?,
                    &PredefinedMenuItem::show_all(handle, None::<&str>)?,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::quit(handle, None::<&str>)?,
                ],
            )?;
            let edit_submenu = Submenu::with_items(
                handle,
                "Edit",
                true,
                &[
                    &PredefinedMenuItem::undo(handle, None::<&str>)?,
                    &PredefinedMenuItem::redo(handle, None::<&str>)?,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::cut(handle, None::<&str>)?,
                    &PredefinedMenuItem::copy(handle, None::<&str>)?,
                    &PredefinedMenuItem::paste(handle, None::<&str>)?,
                    &PredefinedMenuItem::select_all(handle, None::<&str>)?,
                ],
            )?;
            let window_submenu = Submenu::with_items(
                handle,
                "Window",
                true,
                &[
                    &PredefinedMenuItem::minimize(handle, None::<&str>)?,
                    &PredefinedMenuItem::maximize(handle, None::<&str>)?,
                    &PredefinedMenuItem::separator(handle)?,
                    &PredefinedMenuItem::close_window(handle, None::<&str>)?,
                ],
            )?;
            Menu::with_items(handle, &[&app_submenu, &edit_submenu, &window_submenu])
        })
        .on_menu_event(|app, event| {
            if event.id.as_ref() == "about" {
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.show();
                    let _ = win.set_focus();
                }
                let _ = app.emit("navigate-to-about", ());
            }
        })
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&data_dir)?;

            let log_dir = data_dir.join("logs");
            std::fs::create_dir_all(&log_dir)?;
            let default_log_dir = log_dir
                .to_str()
                .expect("invalid log dir path")
                .to_string();

            let db_path = data_dir.join("rsync-studio.db");
            let database = Database::open(db_path.to_str().expect("invalid db path"))
                .expect("failed to open database");

            let conn = database.conn();
            let jobs = Arc::new(SqliteJobRepository::new(conn.clone()));
            let invocations = Arc::new(SqliteInvocationRepository::new(conn.clone()));
            let snapshots = Arc::new(SqliteSnapshotRepository::new(conn.clone()));
            let statistics_repo = Arc::new(SqliteStatisticsRepository::new(conn.clone()));
            let settings_repo = Arc::new(SqliteSettingsRepository::new(conn));

            let job_service = Arc::new(JobService::new(jobs, invocations, snapshots));
            let statistics_service = Arc::new(StatisticsService::new(statistics_repo));
            let settings_service = Arc::new(SettingsService::new(settings_repo));

            app.manage(AppState {
                _database: database,
                job_service,
                statistics_service,
                settings_service,
                running_jobs: execution::RunningJobs::new(),
                default_log_dir,
            });

            // --- Run history retention on startup ---
            run_history_retention(&app.handle());

            // --- System tray ---
            setup_tray(app)?;

            // --- Close-to-tray behavior ---
            let app_handle = app.handle().clone();
            let main_window = app
                .get_webview_window("main")
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
            commands::get_setting,
            commands::set_setting,
            commands::get_log_directory,
            commands::set_log_directory,
            commands::get_retention_settings,
            commands::set_retention_settings,
            commands::get_auto_trailing_slash,
            commands::set_auto_trailing_slash,
            commands::delete_invocation,
            commands::delete_invocations_for_job,
            commands::count_invocations,
            commands::read_log_file,
            commands::read_log_file_lines,
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
        .tooltip("Rsync Studio")
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

fn run_history_retention(app_handle: &tauri::AppHandle) {
    let state: tauri::State<'_, AppState> = app_handle.state();
    let settings_service = Arc::clone(&state.settings_service);
    let job_service = Arc::clone(&state.job_service);

    let retention = match settings_service.get_retention_settings() {
        Ok(r) => r,
        Err(e) => {
            log::error!("Retention: failed to load settings: {}", e);
            return;
        }
    };

    let config = history_retention::HistoryRetentionConfig {
        max_age_days: retention.max_log_age_days,
        max_per_job: retention.max_history_per_job,
    };

    let all_invocations = match job_service.list_all_invocations() {
        Ok(inv) => inv,
        Err(e) => {
            log::error!("Retention: failed to list invocations: {}", e);
            return;
        }
    };

    let to_prune = history_retention::compute_invocations_to_prune(&all_invocations, &config);

    for (inv_id, log_path) in &to_prune {
        // Delete log file if it exists
        if let Some(path) = log_path {
            if std::path::Path::new(path).exists() {
                if let Err(e) = std::fs::remove_file(path) {
                    log::error!("Retention: failed to delete log file {}: {}", path, e);
                }
            }
        }
        // Delete invocation from DB
        if let Err(e) = job_service.delete_invocation(inv_id) {
            log::error!("Retention: failed to delete invocation {}: {}", inv_id, e);
        }
    }

    if !to_prune.is_empty() {
        log::info!("Retention: pruned {} invocations", to_prune.len());
    }
}

fn spawn_scheduler(app_handle: tauri::AppHandle) {
    std::thread::spawn(move || {
        let mut cycle_count: u64 = 0;
        loop {
            std::thread::sleep(Duration::from_secs(SCHEDULER_CHECK_INTERVAL_SECS));
            cycle_count += 1;

            let state: tauri::State<'_, AppState> = app_handle.state();
            let job_service = Arc::clone(&state.job_service);
            let statistics_service = Arc::clone(&state.statistics_service);

            // Periodically run history retention
            if cycle_count % RETENTION_CHECK_EVERY_N_CYCLES == 0 {
                run_history_retention(&app_handle);
            }

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
                    log::info!(
                        "Scheduler: job '{}' ({}) is due, executing",
                        job.name,
                        job.id
                    );
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
