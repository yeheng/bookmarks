use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{fs, thread};

const DEBOUNCE_MS: u64 = 100;

pub struct FileWatcher {
    watchers: Arc<Mutex<HashMap<i64, WatcherHandle>>>,
    event_sender: Sender<FileEvent>,
}

struct WatcherHandle {
    #[allow(dead_code)]
    watcher: RecommendedWatcher,
    path: PathBuf,
}

#[derive(Debug, Clone)]
pub enum FileEvent {
    Created { path: PathBuf, directory_id: i64 },
    Modified { path: PathBuf, directory_id: i64 },
    Deleted { path: PathBuf, directory_id: i64 },
    Renamed { from: PathBuf, to: PathBuf, directory_id: i64 },
}

pub struct FileWatcherConfig {
    pub include_hidden: bool,
}

impl Default for FileWatcherConfig {
    fn default() -> Self {
        FileWatcherConfig {
            include_hidden: false,
        }
    }
}

impl FileWatcher {
    pub fn new() -> (Self, Receiver<FileEvent>) {
        let (event_sender, event_receiver) = channel();

        let watcher = FileWatcher {
            watchers: Arc::new(Mutex::new(HashMap::new())),
            event_sender,
        };

        (watcher, event_receiver)
    }

    pub fn watch_directory(&self, directory_id: i64, path: &Path) -> Result<(), String> {
        let path = path.to_path_buf();
        let event_sender = self.event_sender.clone();
        let dir_id = directory_id;

        let (tx, rx) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            Config::default().with_poll_interval(Duration::from_secs(2)),
        )
        .map_err(|e| format!("Failed to create watcher: {}", e))?;

        watcher
            .watch(&path, RecursiveMode::Recursive)
            .map_err(|e| format!("Failed to watch directory: {}", e))?;

        let path_clone = path.clone();
        thread::spawn(move || {
            let mut pending_events: HashMap<PathBuf, (FileEvent, Instant)> = HashMap::new();

            loop {
                match rx.recv_timeout(Duration::from_millis(DEBOUNCE_MS)) {
                    Ok(event) => {
                        for event_path in event.paths {
                            let file_event = match event.kind {
                                EventKind::Create(_) => Some(FileEvent::Created {
                                    path: event_path.clone(),
                                    directory_id: dir_id,
                                }),
                                EventKind::Modify(_) => Some(FileEvent::Modified {
                                    path: event_path.clone(),
                                    directory_id: dir_id,
                                }),
                                EventKind::Remove(_) => Some(FileEvent::Deleted {
                                    path: event_path.clone(),
                                    directory_id: dir_id,
                                }),
                                _ => None,
                            };

                            if let Some(fe) = file_event {
                                pending_events.insert(event_path, (fe, Instant::now()));
                            }
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        let now = Instant::now();
                        let debounce_duration = Duration::from_millis(DEBOUNCE_MS);

                        let ready_events: Vec<_> = pending_events
                            .iter()
                            .filter(|(_, (_, timestamp))| now.duration_since(*timestamp) >= debounce_duration)
                            .map(|(path, (event, _))| (path.clone(), event.clone()))
                            .collect();

                        for (path, event) in ready_events {
                            pending_events.remove(&path);
                            let _ = event_sender.send(event);
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                        break;
                    }
                }
            }
        });

        let mut watchers = self.watchers.lock().unwrap();
        watchers.insert(
            directory_id,
            WatcherHandle {
                watcher,
                path: path_clone,
            },
        );

        Ok(())
    }

    pub fn unwatch_directory(&self, directory_id: i64) -> Result<(), String> {
        let mut watchers = self.watchers.lock().unwrap();
        watchers.remove(&directory_id);
        Ok(())
    }

    pub fn is_watching(&self, directory_id: i64) -> bool {
        let watchers = self.watchers.lock().unwrap();
        watchers.contains_key(&directory_id)
    }

    pub fn get_watched_directories(&self) -> Vec<(i64, PathBuf)> {
        let watchers = self.watchers.lock().unwrap();
        watchers
            .iter()
            .map(|(id, handle)| (*id, handle.path.clone()))
            .collect()
    }
}

pub fn process_file_event(conn: &Connection, event: FileEvent, include_hidden: bool) -> Result<(), String> {
    match event {
        FileEvent::Created { path, directory_id } => {
            handle_file_created(conn, &path, directory_id, include_hidden)
        }
        FileEvent::Modified { path, directory_id } => {
            handle_file_modified(conn, &path, directory_id)
        }
        FileEvent::Deleted { path, .. } => {
            handle_file_deleted(conn, &path)
        }
        FileEvent::Renamed { from, to, directory_id } => {
            handle_file_deleted(conn, &from)?;
            handle_file_created(conn, &to, directory_id, include_hidden)
        }
    }
}

fn handle_file_created(
    conn: &Connection,
    path: &Path,
    directory_id: i64,
    include_hidden: bool,
) -> Result<(), String> {
    if !path.is_file() {
        return Ok(());
    }

    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    if !include_hidden && file_name.starts_with('.') {
        return Ok(());
    }

    let extension = path.extension().map(|e| e.to_string_lossy().to_string());

    let metadata = fs::metadata(path)
        .map_err(|e| format!("Failed to get metadata: {}", e))?;

    let size = metadata.len() as i64;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let modified_at = metadata
        .modified()
        .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64)
        .unwrap_or(now);

    let created_at = metadata
        .created()
        .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64)
        .unwrap_or(now);

    let path_str = path.to_string_lossy().to_string();

    conn.execute(
        "INSERT OR REPLACE INTO indexed_files
         (path, name, extension, size, modified_at, created_at, indexed_at, directory_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            path_str,
            file_name,
            extension,
            size,
            modified_at,
            created_at,
            now,
            directory_id
        ],
    )
    .map_err(|e| format!("Failed to insert file: {}", e))?;

    conn.execute(
        "UPDATE search_directories SET file_count = file_count + 1 WHERE id = ?1",
        [directory_id],
    )
    .map_err(|e| format!("Failed to update file count: {}", e))?;

    Ok(())
}

fn handle_file_modified(conn: &Connection, path: &Path, _directory_id: i64) -> Result<(), String> {
    if !path.is_file() {
        return Ok(());
    }

    let metadata = fs::metadata(path)
        .map_err(|e| format!("Failed to get metadata: {}", e))?;

    let size = metadata.len() as i64;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let modified_at = metadata
        .modified()
        .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64)
        .unwrap_or(now);

    let path_str = path.to_string_lossy().to_string();

    conn.execute(
        "UPDATE indexed_files SET size = ?1, modified_at = ?2, indexed_at = ?3 WHERE path = ?4",
        rusqlite::params![size, modified_at, now, path_str],
    )
    .map_err(|e| format!("Failed to update file: {}", e))?;

    Ok(())
}

fn handle_file_deleted(conn: &Connection, path: &Path) -> Result<(), String> {
    let path_str = path.to_string_lossy().to_string();

    let directory_id: Option<i64> = conn
        .query_row(
            "SELECT directory_id FROM indexed_files WHERE path = ?1",
            [&path_str],
            |row| row.get(0),
        )
        .ok();

    let deleted = conn
        .execute("DELETE FROM indexed_files WHERE path = ?1", [&path_str])
        .map_err(|e| format!("Failed to delete file: {}", e))?;

    if deleted > 0 {
        if let Some(dir_id) = directory_id {
            conn.execute(
                "UPDATE search_directories SET file_count = file_count - 1 WHERE id = ?1 AND file_count > 0",
                [dir_id],
            )
            .map_err(|e| format!("Failed to update file count: {}", e))?;
        }
    }

    Ok(())
}
