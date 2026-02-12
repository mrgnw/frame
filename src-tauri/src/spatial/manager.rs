use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

use crate::spatial::error::SpatialError;
use crate::spatial::types::{SpatialErrorPayload, SpatialLogPayload, SpatialTask};
use crate::spatial::worker::run_spatial_worker;

pub enum SpatialMessage {
    Enqueue(SpatialTask),
    TaskStarted(String, u32),
    TaskCompleted(String),
    TaskError(String, SpatialError),
}

pub struct SpatialManager {
    pub(crate) sender: mpsc::Sender<SpatialMessage>,
    active_tasks: Arc<Mutex<HashMap<String, u32>>>,
    cancelled_tasks: Arc<Mutex<HashSet<String>>>,
}

impl SpatialManager {
    pub fn new(app: AppHandle) -> Self {
        let (tx, mut rx) = mpsc::channel(32);
        let tx_clone = tx.clone();
        let active_tasks = Arc::new(Mutex::new(HashMap::new()));
        let active_tasks_loop = Arc::clone(&active_tasks);
        let cancelled_tasks = Arc::new(Mutex::new(HashSet::new()));
        let cancelled_tasks_loop = Arc::clone(&cancelled_tasks);

        tauri::async_runtime::spawn(async move {
            let mut queue: VecDeque<SpatialTask> = VecDeque::new();
            let mut queued_ids: HashSet<String> = HashSet::new();
            // Only run one spatial task at a time (GPU-bound)
            let mut running: Option<String> = None;

            while let Some(msg) = rx.recv().await {
                match msg {
                    SpatialMessage::Enqueue(task) => {
                        {
                            let mut cancelled = cancelled_tasks_loop.lock().unwrap();
                            cancelled.remove(&task.id);
                        }

                        if running.as_ref() == Some(&task.id) || queued_ids.contains(&task.id) {
                            continue;
                        }

                        queued_ids.insert(task.id.clone());
                        queue.push_back(task);
                        Self::process_queue(
                            &app,
                            &tx_clone,
                            &mut queue,
                            &mut queued_ids,
                            &mut running,
                            Arc::clone(&cancelled_tasks_loop),
                        )
                        .await;
                    }
                    SpatialMessage::TaskStarted(id, pid) => {
                        let is_cancelled = {
                            let cancelled = cancelled_tasks_loop.lock().unwrap();
                            cancelled.contains(&id)
                        };

                        if is_cancelled {
                            if pid > 0 {
                                let _ = Self::terminate_process(pid);
                            }
                            running = None;
                            {
                                let mut tasks = active_tasks_loop.lock().unwrap();
                                tasks.remove(&id);
                            }
                            Self::process_queue(
                                &app,
                                &tx_clone,
                                &mut queue,
                                &mut queued_ids,
                                &mut running,
                                Arc::clone(&cancelled_tasks_loop),
                            )
                            .await;
                            continue;
                        }

                        let mut tasks = active_tasks_loop.lock().unwrap();
                        tasks.insert(id, pid);
                    }
                    SpatialMessage::TaskCompleted(id) => {
                        running = None;
                        {
                            let mut cancelled = cancelled_tasks_loop.lock().unwrap();
                            cancelled.remove(&id);
                        }
                        {
                            let mut tasks = active_tasks_loop.lock().unwrap();
                            tasks.remove(&id);
                        }

                        Self::process_queue(
                            &app,
                            &tx_clone,
                            &mut queue,
                            &mut queued_ids,
                            &mut running,
                            Arc::clone(&cancelled_tasks_loop),
                        )
                        .await;
                    }
                    SpatialMessage::TaskError(id, err) => {
                        eprintln!("Spatial task {} failed: {}", id, err);

                        let _ = app.emit(
                            "spatial-log",
                            SpatialLogPayload {
                                id: id.clone(),
                                line: format!("[ERROR] {}", err),
                            },
                        );

                        let _ = app.emit(
                            "spatial-error",
                            SpatialErrorPayload {
                                id: id.clone(),
                                error: err.to_string(),
                            },
                        );

                        running = None;
                        {
                            let mut cancelled = cancelled_tasks_loop.lock().unwrap();
                            cancelled.remove(&id);
                        }
                        {
                            let mut tasks = active_tasks_loop.lock().unwrap();
                            tasks.remove(&id);
                        }

                        Self::process_queue(
                            &app,
                            &tx_clone,
                            &mut queue,
                            &mut queued_ids,
                            &mut running,
                            Arc::clone(&cancelled_tasks_loop),
                        )
                        .await;
                    }
                }
            }
        });

        Self {
            sender: tx,
            active_tasks,
            cancelled_tasks,
        }
    }

    async fn process_queue(
        app: &AppHandle,
        tx: &mpsc::Sender<SpatialMessage>,
        queue: &mut VecDeque<SpatialTask>,
        queued_ids: &mut HashSet<String>,
        running: &mut Option<String>,
        cancelled_tasks: Arc<Mutex<HashSet<String>>>,
    ) {
        if running.is_some() {
            return;
        }

        while let Some(task) = queue.pop_front() {
            queued_ids.remove(&task.id);
            let is_cancelled = {
                let mut cancelled = cancelled_tasks.lock().unwrap();
                cancelled.remove(&task.id)
            };
            if is_cancelled {
                continue;
            }

            *running = Some(task.id.clone());

            let app_clone = app.clone();
            let tx_worker = tx.clone();
            let task_clone = task.clone();

            tauri::async_runtime::spawn(async move {
                if let Err(e) = run_spatial_worker(app_clone, tx_worker.clone(), task_clone.clone()).await {
                    let _ = tx_worker
                        .send(SpatialMessage::TaskError(task_clone.id, e))
                        .await;
                } else {
                    let _ = tx_worker
                        .send(SpatialMessage::TaskCompleted(task_clone.id))
                        .await;
                }
            });

            break;
        }
    }

    pub fn cancel_task(&self, id: &str) -> Result<(), SpatialError> {
        {
            let mut cancelled = self.cancelled_tasks.lock().unwrap();
            cancelled.insert(id.to_string());
        }

        let tasks = self.active_tasks.lock().unwrap();
        if let Some(&pid) = tasks.get(id) {
            if pid > 0 {
                Self::terminate_process(pid)?;
            }
        }
        Ok(())
    }

    #[cfg(unix)]
    fn terminate_process(pid: u32) -> Result<(), SpatialError> {
        unsafe {
            let _ = libc::kill(pid as libc::pid_t, libc::SIGCONT);
            if libc::kill(pid as libc::pid_t, libc::SIGKILL) != 0 {
                return Err(SpatialError::Shell("Failed to send SIGKILL".to_string()));
            }
        }
        Ok(())
    }

    #[cfg(windows)]
    fn terminate_process(pid: u32) -> Result<(), SpatialError> {
        unsafe {
            let process_handle = windows::Win32::System::Threading::OpenProcess(
                windows::Win32::System::Threading::PROCESS_TERMINATE,
                false,
                pid,
            )
            .map_err(|e| SpatialError::Shell(format!("Failed to open process: {}", e)))?;

            let _ = windows::Win32::System::Threading::TerminateProcess(process_handle, 1);
            let _ = windows::Win32::Foundation::CloseHandle(process_handle);
        }
        Ok(())
    }
}
