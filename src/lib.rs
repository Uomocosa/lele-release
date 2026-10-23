#[path = "__basic__/mod.rs"]
pub mod __basic__;

pub mod error;
pub use error::Error;

pub mod fleet;
pub use fleet::Fleet;

pub mod git;
pub use git::Git;

#[path = "../methods/mod.rs"]
pub mod methods;

pub mod repo;
pub use repo::Repo;
