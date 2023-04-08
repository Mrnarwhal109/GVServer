mod health_check;
mod subscriptions;
mod subscriptions_confirm;
mod newsletters;

// Export of handlers
pub use health_check::*;
pub use subscriptions::*;
pub use subscriptions_confirm::*;
pub use newsletters::*;