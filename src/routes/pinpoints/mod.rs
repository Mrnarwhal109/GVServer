mod get;
mod post;
mod delete;

pub use get::handle_get_pinpoints;
pub use post::handle_add_pinpoint;
pub use delete::{handle_delete_all_pinpoints, handle_delete_all_user_pinpoints};