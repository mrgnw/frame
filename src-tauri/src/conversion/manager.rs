use std::collections::{HashMap, VecDeque};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

use crate::conversion::types::{ErrorPayload, LogPayload};

#[cfg(unix)]
use libc;

#[cfg(windows)]
use windows::{
    Win32::{
        Foundation::{CloseHandle, HANDLE},
        System::{
            LibraryLoader::{GetModuleHandleA, GetProcAddress},
            Threading::{OpenProcess, PROCESS_SUSPEND_RESUME},
        },
    },
    core::s,
};

use crate::conversion::error::ConversionError;
use crate::conversion::worker::run_ffmpeg_worker;
use crate::conversion::types::{ConversionTask, DEFAULT_MAX_CONCURRENCY};

pub enum ManagerMessage {
    Enqueue(ConversionTask),
    TaskStarted(String, u32),
    TaskCompleted(String),
    TaskError(String, ConversionError),
}

pub struct ConversionManager {
    pub(crate) sender: mpsc::Sender<ManagerMessage>,
    max_concurrency: Arc<AtomicUsize>,
    active_tasks: Arc<Mutex<HashMap<String, u32>>>,
}

impl ConversionManager {
    pub fn new(app: AppHandle) -> Self {
        let (tx, mut rx) = mpsc::channel(32);
        let tx_clone = tx.clone();
        let max_concurrency = Arc::new(AtomicUsize::new(DEFAULT_MAX_CONCURRENCY));
        let limiter = Arc::clone(&max_concurrency);
        let active_tasks = Arc::new(Mutex::new(HashMap::new()));
        let active_tasks_loop = Arc::clone(&active_tasks);

        tauri::async_runtime::spawn(async move {
            let mut queue: VecDeque<ConversionTask> = VecDeque::new();
            let mut running_tasks: HashMap<String, ()> = HashMap::new();

            while let Some(msg) = rx.recv().await {
                match msg {
                    ManagerMessage::Enqueue(task) => {
                        queue.push_back(task);
                        ConversionManager::process_queue(
                            &app,
                            &tx_clone,
                            &mut queue,
                            &mut running_tasks,
                            Arc::clone(&limiter),
                        )
                        .await;
                    }
                    ManagerMessage::TaskStarted(id, pid) => {
                        let mut tasks = active_tasks_loop.lock().unwrap();
                        tasks.insert(id, pid);
                    }
                    ManagerMessage::TaskCompleted(id) => {
                        running_tasks.remove(&id);
                        {
                            let mut tasks = active_tasks_loop.lock().unwrap();
                            tasks.remove(&id);
                        }

                        ConversionManager::process_queue(
                            &app,
                            &tx_clone,
                            &mut queue,
                            &mut running_tasks,
                            Arc::clone(&limiter),
                        )
                        .await;
                    }
                    ManagerMessage::TaskError(id, err) => {
                        eprintln!("Task {} failed: {}", id, err);

                        let _ = app.emit(
                            "conversion-log",
                            LogPayload {
                                id: id.clone(),
                                line: format!("[ERROR] {}", err),
                            },
                        );

                        let _ = app.emit(
                            "conversion-error",
                            ErrorPayload {
                                id: id.clone(),
                                error: err.to_string(),
                            },
                        );

                        running_tasks.remove(&id);
                        {
                            let mut tasks = active_tasks_loop.lock().unwrap();
                            tasks.remove(&id);
                        }

                        ConversionManager::process_queue(
                            &app,
                            &tx_clone,
                            &mut queue,
                            &mut running_tasks,
                            Arc::clone(&limiter),
                        )
                        .await;
                    }
                }
            }
        });

        Self {
            sender: tx,
            max_concurrency,
            active_tasks,
        }
    }

    async fn process_queue(
        app: &AppHandle,
        tx: &mpsc::Sender<ManagerMessage>,
        queue: &mut VecDeque<ConversionTask>,
        running_tasks: &mut HashMap<String, ()>,
        max_concurrency: Arc<AtomicUsize>,
    ) {
        let limit = max_concurrency.load(Ordering::SeqCst).max(1);

        while running_tasks.len() < limit {
            if let Some(task) = queue.pop_front() {
                running_tasks.insert(task.id.clone(), ());

                let app_clone = app.clone();
                let tx_worker = tx.clone();
                let task_clone = task.clone();

                tauri::async_runtime::spawn(async move {
                    if let Err(e) =
                        run_ffmpeg_worker(app_clone, tx_worker.clone(), task_clone.clone()).await
                    {
                        let _ = tx_worker
                            .send(ManagerMessage::TaskError(task_clone.id, e))
                            .await;
                    } else {
                        let _ = tx_worker
                            .send(ManagerMessage::TaskCompleted(task_clone.id))
                            .await;
                    }
                });
            } else {
                break;
            }
        }
    }

    pub fn current_max_concurrency(&self) -> usize {
        self.max_concurrency.load(Ordering::SeqCst)
    }

    pub fn update_max_concurrency(&self, value: usize) -> Result<(), ConversionError> {
        if value == 0 {
            return Err(ConversionError::InvalidInput(
                "Max concurrency must be at least 1".to_string(),
            ));
        }
        self.max_concurrency.store(value, Ordering::SeqCst);
        Ok(())
    }

    pub fn pause_task(&self, id: &str) -> Result<(), ConversionError> {
        let tasks = self.active_tasks.lock().unwrap();
        if let Some(&pid) = tasks.get(id) {
            #[cfg(unix)]
            unsafe {
                if libc::kill(pid as libc::pid_t, libc::SIGSTOP) != 0 {
                    return Err(ConversionError::Shell("Failed to send SIGSTOP".to_string()));
                }
            }

            #[cfg(windows)]
            unsafe {
                windows_suspend_resume(pid, true)?;
            }

            Ok(())
        } else {
            Err(ConversionError::TaskNotFound(id.to_string()))
        }
    }

    pub fn resume_task(&self, id: &str) -> Result<(), ConversionError> {
        let tasks = self.active_tasks.lock().unwrap();
        if let Some(&pid) = tasks.get(id) {
            #[cfg(unix)]
            unsafe {
                if libc::kill(pid as libc::pid_t, libc::SIGCONT) != 0 {
                    return Err(ConversionError::Shell("Failed to send SIGCONT".to_string()));
                }
            }

            #[cfg(windows)]
            unsafe {
                windows_suspend_resume(pid, false)?;
            }

            Ok(())
        } else {
            Err(ConversionError::TaskNotFound(id.to_string()))
        }
    }

    pub fn cancel_task(&self, id: &str) -> Result<(), ConversionError> {
        let tasks = self.active_tasks.lock().unwrap();
        if let Some(&pid) = tasks.get(id) {
            // First resume the process to ensure it can handle the kill signal properly
            #[cfg(unix)]
            unsafe {
                let _ = libc::kill(pid as libc::pid_t, libc::SIGCONT);
                if libc::kill(pid as libc::pid_t, libc::SIGKILL) != 0 {
                    return Err(ConversionError::Shell("Failed to send SIGKILL".to_string()));
                }
            }

            #[cfg(windows)]
            unsafe {
                // Resume first just in case
                let _ = windows_suspend_resume(pid, false);

                let process_handle = OpenProcess(
                    windows::Win32::System::Threading::PROCESS_TERMINATE,
                    false,
                    pid,
                )
                .map_err(|e| {
                    ConversionError::Shell(format!("Failed to open process for termination: {}", e))
                })?;

                let _ = windows::Win32::System::Threading::TerminateProcess(process_handle, 1);
                let _ = CloseHandle(process_handle);
            }

            // Cleanup temp directory for ML upscale tasks
            let temp_dir = std::env::temp_dir().join(format!("frame_upscale_{}", id));
            if temp_dir.exists() {
                let _ = std::fs::remove_dir_all(&temp_dir);
            }

            Ok(())
        } else {
            // Task might not be running yet or already finished, which is fine for cancel
            // Still try to cleanup temp dir in case it exists
            let temp_dir = std::env::temp_dir().join(format!("frame_upscale_{}", id));
            if temp_dir.exists() {
                let _ = std::fs::remove_dir_all(&temp_dir);
            }
            Ok(())
        }
    }
}

#[cfg(windows)]
unsafe fn windows_suspend_resume(pid: u32, suspend: bool) -> Result<(), ConversionError> {
    let process_handle = OpenProcess(PROCESS_SUSPEND_RESUME, false, pid)
        .map_err(|e| ConversionError::Shell(format!("Failed to open process: {}", e)))?;

    let ntdll = GetModuleHandleA(s!("ntdll.dll")).map_err(|e| {
        let _ = CloseHandle(process_handle);
        ConversionError::Shell(format!("Failed to get ntdll handle: {}", e))
    })?;

    let fn_name = if suspend {
        s!("NtSuspendProcess")
    } else {
        s!("NtResumeProcess")
    };

    let func_ptr = GetProcAddress(ntdll, fn_name);

    if let Some(func) = func_ptr {
        let func: extern "system" fn(HANDLE) -> i32 = std::mem::transmute(func);
        let status = func(process_handle);
        let _ = CloseHandle(process_handle);

        if status != 0 {
            return Err(ConversionError::Shell(format!(
                "NtSuspendProcess/NtResumeProcess failed with status: {}",
                status
            )));
        }
        Ok(())
    } else {
        let _ = CloseHandle(process_handle);
        Err(ConversionError::Shell(
            "Could not find NtSuspendProcess/NtResumeProcess in ntdll".to_string(),
        ))
    }
}
