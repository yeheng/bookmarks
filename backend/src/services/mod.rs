pub mod auth_service;
pub mod bookmark_service;
pub mod collection_service;
pub mod indexer_service;
pub mod maintenance_service;
pub mod search_service;
pub mod stats_service;
pub mod tag_service;

pub use auth_service::*;
pub use bookmark_service::*;
pub use collection_service::*;
pub use indexer_service::*;
pub use maintenance_service::*;
pub use search_service::*;
pub use stats_service::*;
pub use tag_service::*;

#[cfg(test)]
mod collection_service_test;
