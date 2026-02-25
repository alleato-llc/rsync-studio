pub mod test_file_system;
pub mod test_helpers;
pub mod test_rsync_client;

#[cfg(test)]
mod test_file_system_tests;
#[cfg(test)]
mod test_rsync_client_tests;
#[cfg(test)]
mod sqlite_job_repository_tests;
#[cfg(test)]
mod sqlite_invocation_repository_tests;
#[cfg(test)]
mod sqlite_snapshot_repository_tests;
#[cfg(test)]
mod sqlite_statistics_repository_tests;
#[cfg(test)]
mod sqlite_settings_repository_tests;
#[cfg(test)]
mod job_service_integration_tests;
#[cfg(test)]
mod statistics_service_tests;
#[cfg(test)]
mod log_scrubber_tests;
