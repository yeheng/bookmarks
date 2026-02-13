//! Plugin runtime executor — spawns subprocesses and communicates via JSON stdin/stdout.

use crate::error::AppError;
use crate::plugins::manifest::PluginCommand;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Default execution timeout in seconds.
const DEFAULT_TIMEOUT_SECS: u64 = 10;

/// Maximum number of cached plugin results.
/// Prevents unbounded memory growth from plugin caching.
const MAX_CACHE_SIZE: usize = 1000;

/// Request sent to plugin via stdin.
#[derive(Debug, Serialize)]
pub struct PluginRequest {
    pub command: String,
    pub query: String,
    pub preferences: HashMap<String, String>,
}

/// Response from plugin via stdout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResponse {
    #[serde(default)]
    pub items: Vec<PluginResultItem>,
    #[serde(default)]
    pub cache: Option<CacheDirective>,
}

/// A single result item from a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResultItem {
    pub uid: String,
    pub title: String,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub arg: Option<String>,
    #[serde(default)]
    pub icon: Option<PluginIcon>,
    #[serde(default)]
    pub actions: Vec<PluginAction>,
    #[serde(default)]
    pub badge: Option<String>,
}

/// Plugin result icon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginIcon {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub emoji: Option<String>,
}

/// An action that can be performed on a result item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PluginAction {
    #[serde(rename = "open-url")]
    OpenUrl {
        url: String,
        #[serde(default)]
        title: Option<String>,
    },
    #[serde(rename = "copy")]
    Copy {
        text: String,
        #[serde(default)]
        title: Option<String>,
    },
    #[serde(rename = "paste")]
    Paste {
        text: String,
        #[serde(default)]
        title: Option<String>,
    },
    #[serde(rename = "open-file")]
    OpenFile {
        path: String,
        #[serde(default)]
        title: Option<String>,
    },
    #[serde(rename = "run-command")]
    RunCommand {
        command: String,
        #[serde(default)]
        arg: Option<String>,
        #[serde(default)]
        title: Option<String>,
    },
}

/// Cache directive from the plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheDirective {
    pub ttl_seconds: u64,
}

/// Cache entry.
struct CacheEntry {
    response: PluginResponse,
    expires_at: Instant,
}

/// Plugin executor — runs plugins in subprocesses with timeout and caching.
pub struct PluginExecutor {
    /// Result cache: (plugin_id, command, query) → response.
    cache: Mutex<HashMap<String, CacheEntry>>,
}

impl PluginExecutor {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// Execute a plugin command.
    pub fn execute(
        &self,
        plugin_dir: &Path,
        command: &PluginCommand,
        query: &str,
        preferences: HashMap<String, String>,
        api_version: &str,
    ) -> Result<PluginResponse, AppError> {
        let cache_key = format!("{}:{}:{}", plugin_dir.display(), command.name, query);

        // Check cache
        if let Some(cached) = self.check_cache(&cache_key) {
            return Ok(cached);
        }

        // Resolve runtime executable
        let (program, args) = resolve_runtime(&command.runtime, &command.script, plugin_dir)?;

        // Build request
        let request = PluginRequest {
            command: command.name.clone(),
            query: query.to_string(),
            preferences: preferences.clone(),
        };
        let request_json = serde_json::to_string(&request)
            .map_err(|e| AppError::Generic(format!("Failed to serialize request: {}", e)))?;

        // Build environment
        let env = build_env(plugin_dir, api_version, &preferences);

        // Spawn subprocess
        let timeout = Duration::from_secs(command.timeout.unwrap_or(DEFAULT_TIMEOUT_SECS));
        let result = self.run_subprocess(&program, &args, plugin_dir, &request_json, &env, timeout)?;

        // Log stderr if any
        if !result.stderr.is_empty() {
            log_stderr(plugin_dir, &result.stderr);
        }

        // Parse response
        let response: PluginResponse = serde_json::from_str(&result.stdout)
            .map_err(|e| AppError::Generic(format!(
                "Plugin returned invalid JSON: {}. Output: {}",
                e,
                &result.stdout[..result.stdout.len().min(200)]
            )))?;

        // Store in cache if plugin requests it
        if let Some(ref cache) = response.cache {
            self.store_cache(cache_key, &response, cache.ttl_seconds);
        }

        Ok(response)
    }

    fn check_cache(&self, key: &str) -> Option<PluginResponse> {
        let cache = self.cache.lock().ok()?;
        let entry = cache.get(key)?;
        if Instant::now() < entry.expires_at {
            Some(entry.response.clone())
        } else {
            None
        }
    }

    fn store_cache(&self, key: String, response: &PluginResponse, ttl_secs: u64) {
        if let Ok(mut cache) = self.cache.lock() {
            // Evict expired entries first
            let now = Instant::now();
            cache.retain(|_, v| v.expires_at > now);

            // If cache is full, evict oldest 10% of entries
            if cache.len() >= MAX_CACHE_SIZE {
                let evict_count = MAX_CACHE_SIZE / 10;
                let keys_to_remove: Vec<_> = cache.keys().take(evict_count).cloned().collect();
                for k in keys_to_remove {
                    cache.remove(&k);
                }
            }

            cache.insert(key, CacheEntry {
                response: response.clone(),
                expires_at: Instant::now() + Duration::from_secs(ttl_secs),
            });
        }
    }

    fn run_subprocess(
        &self,
        program: &str,
        args: &[String],
        working_dir: &Path,
        stdin_data: &str,
        env: &HashMap<String, String>,
        timeout: Duration,
    ) -> Result<SubprocessResult, AppError> {
        let mut child = Command::new(program)
            .args(args)
            .current_dir(working_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .envs(env)
            .spawn()
            .map_err(|e| AppError::Generic(format!("Failed to spawn plugin process '{}': {}", program, e)))?;

        // Write stdin
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(stdin_data.as_bytes());
            // stdin is dropped here, closing the pipe
        }

        // Wait with timeout
        let start = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    let output = child.wait_with_output()
                        .map_err(|e| AppError::Generic(format!("Failed to read process output: {}", e)))?;

                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                    if !status.success() && stdout.is_empty() {
                        return Err(AppError::Generic(format!(
                            "Plugin exited with code {:?}. stderr: {}",
                            status.code(),
                            &stderr[..stderr.len().min(500)]
                        )));
                    }

                    return Ok(SubprocessResult { stdout, stderr });
                }
                Ok(None) => {
                    if start.elapsed() > timeout {
                        let _ = child.kill();
                        return Err(AppError::Generic(format!(
                            "Plugin timed out after {}s",
                            timeout.as_secs()
                        )));
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(e) => {
                    return Err(AppError::Generic(format!("Failed to check process status: {}", e)));
                }
            }
        }
    }

    /// Read the latest log file for a plugin.
    pub fn get_log(&self, plugin_dir: &Path) -> Result<String, AppError> {
        let log_path = plugin_dir.join("logs").join("latest.log");
        if log_path.exists() {
            Ok(std::fs::read_to_string(&log_path)?)
        } else {
            Ok(String::new())
        }
    }
}

struct SubprocessResult {
    stdout: String,
    stderr: String,
}

/// Resolve the runtime command and arguments.
fn resolve_runtime(runtime: &str, script: &str, plugin_dir: &Path) -> Result<(String, Vec<String>), AppError> {
    let script_path = plugin_dir.join(script);

    match runtime {
        "node" => {
            check_runtime_available("node")?;
            Ok(("node".to_string(), vec![script_path.to_string_lossy().to_string()]))
        }
        "python" => {
            // Try python3 first, then python
            if which("python3") {
                Ok(("python3".to_string(), vec![script_path.to_string_lossy().to_string()]))
            } else if which("python") {
                Ok(("python".to_string(), vec![script_path.to_string_lossy().to_string()]))
            } else {
                Err(AppError::Generic("Python runtime not found. Please install Python 3.".to_string()))
            }
        }
        "bash" => {
            check_runtime_available("bash")?;
            Ok(("bash".to_string(), vec![script_path.to_string_lossy().to_string()]))
        }
        "binary" => {
            if !script_path.exists() {
                return Err(AppError::Generic(format!("Binary not found: {}", script_path.display())));
            }
            Ok((script_path.to_string_lossy().to_string(), vec![]))
        }
        _ => Err(AppError::Generic(format!("Unknown runtime: {}", runtime))),
    }
}

fn check_runtime_available(name: &str) -> Result<(), AppError> {
    if !which(name) {
        Err(AppError::Generic(format!(
            "Runtime '{}' not found. Please install it.",
            name
        )))
    } else {
        Ok(())
    }
}

/// Check if a command is available on PATH.
fn which(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Build environment variables for plugin subprocess.
fn build_env(plugin_dir: &Path, api_version: &str, preferences: &HashMap<String, String>) -> HashMap<String, String> {
    let mut env = HashMap::new();

    env.insert("LAUNCHER_PLUGIN_DIR".to_string(), plugin_dir.to_string_lossy().to_string());
    env.insert(
        "LAUNCHER_DATA_DIR".to_string(),
        plugin_dir.join("data").to_string_lossy().to_string(),
    );
    env.insert("LAUNCHER_API_VERSION".to_string(), api_version.to_string());

    // Inject preferences as LAUNCHER_PREF_{UPPER_CASE_NAME}
    for (key, value) in preferences {
        let env_key = format!("LAUNCHER_PREF_{}", key.to_uppercase().replace('-', "_"));
        env.insert(env_key, value.clone());
    }

    env
}

/// Append stderr output to the plugin's log file.
fn log_stderr(plugin_dir: &Path, stderr: &str) {
    let log_dir = plugin_dir.join("logs");
    let _ = std::fs::create_dir_all(&log_dir);
    let log_path = log_dir.join("latest.log");

    let timestamp = chrono::Utc::now().to_rfc3339();
    let log_entry = format!("[{}] {}\n", timestamp, stderr);

    // Append to log file (truncate if too large)
    if let Ok(metadata) = std::fs::metadata(&log_path) {
        if metadata.len() > 1_000_000 {
            // > 1MB, truncate
            let _ = std::fs::write(&log_path, &log_entry);
            return;
        }
    }

    use std::fs::OpenOptions;
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&log_path) {
        let _ = file.write_all(log_entry.as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_script(dir: &Path, filename: &str, content: &str) -> PathBuf {
        let script_path = dir.join(filename);
        std::fs::write(&script_path, content).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        script_path
    }

    #[test]
    fn test_execute_bash_plugin() {
        let tmp = TempDir::new().unwrap();
        let plugin_dir = tmp.path();

        create_test_script(
            plugin_dir,
            "search.sh",
            r#"#!/bin/bash
read input
echo '{"items":[{"uid":"1","title":"Hello","subtitle":"World"}]}'
"#,
        );

        let cmd = PluginCommand {
            name: "search".to_string(),
            title: "Search".to_string(),
            description: "Test".to_string(),
            keyword: "test".to_string(),
            mode: "search".to_string(),
            script: "search.sh".to_string(),
            runtime: "bash".to_string(),
            timeout: Some(5),
        };

        let executor = PluginExecutor::new();
        let response = executor.execute(
            plugin_dir,
            &cmd,
            "test query",
            HashMap::new(),
            "0.1",
        ).unwrap();

        assert_eq!(response.items.len(), 1);
        assert_eq!(response.items[0].title, "Hello");
        assert_eq!(response.items[0].subtitle.as_deref(), Some("World"));
    }

    #[test]
    fn test_execute_timeout() {
        let tmp = TempDir::new().unwrap();
        let plugin_dir = tmp.path();

        create_test_script(
            plugin_dir,
            "slow.sh",
            r#"#!/bin/bash
sleep 30
echo '{"items":[]}'
"#,
        );

        let cmd = PluginCommand {
            name: "slow".to_string(),
            title: "Slow".to_string(),
            description: "Test".to_string(),
            keyword: "slow".to_string(),
            mode: "search".to_string(),
            script: "slow.sh".to_string(),
            runtime: "bash".to_string(),
            timeout: Some(1), // 1 second timeout
        };

        let executor = PluginExecutor::new();
        let result = executor.execute(
            plugin_dir,
            &cmd,
            "test",
            HashMap::new(),
            "0.1",
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timed out"));
    }

    #[test]
    fn test_execute_invalid_json() {
        let tmp = TempDir::new().unwrap();
        let plugin_dir = tmp.path();

        create_test_script(
            plugin_dir,
            "bad.sh",
            r#"#!/bin/bash
read input
echo 'not valid json'
"#,
        );

        let cmd = PluginCommand {
            name: "bad".to_string(),
            title: "Bad".to_string(),
            description: "Test".to_string(),
            keyword: "bad".to_string(),
            mode: "search".to_string(),
            script: "bad.sh".to_string(),
            runtime: "bash".to_string(),
            timeout: Some(5),
        };

        let executor = PluginExecutor::new();
        let result = executor.execute(
            plugin_dir,
            &cmd,
            "test",
            HashMap::new(),
            "0.1",
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid JSON"));
    }

    #[test]
    fn test_execute_with_crash() {
        let tmp = TempDir::new().unwrap();
        let plugin_dir = tmp.path();

        create_test_script(
            plugin_dir,
            "crash.sh",
            r#"#!/bin/bash
echo "error message" >&2
exit 1
"#,
        );

        let cmd = PluginCommand {
            name: "crash".to_string(),
            title: "Crash".to_string(),
            description: "Test".to_string(),
            keyword: "crash".to_string(),
            mode: "search".to_string(),
            script: "crash.sh".to_string(),
            runtime: "bash".to_string(),
            timeout: Some(5),
        };

        let executor = PluginExecutor::new();
        let result = executor.execute(plugin_dir, &cmd, "test", HashMap::new(), "0.1");

        assert!(result.is_err());
    }

    #[test]
    fn test_result_caching() {
        let tmp = TempDir::new().unwrap();
        let plugin_dir = tmp.path();

        // Script that includes cache directive
        create_test_script(
            plugin_dir,
            "cached.sh",
            r#"#!/bin/bash
read input
echo '{"items":[{"uid":"1","title":"Cached"}],"cache":{"ttl_seconds":300}}'
"#,
        );

        let cmd = PluginCommand {
            name: "cached".to_string(),
            title: "Cached".to_string(),
            description: "Test".to_string(),
            keyword: "cache".to_string(),
            mode: "search".to_string(),
            script: "cached.sh".to_string(),
            runtime: "bash".to_string(),
            timeout: Some(5),
        };

        let executor = PluginExecutor::new();

        // First call — executes script
        let r1 = executor.execute(plugin_dir, &cmd, "test", HashMap::new(), "0.1").unwrap();
        assert_eq!(r1.items.len(), 1);

        // Second call — should hit cache (no subprocess spawned)
        let r2 = executor.execute(plugin_dir, &cmd, "test", HashMap::new(), "0.1").unwrap();
        assert_eq!(r2.items.len(), 1);
    }

    #[test]
    fn test_environment_variables() {
        let tmp = TempDir::new().unwrap();
        let plugin_dir = tmp.path();

        create_test_script(
            plugin_dir,
            "env.sh",
            r#"#!/bin/bash
read input
echo "{\"items\":[{\"uid\":\"1\",\"title\":\"$LAUNCHER_PLUGIN_DIR\",\"subtitle\":\"$LAUNCHER_PREF_API_KEY\"}]}"
"#,
        );

        let cmd = PluginCommand {
            name: "env".to_string(),
            title: "Env".to_string(),
            description: "Test".to_string(),
            keyword: "env".to_string(),
            mode: "search".to_string(),
            script: "env.sh".to_string(),
            runtime: "bash".to_string(),
            timeout: Some(5),
        };

        let mut prefs = HashMap::new();
        prefs.insert("api_key".to_string(), "secret123".to_string());

        let executor = PluginExecutor::new();
        let result = executor.execute(plugin_dir, &cmd, "test", prefs, "0.1").unwrap();

        assert_eq!(result.items[0].title, plugin_dir.to_string_lossy().to_string());
        assert_eq!(result.items[0].subtitle.as_deref(), Some("secret123"));
    }

    #[test]
    fn test_stderr_logging() {
        let tmp = TempDir::new().unwrap();
        let plugin_dir = tmp.path();

        create_test_script(
            plugin_dir,
            "warn.sh",
            r#"#!/bin/bash
read input
echo "debug info" >&2
echo '{"items":[]}'
"#,
        );

        let cmd = PluginCommand {
            name: "warn".to_string(),
            title: "Warn".to_string(),
            description: "Test".to_string(),
            keyword: "warn".to_string(),
            mode: "search".to_string(),
            script: "warn.sh".to_string(),
            runtime: "bash".to_string(),
            timeout: Some(5),
        };

        let executor = PluginExecutor::new();
        executor.execute(plugin_dir, &cmd, "test", HashMap::new(), "0.1").unwrap();

        // Check log file was created
        let log_content = executor.get_log(plugin_dir).unwrap();
        assert!(log_content.contains("debug info"));
    }
}
