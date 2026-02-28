use crate::models::execution::itemize::ItemizedChange;
use crate::models::execution::progress::ProgressUpdate;

#[derive(Debug, Clone)]
pub enum ExecutionEvent {
    StdoutLine(String),
    StderrLine(String),
    Progress(ProgressUpdate),
    ItemizedChange(ItemizedChange),
    Finished { exit_code: Option<i32> },
}
