mod health_check;
pub mod login;
pub mod pinpoints;
pub mod users;

// Export of handlers
pub use health_check::*;
pub use users::post::handle_signup;
pub use login::*;
pub use pinpoints::*;