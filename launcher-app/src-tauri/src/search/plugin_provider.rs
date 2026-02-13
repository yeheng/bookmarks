//! Plugin search provider — wraps PluginExecutor for plugin-based search.
//!
//! Detects plugin keywords server-side and delegates to the plugin executor.
//! This provider handles queries of the form "keyword rest-of-query"
//! and routes them to the matching plugin command.

use async_trait::async_trait;
use std::sync::Arc;

use crate::plugins::executor::PluginExecutor;
use crate::plugins::registry::PluginRegistry;
use crate::services::data_service::DataService;
use super::engine::SearchError;
use super::provider::{ProviderResult, SearchContext, SearchProvider, SourceType};

/// Provides search results from installed plugins.
///
/// Examines the query for a plugin keyword prefix. If found, the query
/// is routed to the matching plugin command via `PluginExecutor`.
/// If no keyword matches, returns empty results.
pub struct PluginSearchProvider {
    registry: Arc<PluginRegistry>,
    executor: Arc<PluginExecutor>,
    data_service: Arc<DataService>,
}

impl PluginSearchProvider {
    pub fn new(
        registry: Arc<PluginRegistry>,
        executor: Arc<PluginExecutor>,
        data_service: Arc<DataService>,
    ) -> Self {
        Self {
            registry,
            executor,
            data_service,
        }
    }

    /// Detect if query starts with a plugin keyword.
    /// Returns (keyword, rest_of_query) if matched.
    fn detect_keyword(&self, query: &str) -> Option<(String, String)> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return None;
        }

        let keywords = self.registry.get_keywords();
        if keywords.is_empty() {
            return None;
        }

        let space_index = trimmed.find(' ');
        let potential_keyword = match space_index {
            Some(i) => &trimmed[..i],
            None => trimmed,
        };
        let rest_query = match space_index {
            Some(i) => trimmed[i + 1..].to_string(),
            None => String::new(),
        };

        if keywords.contains(&potential_keyword.to_string()) {
            Some((potential_keyword.to_string(), rest_query))
        } else {
            None
        }
    }
}

#[async_trait]
impl SearchProvider for PluginSearchProvider {
    fn source_id(&self) -> &str {
        "plugins"
    }

    fn source_label(&self) -> &str {
        "Plugins"
    }

    fn source_type(&self) -> SourceType {
        SourceType::Plugin
    }

    async fn search(&self, ctx: &SearchContext) -> Result<Vec<ProviderResult>, SearchError> {
        // Check if query matches a plugin keyword
        let (keyword, plugin_query) = match self.detect_keyword(&ctx.query) {
            Some(kq) => kq,
            None => return Ok(Vec::new()), // No keyword match → no plugin results
        };

        // Resolve keyword to plugin command entry
        let entry = match self.registry.resolve_keyword(&keyword) {
            Some(e) => e,
            None => return Ok(Vec::new()),
        };

        // Get the manifest to find the command definition
        let manifest = match self.registry.get_manifest(&entry.plugin_id) {
            Some(m) => m,
            None => return Ok(Vec::new()),
        };

        let command = match manifest.commands.iter().find(|c| c.name == entry.command_name) {
            Some(c) => c,
            None => return Ok(Vec::new()),
        };

        // Get plugin preferences
        let preferences = self.data_service.with_db(|conn| {
            self.registry.get_preferences(conn, &entry.plugin_id)
        }).unwrap_or_default();

        // Execute plugin command (synchronous — runs in blocking context)
        let response = self.executor.execute(
            &entry.plugin_dir,
            command,
            &plugin_query,
            preferences,
            &manifest.plugin.api_version,
        ).map_err(|e| SearchError::IndexError(format!("Plugin execution failed: {}", e)))?;

        // Map plugin results to ProviderResult
        Ok(response
            .items
            .into_iter()
            .map(|item| {
                let icon = item.icon.as_ref().and_then(|i| {
                    i.emoji.clone().or_else(|| i.url.clone())
                });

                let actions: Option<Vec<serde_json::Value>> = if item.actions.is_empty() {
                    None
                } else {
                    serde_json::to_value(&item.actions)
                        .ok()
                        .and_then(|v| v.as_array().cloned())
                };

                ProviderResult {
                    id: item.uid,
                    title: item.title,
                    subtitle: item.subtitle.unwrap_or_default(),
                    source_type: SourceType::Plugin,
                    source_id: "plugins".to_string(),
                    score: 1.0, // Plugin results get max relevance when keyword matches
                    frecency_score: 0.0,
                    icon,
                    url: None,
                    path: None,
                    favicon_url: None,
                    description: None,
                    extension: None,
                    size: None,
                    modified_at: None,
                    plugin_actions: actions,
                    plugin_badge: item.badge,
                    plugin_keyword: Some(keyword.clone()),
                }
            })
            .collect())
    }
}
