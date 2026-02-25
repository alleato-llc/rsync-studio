use std::sync::Arc;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use uuid::Uuid;

use rsync_core::models::backup::{BackupInvocation, InvocationTrigger};
use rsync_core::models::job::JobDefinition;
use rsync_core::models::progress::{JobStatusEvent, LogLine, ProgressUpdate};
use rsync_core::models::statistics::AggregatedStats;
use rsync_core::services::command_explainer::{self, CommandExplanation};
use rsync_core::services::command_parser;
use rsync_core::services::job_executor::JobExecutor;
use rsync_core::services::job_service::JobService;
use rsync_core::services::settings_service::SettingsService;
use rsync_core::services::statistics_service::StatisticsService;

use crate::handler::{TuiEvent, TuiEventHandler};
use crate::theme::{self, Theme};
use crate::ui::text_input::TextInput;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Jobs,
    History,
    Statistics,
    Tools,
    Settings,
    About,
}

impl Page {
    pub const ALL: [Page; 6] = [
        Page::Jobs,
        Page::History,
        Page::Statistics,
        Page::Tools,
        Page::Settings,
        Page::About,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Page::Jobs => "Jobs",
            Page::History => "History",
            Page::Statistics => "Stats",
            Page::Tools => "Tools",
            Page::Settings => "Settings",
            Page::About => "About",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            Page::Jobs => 0,
            Page::History => 1,
            Page::Statistics => 2,
            Page::Tools => 3,
            Page::Settings => 4,
            Page::About => 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PopupKind {
    Help,
    Confirm {
        title: String,
        message: String,
        action: ConfirmAction,
    },
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    DeleteJob(Uuid),
    DeleteInvocation(Uuid),
    DeleteAllHistory(Uuid),
    ResetStatistics,
    ResetStatisticsForJob(Uuid),
}

/// Mode for the job form
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobFormMode {
    Create,
    Edit(Uuid),
}

/// State for the jobs page
#[derive(Debug)]
pub struct JobsState {
    pub jobs: Vec<JobDefinition>,
    pub selected: usize,
    pub search_active: bool,
    pub search_input: TextInput,
}

impl Default for JobsState {
    fn default() -> Self {
        Self {
            jobs: Vec::new(),
            selected: 0,
            search_active: false,
            search_input: TextInput::new(),
        }
    }
}

/// State for viewing live job output
#[derive(Debug)]
pub struct JobOutputState {
    pub job_id: Uuid,
    pub job_name: String,
    pub invocation_id: Option<Uuid>,
    pub log_lines: Vec<LogLine>,
    pub progress: Option<ProgressUpdate>,
    pub status: Option<JobStatusEvent>,
    pub scroll_offset: usize,
    pub follow: bool,
}

/// State for the job form
pub struct JobFormState {
    pub mode: JobFormMode,
    pub job: JobDefinition,
    pub field_index: usize,
    pub editing: bool,
    pub field_inputs: Vec<TextInput>,
}

/// State for the history page
#[derive(Debug)]
pub struct HistoryState {
    pub invocations: Vec<BackupInvocation>,
    pub selected: usize,
    pub viewing_log: bool,
    pub log_lines: Vec<String>,
    pub log_scroll: usize,
    pub log_follow: bool,
    pub log_search_active: bool,
    pub log_search_input: TextInput,
}

impl Default for HistoryState {
    fn default() -> Self {
        Self {
            invocations: Vec::new(),
            selected: 0,
            viewing_log: false,
            log_lines: Vec::new(),
            log_scroll: 0,
            log_follow: false,
            log_search_active: false,
            log_search_input: TextInput::new(),
        }
    }
}

/// State for the statistics page
#[derive(Debug, Default)]
pub struct StatisticsState {
    pub aggregated: Option<AggregatedStats>,
    pub per_job: Vec<(String, AggregatedStats)>,
    pub selected: usize,
}

/// State for the tools page
pub struct ToolsState {
    pub active_tab: usize, // 0 = explainer, 1 = scrubber
    pub command_input: TextInput,
    pub explanation: Option<CommandExplanation>,
    pub explanation_error: Option<String>,
    pub scrub_input: TextInput,
}

impl Default for ToolsState {
    fn default() -> Self {
        Self {
            active_tab: 0,
            command_input: TextInput::new(),
            explanation: None,
            explanation_error: None,
            scrub_input: TextInput::new(),
        }
    }
}

/// State for the settings page
#[derive(Debug)]
pub struct SettingsState {
    pub selected: usize,
    pub editing: bool,
    pub edit_input: TextInput,
    pub log_directory: String,
    pub max_log_age_days: u32,
    pub max_history_per_job: usize,
    pub auto_trailing_slash: bool,
    pub tui_theme: String,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            selected: 0,
            editing: false,
            edit_input: TextInput::new(),
            log_directory: String::new(),
            max_log_age_days: 90,
            max_history_per_job: 15,
            auto_trailing_slash: true,
            tui_theme: "Default".to_string(),
        }
    }
}

pub struct App {
    pub current_page: Page,
    pub should_quit: bool,
    pub job_executor: Arc<JobExecutor>,
    pub job_service: Arc<JobService>,
    pub statistics_service: Arc<StatisticsService>,
    pub settings_service: Arc<SettingsService>,
    pub theme: &'static Theme,

    // Event channel for job execution events
    pub job_sender: std::sync::mpsc::Sender<TuiEvent>,

    // Per-page state
    pub jobs_state: JobsState,
    pub job_output: Option<JobOutputState>,
    pub job_form: Option<JobFormState>,
    pub history_state: HistoryState,
    pub statistics_state: StatisticsState,
    pub tools_state: ToolsState,
    pub settings_state: SettingsState,

    // Popup
    pub popup: Option<PopupKind>,
}

impl App {
    pub fn new(
        job_executor: Arc<JobExecutor>,
        job_service: Arc<JobService>,
        statistics_service: Arc<StatisticsService>,
        settings_service: Arc<SettingsService>,
        job_sender: std::sync::mpsc::Sender<TuiEvent>,
    ) -> Self {
        // Load theme from settings
        let theme_name = settings_service
            .get_setting("tui_theme")
            .ok()
            .flatten()
            .unwrap_or_else(|| "Default".to_string());
        let theme = theme::get_theme(&theme_name);

        let mut app = Self {
            current_page: Page::Jobs,
            should_quit: false,
            job_executor,
            job_service,
            statistics_service,
            settings_service,
            theme,
            job_sender,
            jobs_state: JobsState::default(),
            job_output: None,
            job_form: None,
            history_state: HistoryState::default(),
            statistics_state: StatisticsState::default(),
            tools_state: ToolsState::default(),
            settings_state: SettingsState::default(),
            popup: None,
        };

        app.refresh_current_page();
        app
    }

    /// Refresh data for the current page.
    pub fn refresh_current_page(&mut self) {
        match self.current_page {
            Page::Jobs => self.refresh_jobs(),
            Page::History => self.refresh_history(),
            Page::Statistics => self.refresh_statistics(),
            Page::Settings => self.refresh_settings(),
            _ => {}
        }
    }

    pub fn refresh_jobs(&mut self) {
        if let Ok(jobs) = self.job_service.list_jobs() {
            self.jobs_state.jobs = jobs;
            if self.jobs_state.selected >= self.jobs_state.jobs.len() && !self.jobs_state.jobs.is_empty() {
                self.jobs_state.selected = self.jobs_state.jobs.len() - 1;
            }
        }
    }

    pub fn refresh_history(&mut self) {
        if let Ok(invocations) = self.job_service.list_all_invocations() {
            let mut invocations = invocations;
            invocations.sort_by(|a, b| b.started_at.cmp(&a.started_at));
            self.history_state.invocations = invocations;
            if self.history_state.selected >= self.history_state.invocations.len()
                && !self.history_state.invocations.is_empty()
            {
                self.history_state.selected = self.history_state.invocations.len() - 1;
            }
        }
    }

    pub fn refresh_statistics(&mut self) {
        if let Ok(agg) = self.statistics_service.get_aggregated() {
            self.statistics_state.aggregated = Some(agg);
        }
        // Per-job stats
        if let Ok(jobs) = self.job_service.list_jobs() {
            let mut per_job = Vec::new();
            for job in &jobs {
                if let Ok(stats) = self.statistics_service.get_aggregated_for_job(&job.id) {
                    if stats.total_jobs_run > 0 {
                        per_job.push((job.name.clone(), stats));
                    }
                }
            }
            self.statistics_state.per_job = per_job;
        }
    }

    pub fn refresh_settings(&mut self) {
        let ss = &self.settings_service;
        self.settings_state.log_directory = ss
            .get_log_directory()
            .ok()
            .flatten()
            .unwrap_or_else(|| self.job_executor.default_log_dir().to_string());

        if let Ok(ret) = ss.get_retention_settings() {
            self.settings_state.max_log_age_days = ret.max_log_age_days;
            self.settings_state.max_history_per_job = ret.max_history_per_job;
        }
        self.settings_state.auto_trailing_slash = ss.get_auto_trailing_slash().unwrap_or(true);
        self.settings_state.tui_theme = ss
            .get_setting("tui_theme")
            .ok()
            .flatten()
            .unwrap_or_else(|| "Default".to_string());
    }

    /// Handle a job execution event from the background thread.
    pub fn handle_job_event(&mut self, event: TuiEvent) {
        match event {
            TuiEvent::LogLine(log_line) => {
                if let Some(ref mut output) = self.job_output {
                    if Some(log_line.invocation_id) == output.invocation_id {
                        output.log_lines.push(log_line);
                        if output.follow {
                            output.scroll_offset = output.log_lines.len().saturating_sub(1);
                        }
                    }
                }
            }
            TuiEvent::Progress(progress) => {
                if let Some(ref mut output) = self.job_output {
                    if Some(progress.invocation_id) == output.invocation_id {
                        output.progress = Some(progress);
                    }
                }
            }
            TuiEvent::StatusChange(status) => {
                if let Some(ref mut output) = self.job_output {
                    if Some(status.invocation_id) == output.invocation_id {
                        output.status = Some(status);
                    }
                }
                // Refresh jobs list to update status
                self.refresh_jobs();
            }
        }
    }

    /// Handle a key event.
    pub fn handle_key(&mut self, key: KeyEvent) {
        // Quit shortcuts
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.should_quit = true;
            return;
        }

        // Popup handling takes priority
        if let Some(popup) = &self.popup {
            match popup {
                PopupKind::Help | PopupKind::Error(_) => {
                    match key.code {
                        KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') | KeyCode::Char('?') => {
                            self.popup = None;
                        }
                        _ => {}
                    }
                    return;
                }
                PopupKind::Confirm { action, .. } => {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Enter => {
                            let action = action.clone();
                            self.popup = None;
                            self.execute_confirm_action(action);
                        }
                        KeyCode::Char('n') | KeyCode::Esc => {
                            self.popup = None;
                        }
                        _ => {}
                    }
                    return;
                }
            }
        }

        // Job output viewer takes full control
        if self.job_output.is_some() {
            self.handle_job_output_key(key);
            return;
        }

        // Job form takes full control
        if self.job_form.is_some() {
            self.handle_job_form_key(key);
            return;
        }

        // Log viewer in history
        if self.history_state.viewing_log {
            self.handle_log_viewer_key(key);
            return;
        }

        // Search mode in jobs
        if self.jobs_state.search_active {
            self.handle_jobs_search_key(key);
            return;
        }

        // Text input mode in tools
        if self.current_page == Page::Tools
            && (self.tools_state.command_input.is_focused || self.tools_state.scrub_input.is_focused)
        {
            self.handle_tools_input_key(key);
            return;
        }

        // Settings editing mode
        if self.settings_state.editing {
            self.handle_settings_edit_key(key);
            return;
        }

        // Global keys
        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
                return;
            }
            KeyCode::Char('?') => {
                self.popup = Some(PopupKind::Help);
                return;
            }
            KeyCode::Char('1') => { self.switch_page(Page::Jobs); return; }
            KeyCode::Char('2') => { self.switch_page(Page::History); return; }
            KeyCode::Char('3') => { self.switch_page(Page::Statistics); return; }
            KeyCode::Char('4') => { self.switch_page(Page::Tools); return; }
            KeyCode::Char('5') => { self.switch_page(Page::Settings); return; }
            KeyCode::Char('6') => { self.switch_page(Page::About); return; }
            KeyCode::Tab => {
                let idx = self.current_page.index();
                let next = (idx + 1) % Page::ALL.len();
                self.switch_page(Page::ALL[next]);
                return;
            }
            KeyCode::BackTab => {
                let idx = self.current_page.index();
                let prev = if idx == 0 { Page::ALL.len() - 1 } else { idx - 1 };
                self.switch_page(Page::ALL[prev]);
                return;
            }
            _ => {}
        }

        // Page-specific keys
        match self.current_page {
            Page::Jobs => self.handle_jobs_key(key),
            Page::History => self.handle_history_key(key),
            Page::Statistics => self.handle_statistics_key(key),
            Page::Tools => self.handle_tools_key(key),
            Page::Settings => self.handle_settings_key(key),
            Page::About => {} // No special keys
        }
    }

    fn switch_page(&mut self, page: Page) {
        self.current_page = page;
        self.refresh_current_page();
    }

    fn execute_confirm_action(&mut self, action: ConfirmAction) {
        match action {
            ConfirmAction::DeleteJob(id) => {
                if let Err(e) = self.job_service.delete_job(&id) {
                    self.popup = Some(PopupKind::Error(format!("Failed to delete job: {}", e)));
                } else {
                    self.refresh_jobs();
                }
            }
            ConfirmAction::DeleteInvocation(id) => {
                if let Err(e) = self.job_service.delete_invocation(&id) {
                    self.popup = Some(PopupKind::Error(format!("Failed to delete invocation: {}", e)));
                } else {
                    self.refresh_history();
                }
            }
            ConfirmAction::DeleteAllHistory(job_id) => {
                if let Err(e) = self.job_service.delete_invocations_for_job(&job_id) {
                    self.popup = Some(PopupKind::Error(format!("Failed to delete history: {}", e)));
                } else {
                    self.refresh_history();
                }
            }
            ConfirmAction::ResetStatistics => {
                if let Err(e) = self.statistics_service.reset() {
                    self.popup = Some(PopupKind::Error(format!("Failed to reset statistics: {}", e)));
                } else {
                    self.refresh_statistics();
                }
            }
            ConfirmAction::ResetStatisticsForJob(id) => {
                if let Err(e) = self.statistics_service.reset_for_job(&id) {
                    self.popup = Some(PopupKind::Error(format!("Failed to reset job statistics: {}", e)));
                } else {
                    self.refresh_statistics();
                }
            }
        }
    }

    // --- Jobs page keys ---

    fn handle_jobs_key(&mut self, key: KeyEvent) {
        let len = self.filtered_jobs().len();
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if len > 0 {
                    self.jobs_state.selected = (self.jobs_state.selected + 1).min(len - 1);
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.jobs_state.selected = self.jobs_state.selected.saturating_sub(1);
            }
            KeyCode::Char('n') => {
                self.open_job_form_create();
            }
            KeyCode::Enter => {
                if let Some(job) = self.selected_job() {
                    let job_id = job.id;
                    self.open_job_form_edit(job_id);
                }
            }
            KeyCode::Char('r') => {
                if let Some(job) = self.selected_job() {
                    self.run_job(&job.clone(), false);
                }
            }
            KeyCode::Char('d') => {
                if let Some(job) = self.selected_job() {
                    self.run_job(&job.clone(), true);
                }
            }
            KeyCode::Char('c') => {
                if let Some(job) = self.selected_job() {
                    let job_id = job.id;
                    self.job_executor.cancel(&job_id);
                }
            }
            KeyCode::Char('x') => {
                if let Some(job) = self.selected_job() {
                    self.popup = Some(PopupKind::Confirm {
                        title: "Delete Job".to_string(),
                        message: format!("Delete '{}'? This cannot be undone.", job.name),
                        action: ConfirmAction::DeleteJob(job.id),
                    });
                }
            }
            KeyCode::Char('o') => {
                if let Some(job) = self.selected_job() {
                    self.open_job_output(job.id, job.name.clone());
                }
            }
            KeyCode::Char('/') => {
                self.jobs_state.search_active = true;
                self.jobs_state.search_input.clear();
                self.jobs_state.search_input.is_focused = true;
            }
            _ => {}
        }
    }

    fn handle_jobs_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.jobs_state.search_active = false;
                self.jobs_state.search_input.is_focused = false;
                self.jobs_state.search_input.clear();
            }
            KeyCode::Enter => {
                self.jobs_state.search_input.is_focused = false;
            }
            _ => {
                self.jobs_state.search_input.handle_key(key);
            }
        }
    }

    pub fn filtered_jobs(&self) -> Vec<&JobDefinition> {
        let query = self.jobs_state.search_input.value().to_lowercase();
        if query.is_empty() {
            self.jobs_state.jobs.iter().collect()
        } else {
            self.jobs_state
                .jobs
                .iter()
                .filter(|j| j.name.to_lowercase().contains(&query))
                .collect()
        }
    }

    fn selected_job(&self) -> Option<JobDefinition> {
        let jobs = self.filtered_jobs();
        jobs.get(self.jobs_state.selected).map(|j| (*j).clone())
    }

    fn run_job(&mut self, job: &JobDefinition, dry_run: bool) {
        let mut job = job.clone();
        if dry_run {
            job.options.dry_run = true;
        }

        let handler = Arc::new(TuiEventHandler::new(self.job_sender.clone()));
        match self.job_executor.execute(&job, InvocationTrigger::Manual, handler) {
            Ok(invocation_id) => {
                self.open_job_output(job.id, job.name.clone());
                if let Some(ref mut output) = self.job_output {
                    output.invocation_id = Some(invocation_id);
                }
            }
            Err(e) => {
                self.popup = Some(PopupKind::Error(format!("Failed to execute job: {}", e)));
            }
        }
    }

    fn open_job_output(&mut self, job_id: Uuid, job_name: String) {
        self.job_output = Some(JobOutputState {
            job_id,
            job_name,
            invocation_id: None,
            log_lines: Vec::new(),
            progress: None,
            status: None,
            scroll_offset: 0,
            follow: true,
        });
    }

    fn open_job_form_create(&mut self) {
        let now = chrono::Utc::now();
        let job = JobDefinition {
            id: uuid::Uuid::new_v4(),
            name: String::new(),
            description: None,
            source: rsync_core::models::job::StorageLocation::Local {
                path: String::new(),
            },
            destination: rsync_core::models::job::StorageLocation::Local {
                path: String::new(),
            },
            backup_mode: rsync_core::models::job::BackupMode::Mirror,
            options: rsync_core::models::job::RsyncOptions::default(),
            ssh_config: None,
            schedule: None,
            enabled: true,
            created_at: now,
            updated_at: now,
        };

        self.job_form = Some(JobFormState {
            mode: JobFormMode::Create,
            job,
            field_index: 0,
            editing: false,
            field_inputs: (0..20).map(|_| TextInput::new()).collect(),
        });
    }

    fn open_job_form_edit(&mut self, job_id: Uuid) {
        if let Ok(job) = self.job_service.get_job(&job_id) {
            let mut inputs: Vec<TextInput> = (0..20).map(|_| TextInput::new()).collect();
            // Pre-fill inputs from job
            inputs[0].set_value(&job.name);
            inputs[1].set_value(job.description.as_deref().unwrap_or(""));
            match &job.source {
                rsync_core::models::job::StorageLocation::Local { path } => {
                    inputs[2].set_value(path);
                }
                _ => {}
            }
            match &job.destination {
                rsync_core::models::job::StorageLocation::Local { path } => {
                    inputs[3].set_value(path);
                }
                _ => {}
            }

            self.job_form = Some(JobFormState {
                mode: JobFormMode::Edit(job_id),
                job,
                field_index: 0,
                editing: false,
                field_inputs: inputs,
            });
        }
    }

    // --- Job output viewer keys ---

    fn handle_job_output_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.job_output = None;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if let Some(ref mut output) = self.job_output {
                    output.follow = false;
                    output.scroll_offset = (output.scroll_offset + 1)
                        .min(output.log_lines.len().saturating_sub(1));
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if let Some(ref mut output) = self.job_output {
                    output.follow = false;
                    output.scroll_offset = output.scroll_offset.saturating_sub(1);
                }
            }
            KeyCode::Char('g') => {
                if let Some(ref mut output) = self.job_output {
                    output.follow = false;
                    output.scroll_offset = 0;
                }
            }
            KeyCode::Char('G') => {
                if let Some(ref mut output) = self.job_output {
                    output.follow = true;
                    output.scroll_offset = output.log_lines.len().saturating_sub(1);
                }
            }
            KeyCode::Char('f') => {
                if let Some(ref mut output) = self.job_output {
                    output.follow = !output.follow;
                    if output.follow {
                        output.scroll_offset = output.log_lines.len().saturating_sub(1);
                    }
                }
            }
            KeyCode::Char('c') => {
                if let Some(ref output) = self.job_output {
                    self.job_executor.cancel(&output.job_id);
                }
            }
            KeyCode::PageDown => {
                if let Some(ref mut output) = self.job_output {
                    output.follow = false;
                    output.scroll_offset = (output.scroll_offset + 20)
                        .min(output.log_lines.len().saturating_sub(1));
                }
            }
            KeyCode::PageUp => {
                if let Some(ref mut output) = self.job_output {
                    output.follow = false;
                    output.scroll_offset = output.scroll_offset.saturating_sub(20);
                }
            }
            _ => {}
        }
    }

    // --- Job form keys ---

    fn handle_job_form_key(&mut self, key: KeyEvent) {
        let form = match self.job_form.as_mut() {
            Some(f) => f,
            None => return,
        };

        if form.editing {
            match key.code {
                KeyCode::Esc => {
                    form.editing = false;
                    form.field_inputs[form.field_index].is_focused = false;
                }
                KeyCode::Enter => {
                    form.editing = false;
                    form.field_inputs[form.field_index].is_focused = false;
                    // Apply the field value to the job
                    apply_form_field(form);
                }
                _ => {
                    form.field_inputs[form.field_index].handle_key(key);
                }
            }
            return;
        }

        match key.code {
            KeyCode::Esc => {
                self.job_form = None;
            }
            KeyCode::Tab | KeyCode::Char('j') | KeyCode::Down => {
                form.field_index = (form.field_index + 1).min(form.field_inputs.len() - 1);
            }
            KeyCode::BackTab | KeyCode::Char('k') | KeyCode::Up => {
                form.field_index = form.field_index.saturating_sub(1);
            }
            KeyCode::Enter => {
                // Check if this is the "Save" pseudo-field (last field)
                if form.field_index >= 10 {
                    // Save the job
                    self.save_job_form();
                } else {
                    form.editing = true;
                    form.field_inputs[form.field_index].is_focused = true;
                }
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.save_job_form();
            }
            _ => {}
        }
    }

    fn save_job_form(&mut self) {
        let form = match self.job_form.take() {
            Some(f) => f,
            None => return,
        };

        let result = match form.mode {
            JobFormMode::Create => self.job_service.create_job(form.job),
            JobFormMode::Edit(_) => self.job_service.update_job(form.job),
        };

        match result {
            Ok(_) => {
                self.refresh_jobs();
            }
            Err(e) => {
                self.popup = Some(PopupKind::Error(format!("Failed to save job: {}", e)));
            }
        }
    }

    // --- History page keys ---

    fn handle_history_key(&mut self, key: KeyEvent) {
        let len = self.history_state.invocations.len();
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                if len > 0 {
                    self.history_state.selected = (self.history_state.selected + 1).min(len - 1);
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.history_state.selected = self.history_state.selected.saturating_sub(1);
            }
            KeyCode::Enter => {
                if let Some(inv) = self.history_state.invocations.get(self.history_state.selected) {
                    if let Some(ref path) = inv.log_file_path {
                        if let Ok(content) = std::fs::read_to_string(path) {
                            self.history_state.log_lines =
                                content.lines().map(String::from).collect();
                            self.history_state.log_scroll = 0;
                            self.history_state.viewing_log = true;
                        }
                    }
                }
            }
            KeyCode::Char('d') => {
                if let Some(inv) = self.history_state.invocations.get(self.history_state.selected) {
                    self.popup = Some(PopupKind::Confirm {
                        title: "Delete Invocation".to_string(),
                        message: "Delete this invocation record?".to_string(),
                        action: ConfirmAction::DeleteInvocation(inv.id),
                    });
                }
            }
            _ => {}
        }
    }

    fn handle_log_viewer_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.history_state.viewing_log = false;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.history_state.log_scroll = (self.history_state.log_scroll + 1)
                    .min(self.history_state.log_lines.len().saturating_sub(1));
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.history_state.log_scroll = self.history_state.log_scroll.saturating_sub(1);
            }
            KeyCode::Char('g') => {
                self.history_state.log_scroll = 0;
            }
            KeyCode::Char('G') => {
                self.history_state.log_scroll =
                    self.history_state.log_lines.len().saturating_sub(1);
            }
            KeyCode::PageDown => {
                self.history_state.log_scroll = (self.history_state.log_scroll + 20)
                    .min(self.history_state.log_lines.len().saturating_sub(1));
            }
            KeyCode::PageUp => {
                self.history_state.log_scroll = self.history_state.log_scroll.saturating_sub(20);
            }
            _ => {}
        }
    }

    // --- Statistics page keys ---

    fn handle_statistics_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('r') => {
                self.popup = Some(PopupKind::Confirm {
                    title: "Reset Statistics".to_string(),
                    message: "Reset all statistics? This cannot be undone.".to_string(),
                    action: ConfirmAction::ResetStatistics,
                });
            }
            KeyCode::Char('e') => {
                if let Ok(json) = self.statistics_service.export() {
                    let path = format!(
                        "{}/statistics-export.json",
                        self.job_executor.default_log_dir()
                    );
                    if std::fs::write(&path, &json).is_ok() {
                        self.popup = Some(PopupKind::Error(format!("Exported to {}", path)));
                    }
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let len = self.statistics_state.per_job.len();
                if len > 0 {
                    self.statistics_state.selected =
                        (self.statistics_state.selected + 1).min(len - 1);
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.statistics_state.selected = self.statistics_state.selected.saturating_sub(1);
            }
            _ => {}
        }
    }

    // --- Tools page keys ---

    fn handle_tools_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Tab => {
                self.tools_state.active_tab = (self.tools_state.active_tab + 1) % 2;
            }
            KeyCode::Enter | KeyCode::Char('i') => {
                if self.tools_state.active_tab == 0 {
                    self.tools_state.command_input.is_focused = true;
                } else {
                    self.tools_state.scrub_input.is_focused = true;
                }
            }
            _ => {}
        }
    }

    fn handle_tools_input_key(&mut self, key: KeyEvent) {
        if self.tools_state.command_input.is_focused {
            match key.code {
                KeyCode::Esc => {
                    self.tools_state.command_input.is_focused = false;
                }
                KeyCode::Enter => {
                    let cmd = self.tools_state.command_input.value().to_string();
                    self.tools_state.command_input.is_focused = false;
                    match command_parser::parse_rsync_command(&cmd) {
                        Ok(parsed) => {
                            self.tools_state.explanation = Some(command_explainer::explain_command(&parsed));
                            self.tools_state.explanation_error = None;
                        }
                        Err(e) => {
                            self.tools_state.explanation = None;
                            self.tools_state.explanation_error = Some(e);
                        }
                    }
                }
                _ => {
                    self.tools_state.command_input.handle_key(key);
                }
            }
        } else if self.tools_state.scrub_input.is_focused {
            match key.code {
                KeyCode::Esc => {
                    self.tools_state.scrub_input.is_focused = false;
                }
                _ => {
                    self.tools_state.scrub_input.handle_key(key);
                }
            }
        }
    }

    // --- Settings page keys ---

    fn handle_settings_key(&mut self, key: KeyEvent) {
        let settings_count = 5; // log_dir, max_age, max_per_job, auto_slash, theme
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.settings_state.selected =
                    (self.settings_state.selected + 1).min(settings_count - 1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.settings_state.selected = self.settings_state.selected.saturating_sub(1);
            }
            KeyCode::Enter => {
                self.settings_state.editing = true;
                // Pre-fill the input with current value
                let val = match self.settings_state.selected {
                    0 => self.settings_state.log_directory.clone(),
                    1 => self.settings_state.max_log_age_days.to_string(),
                    2 => self.settings_state.max_history_per_job.to_string(),
                    3 => {
                        // Toggle boolean
                        let new_val = !self.settings_state.auto_trailing_slash;
                        let _ = self.settings_service.set_auto_trailing_slash(new_val);
                        self.settings_state.auto_trailing_slash = new_val;
                        self.settings_state.editing = false;
                        return;
                    }
                    4 => {
                        // Cycle theme
                        let names = theme::theme_names();
                        let idx = names
                            .iter()
                            .position(|n| n.eq_ignore_ascii_case(&self.settings_state.tui_theme))
                            .unwrap_or(0);
                        let next = (idx + 1) % names.len();
                        let new_theme = names[next].to_string();
                        let _ = self.settings_service.set_setting("tui_theme", &new_theme);
                        self.settings_state.tui_theme = new_theme.clone();
                        self.theme = theme::get_theme(&new_theme);
                        self.settings_state.editing = false;
                        return;
                    }
                    _ => String::new(),
                };
                self.settings_state.edit_input.set_value(&val);
                self.settings_state.edit_input.is_focused = true;
            }
            KeyCode::Char('e') => {
                // Export jobs
                if let Ok(jobs) = self.job_service.list_jobs() {
                    if let Ok(json) = rsync_core::services::export_import::export_jobs(jobs) {
                        let path = format!(
                            "{}/jobs-export.json",
                            self.job_executor.default_log_dir()
                        );
                        if std::fs::write(&path, &json).is_ok() {
                            self.popup = Some(PopupKind::Error(format!("Exported to {}", path)));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_settings_edit_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.settings_state.editing = false;
                self.settings_state.edit_input.is_focused = false;
            }
            KeyCode::Enter => {
                let val = self.settings_state.edit_input.value().to_string();
                self.settings_state.editing = false;
                self.settings_state.edit_input.is_focused = false;

                match self.settings_state.selected {
                    0 => {
                        let _ = self.settings_service.set_log_directory(&val);
                        self.settings_state.log_directory = val;
                    }
                    1 => {
                        if let Ok(days) = val.parse::<u32>() {
                            let _ = self.settings_service.set_retention_settings(
                                &rsync_core::services::settings_service::RetentionSettings {
                                    max_log_age_days: days,
                                    max_history_per_job: self.settings_state.max_history_per_job,
                                },
                            );
                            self.settings_state.max_log_age_days = days;
                        }
                    }
                    2 => {
                        if let Ok(max) = val.parse::<usize>() {
                            let _ = self.settings_service.set_retention_settings(
                                &rsync_core::services::settings_service::RetentionSettings {
                                    max_log_age_days: self.settings_state.max_log_age_days,
                                    max_history_per_job: max,
                                },
                            );
                            self.settings_state.max_history_per_job = max;
                        }
                    }
                    _ => {}
                }
            }
            _ => {
                self.settings_state.edit_input.handle_key(key);
            }
        }
    }

    /// Get the count of currently running jobs.
    pub fn running_count(&self) -> usize {
        self.job_executor.running_job_ids().len()
    }
}

fn apply_form_field(form: &mut JobFormState) {
    let val = form.field_inputs[form.field_index].value().to_string();
    match form.field_index {
        0 => form.job.name = val,
        1 => form.job.description = if val.is_empty() { None } else { Some(val) },
        2 => {
            form.job.source = rsync_core::models::job::StorageLocation::Local { path: val };
        }
        3 => {
            form.job.destination = rsync_core::models::job::StorageLocation::Local { path: val };
        }
        // Options toggles would go here, handled differently
        _ => {}
    }
}
