//! Plugin search provider — wraps PluginExecutor for plugin-based search.
//!
//! Detects plugin keywords server-side and delegates to the plugin executor.
//! This provider handles queries of the form "keyword rest-of-query"
//! and routes them to the matching plugin command.

use async_trait::async_trait;
use std::sync::Arc;

use crate::plugins::executor::PluginExecutor;
use crate::plugins::manifest::PluginCommand;
use crate::services::data_service::DataService;
use super::engine::SearchError;
use super::provider::{ProviderResult, SearchContext, SearchProvider, SourceType};

/// Provides search results for a specific plugin command.
///
/// Acts as a proxy that binds a specific "keyword" to a plugin command execution.
pub struct PluginSearchProvider {
    plugin_name: String,
    plugin_dir: std::path::PathBuf,
    command: PluginCommand,
    executor: Arc<PluginExecutor>,
    data_service: Arc<DataService>,
    /// Pre-computed source ID: "plugin:<keyword>"
    source_id: String,
}

impl PluginSearchProvider {
    pub fn new(
        plugin_name: String,
        plugin_dir: std::path::PathBuf,
        command: PluginCommand,
        executor: Arc<PluginExecutor>,
        data_service: Arc<DataService>,
    ) -> Self {
        let source_id = format!("plugin:{}", command.keyword);
        Self {
            plugin_name,
            plugin_dir,
            command,
            executor,
            data_service,
            source_id,
        }
    }
}

#[async_trait]
impl SearchProvider for PluginSearchProvider {
    fn source_id(&self) -> &str {
        &self.source_id
    }

    fn source_label(&self) -> &str {
        &self.command.title
    }

    fn source_type(&self) -> SourceType {
        SourceType::Plugin
    }

    async fn search(&self, ctx: &SearchContext) -> Result<Vec<ProviderResult>, SearchError> {
        let keyword = &self.command.keyword;
        let query = &ctx.query;

        let plugin_query = if let Some(scope) = &ctx.structured_query.scope {
            if scope == keyword {
                ctx.structured_query.terms.join(" ")
            } else {
                return Ok(Vec::new());
            }
        } else {
            let trimmed = query.trim();
            if trimmed == keyword || trimmed.starts_with(&format!("{} ", keyword)) {
                if trimmed.len() > keyword.len() {
                    trimmed[keyword.len()..].trim().to_string()
                } else {
                    "".to_string()
                }
            } else {
                return Ok(Vec::new());
            }
        };

        // Get plugin preferences from the JSON store
        let preferences = self.data_service
            .with_plugin_store(|store| {
                Ok(store.get_preferences(&self.plugin_name))
            })
            .unwrap_or_default();

        // Execute plugin command
        let response = self.executor.execute(
            &self.plugin_dir,
            &self.command,
            &plugin_query,
            ctx.structured_query.filters.clone(),
            preferences,
            &crate::plugins::manifest::HOST_API_VERSION,
        ).map_err(|e| SearchError::IndexError(format!("Plugin execution failed: {}", e)))?;

        // Map plugin results
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
                    source_id: format!("plugin:{}", keyword),
                    score: 1.0,
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
