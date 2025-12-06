pub mod collection;
pub mod resource; // 重命名 bookmark → resource
pub mod search;
pub mod stats;
pub mod tag;
pub mod user;

pub use collection::*;
pub use resource::*; // 重命名 bookmark → resource
pub use search::*;
pub use stats::*;
pub use tag::*;
pub use user::*;
