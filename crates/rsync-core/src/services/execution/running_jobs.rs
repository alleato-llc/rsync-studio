use std::collections::HashMap;
use std::process::Child;
use std::sync::{Arc, Mutex};

use uuid::Uuid;

pub struct RunningJobs {
    children: Mutex<HashMap<Uuid, Arc<Mutex<Child>>>>,
}

impl RunningJobs {
    pub fn new() -> Self {
        Self {
            children: Mutex::new(HashMap::new()),
        }
    }

    pub fn insert(&self, job_id: Uuid, child: Child) -> Arc<Mutex<Child>> {
        let arc = Arc::new(Mutex::new(child));
        self.children
            .lock()
            .expect("lock poisoned")
            .insert(job_id, arc.clone());
        arc
    }

    pub fn is_running(&self, job_id: &Uuid) -> bool {
        self.children
            .lock()
            .expect("lock poisoned")
            .contains_key(job_id)
    }

    pub fn cancel(&self, job_id: &Uuid) -> bool {
        if let Some(child_arc) = self
            .children
            .lock()
            .expect("lock poisoned")
            .get(job_id)
            .cloned()
        {
            if let Ok(mut child) = child_arc.lock() {
                let _ = child.kill();
            }
            true
        } else {
            false
        }
    }

    pub fn remove(&self, job_id: &Uuid) -> Option<Arc<Mutex<Child>>> {
        self.children
            .lock()
            .expect("lock poisoned")
            .remove(job_id)
    }

    pub fn running_job_ids(&self) -> Vec<Uuid> {
        self.children
            .lock()
            .expect("lock poisoned")
            .keys()
            .copied()
            .collect()
    }
}

impl Default for RunningJobs {
    fn default() -> Self {
        Self::new()
    }
}
