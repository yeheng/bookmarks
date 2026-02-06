use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const SKIP_DIRECTORIES: &[&str] = &[
    "node_modules",
    ".git",
    ".svn",
    ".hg",
    "__pycache__",
    ".cache",
    ".npm",
    ".cargo",
    "target",
    "build",
    "dist",
    ".next",
    ".nuxt",
    "vendor",
    ".idea",
    ".vscode",
    "Library",
    "System",
    "Windows",
    "Program Files",
    "Program Files (x86)",
    "$Recycle.Bin",
];

const SKIP_EXTENSIONS: &[&str] = &[
    "o", "obj", "pyc", "pyo", "class", "dll", "so", "dylib",
    "exe", "bin", "lock", "log",
];

/// Maximum number of files to sample when estimating directory size
const MAX_SAMPLE_FILES: usize = 1000;

/// Multiplier to estimate total file count when sampling reaches limit
const FILE_COUNT_ESTIMATE_MULTIPLIER: usize = 100;

pub struct FileScanner {
    include_hidden: bool,
}

pub struct ScanResult {
    pub files_scanned: usize,
    pub files_indexed: usize,
    pub errors: Vec<String>,
}

impl FileScanner {
    pub fn new(include_hidden: bool) -> Self {
        FileScanner {
            include_hidden,
        }
    }

    pub fn estimate_file_count(&self, path: &Path) -> Result<usize, String> {
        let mut count = 0;

        self.count_files_recursive(path, &mut count, MAX_SAMPLE_FILES)?;

        if count >= MAX_SAMPLE_FILES {
            Ok(count * FILE_COUNT_ESTIMATE_MULTIPLIER)
        } else {
            Ok(count)
        }
    }

    fn count_files_recursive(&self, path: &Path, count: &mut usize, max: usize) -> Result<(), String> {
        if *count >= max {
            return Ok(());
        }

        let entries = fs::read_dir(path)
            .map_err(|e| format!("Failed to read directory {:?}: {}", path, e))?;

        for entry in entries.flatten() {
            if *count >= max {
                return Ok(());
            }

            let entry_path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();

            if !self.include_hidden && file_name.starts_with('.') {
                continue;
            }

            if entry_path.is_dir() {
                if self.should_skip_directory(&file_name) {
                    continue;
                }
                self.count_files_recursive(&entry_path, count, max)?;
            } else {
                *count += 1;
            }
        }

        Ok(())
    }

    pub fn scan_directory(
        &self,
        conn: &Connection,
        directory_id: i64,
        path: &Path,
    ) -> Result<ScanResult, String> {
        let mut result = ScanResult {
            files_scanned: 0,
            files_indexed: 0,
            errors: Vec::new(),
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        self.scan_recursive(conn, directory_id, path, now, &mut result)?;

        conn.execute(
            "UPDATE search_directories SET last_indexed_at = ?1, file_count = ?2 WHERE id = ?3",
            rusqlite::params![now, result.files_indexed as i64, directory_id],
        )
        .map_err(|e| format!("Failed to update directory stats: {}", e))?;

        Ok(result)
    }

    fn scan_recursive(
        &self,
        conn: &Connection,
        directory_id: i64,
        path: &Path,
        indexed_at: i64,
        result: &mut ScanResult,
    ) -> Result<(), String> {
        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(e) => {
                result.errors.push(format!("Failed to read {:?}: {}", path, e));
                return Ok(());
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    result.errors.push(format!("Failed to read entry: {}", e));
                    continue;
                }
            };

            let entry_path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();

            if !self.include_hidden && file_name.starts_with('.') {
                continue;
            }

            result.files_scanned += 1;

            if entry_path.is_dir() {
                if self.should_skip_directory(&file_name) {
                    continue;
                }
                self.scan_recursive(conn, directory_id, &entry_path, indexed_at, result)?;
            } else if entry_path.is_file() {
                if let Err(e) = self.index_file(conn, directory_id, &entry_path, indexed_at) {
                    result.errors.push(e);
                } else {
                    result.files_indexed += 1;
                }
            }
        }

        Ok(())
    }

    fn should_skip_directory(&self, name: &str) -> bool {
        SKIP_DIRECTORIES.iter().any(|&skip| name == skip)
    }

    fn should_skip_file(&self, name: &str) -> bool {
        if let Some(ext) = Path::new(name).extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            SKIP_EXTENSIONS.iter().any(|&skip| ext_str == skip)
        } else {
            false
        }
    }

    fn index_file(
        &self,
        conn: &Connection,
        directory_id: i64,
        path: &Path,
        indexed_at: i64,
    ) -> Result<(), String> {
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        if self.should_skip_file(&file_name) {
            return Ok(());
        }

        let extension = path
            .extension()
            .map(|e| e.to_string_lossy().to_string());

        let metadata = fs::metadata(path)
            .map_err(|e| format!("Failed to get metadata for {:?}: {}", path, e))?;

        let size = metadata.len() as i64;

        let modified_at = metadata
            .modified()
            .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64)
            .unwrap_or(indexed_at);

        let created_at = metadata
            .created()
            .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64)
            .unwrap_or(indexed_at);

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
                indexed_at,
                directory_id
            ],
        )
        .map_err(|e| format!("Failed to insert file {:?}: {}", path, e))?;

        Ok(())
    }

    pub fn remove_directory_files(conn: &Connection, directory_id: i64) -> Result<usize, String> {
        let deleted = conn
            .execute(
                "DELETE FROM indexed_files WHERE directory_id = ?1",
                [directory_id],
            )
            .map_err(|e| format!("Failed to delete files: {}", e))?;

        Ok(deleted)
    }

    pub fn refresh_stale_files(
        conn: &Connection,
        directory_id: i64,
        _directory_path: &Path,
    ) -> Result<(usize, usize), String> {
        let mut updated = 0;
        let mut removed = 0;

        let mut stmt = conn
            .prepare(
                "SELECT id, path, modified_at FROM indexed_files WHERE directory_id = ?1",
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let files: Vec<(i64, String, i64)> = stmt
            .query_map([directory_id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .map_err(|e| format!("Failed to query files: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        drop(stmt);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        for (file_id, file_path, indexed_modified_at) in files {
            let path = PathBuf::from(&file_path);

            if !path.exists() {
                conn.execute("DELETE FROM indexed_files WHERE id = ?1", [file_id])
                    .map_err(|e| format!("Failed to delete missing file: {}", e))?;
                removed += 1;
                continue;
            }

            if let Ok(metadata) = fs::metadata(&path) {
                let current_modified = metadata
                    .modified()
                    .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64)
                    .unwrap_or(0);

                if current_modified > indexed_modified_at {
                    let size = metadata.len() as i64;
                    conn.execute(
                        "UPDATE indexed_files SET size = ?1, modified_at = ?2, indexed_at = ?3 WHERE id = ?4",
                        rusqlite::params![size, current_modified, now, file_id],
                    )
                    .map_err(|e| format!("Failed to update file: {}", e))?;
                    updated += 1;
                }
            }
        }

        Ok((updated, removed))
    }

    pub fn get_default_directories() -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        if let Some(home) = dirs::home_dir() {
            let desktop = home.join("Desktop");
            if desktop.exists() {
                dirs.push(desktop);
            }

            let documents = home.join("Documents");
            if documents.exists() {
                dirs.push(documents);
            }

            let downloads = home.join("Downloads");
            if downloads.exists() {
                dirs.push(downloads);
            }
        }

        dirs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_skip_directory() {
        let scanner = FileScanner::new(false);
        assert!(scanner.should_skip_directory("node_modules"));
        assert!(scanner.should_skip_directory(".git"));
        assert!(!scanner.should_skip_directory("src"));
    }

    #[test]
    fn test_should_skip_file() {
        let scanner = FileScanner::new(false);
        assert!(scanner.should_skip_file("test.pyc"));
        assert!(scanner.should_skip_file("main.o"));
        assert!(!scanner.should_skip_file("document.pdf"));
    }
}
