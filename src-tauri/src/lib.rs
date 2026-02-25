use std::sync::Arc;

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager, WindowEvent};

use rsync_core::database::sqlite::Database;
use rsync_core::repository::sqlite::invocation::SqliteInvocationRepository;
use rsync_core::repository::sqlite::job::SqliteJobRepository;
use rsync_core::repository::sqlite::settings::SqliteSettingsRepository;
use rsync_core::repository::sqlite::snapshot::SqliteSnapshotRepository;
use rsync_core::repository::sqlite::statistics::SqliteStatisticsRepository;
use rsync_core::services::job_executor::JobExecutor;
use rsync_core::services::job_service::JobService;
use rsync_core::services::retention_runner;
use rsync_core::services::running_jobs::RunningJobs;
use rsync_core::services::scheduler_backend::{InProcessScheduler, SchedulerBackend, SchedulerConfig};
use rsync_core::services::settings_service::SettingsService;
use rsync_core::services::statistics_service::StatisticsService;

mod commands;
mod execution;
mod state;

use execution::TauriEventHandler;
use state::AppState;

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
            let running_jobs = Arc::new(RunningJobs::new());

            let job_executor = Arc::new(JobExecutor::new(
                Arc::clone(&job_service),
                Arc::clone(&statistics_service),
                Arc::clone(&settings_service),
                Arc::clone(&running_jobs),
                default_log_dir,
            ));

            app.manage(AppState {
                _database: database,
                job_service: Arc::clone(&job_service),
                statistics_service: Arc::clone(&statistics_service),
                settings_service: Arc::clone(&settings_service),
                job_executor: Arc::clone(&job_executor),
            });

            // --- Run history retention on startup ---
            retention_runner::run_history_retention(&job_service, &settings_service);

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
            let scheduler_app_handle = app.handle().clone();
            let handler_factory: Arc<dyn Fn() -> Arc<dyn rsync_core::services::execution_handler::ExecutionEventHandler> + Send + Sync> =
                Arc::new(move || {
                    Arc::new(TauriEventHandler::new(scheduler_app_handle.clone()))
                });

            let scheduler_app_handle2 = app.handle().clone();
            let on_job_scheduled: Arc<dyn Fn(&uuid::Uuid) + Send + Sync> =
                Arc::new(move |job_id| {
                    let _ = scheduler_app_handle2.emit("job-scheduled", &job_id.to_string());
                });

            let in_process_scheduler = InProcessScheduler::new(
                SchedulerConfig::default(),
                Arc::clone(&job_executor),
                Arc::clone(&job_service),
                Arc::clone(&settings_service),
                handler_factory,
            )
            .with_on_job_scheduled(on_job_scheduled);

            // Start the scheduler — handle is intentionally leaked to keep the thread alive
            let _scheduler_handle = in_process_scheduler.start();
            std::mem::forget(_scheduler_handle);

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
            commands::get_dry_mode_settings,
            commands::set_dry_mode_settings,
            commands::delete_invocation,
            commands::delete_invocations_for_job,
            commands::count_invocations,
            commands::read_log_file,
            commands::read_log_file_lines,
            commands::scrub_scan_logs,
            commands::scrub_apply_logs,
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
            if let tauri::tray::TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(win) = app.get_webview_window("main") {
                    let visible = win.is_visible().unwrap_or(false);
                    let minimized = win.is_minimized().unwrap_or(false);
                    if visible && !minimized {
                        let _ = win.hide();
                    } else {
                        let _ = win.show();
                        let _ = win.unminimize();
                        let _ = win.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}
