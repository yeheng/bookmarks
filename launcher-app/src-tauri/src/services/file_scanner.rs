use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

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

const MAX_SAMPLE_FILES: usize = 1000;
const FILE_COUNT_ESTIMATE_MULTIPLIER: usize = 100;

pub struct FileScanner {
    include_hidden: bool,
}

pub struct ScanResult {
    pub files_scanned: usize,
    pub files_indexed: usize,
    pub errors: Vec<String>,
}

/// File data tuple for Tantivy indexing: (id, path, name, extension, size, modified_at, directory_id)
pub type FileDataTuple = (i64, String, String, Option<String>, i64, i64, i64);

impl FileScanner {
    pub fn new(include_hidden: bool) -> Self {
        FileScanner { include_hidden }
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

    /// Scan a directory and return file data tuples for Tantivy indexing.
    /// No longer writes to SQLite — returns data directly.
    pub fn scan_directory_for_tantivy(
        &self,
        directory_id: i64,
        path: &Path,
    ) -> Result<(ScanResult, Vec<FileDataTuple>), String> {
        let mut result = ScanResult {
            files_scanned: 0,
            files_indexed: 0,
            errors: Vec::new(),
        };

        let mut files = Vec::new();
        let mut next_file_id: i64 = 1;

        self.scan_recursive_tantivy(directory_id, path, &mut result, &mut files, &mut next_file_id)?;

        Ok((result, files))
    }

    fn scan_recursive_tantivy(
        &self,
        directory_id: i64,
        path: &Path,
        result: &mut ScanResult,
        files: &mut Vec<FileDataTuple>,
        next_id: &mut i64,
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
                self.scan_recursive_tantivy(directory_id, &entry_path, result, files, next_id)?;
            } else if entry_path.is_file() {
                if let Some(file_data) = self.collect_file_data(*next_id, directory_id, &entry_path) {
                    files.push(file_data);
                    result.files_indexed += 1;
                    *next_id += 1;
                }
            }
        }

        Ok(())
    }

    fn collect_file_data(
        &self,
        file_id: i64,
        directory_id: i64,
        path: &Path,
    ) -> Option<FileDataTuple> {
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        if self.should_skip_file(&file_name) {
            return None;
        }

        let extension = path
            .extension()
            .map(|e| e.to_string_lossy().to_string());

        let metadata = fs::metadata(path).ok()?;
        let size = metadata.len() as i64;

        let modified_at = metadata
            .modified()
            .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64)
            .unwrap_or(0);

        let path_str = path.to_string_lossy().to_string();

        Some((file_id, path_str, file_name, extension, size, modified_at, directory_id))
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
