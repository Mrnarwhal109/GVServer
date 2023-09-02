mod health_check;
pub mod signup;
pub mod login;
pub mod pinpoints;

// Export of handlers
pub use health_check::*;
pub use signup::*;
pub use login::*;
pub use pinpoints::*;