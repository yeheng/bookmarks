pub mod resource;  // 重命名 bookmark → resource
pub mod collection;
pub mod search;
pub mod stats;
pub mod tag;
pub mod user;

pub use resource::*;  // 重命名 bookmark → resource
pub use collection::*;
pub use search::*;
pub use stats::*;
pub use tag::*;
pub use user::*;
