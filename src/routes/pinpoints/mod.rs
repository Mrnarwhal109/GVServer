pub mod delete;
pub mod get;
pub mod post;

pub use get::get_routing::handle_get_pinpoints;
pub use post::post_routing::handle_add_pinpoint;