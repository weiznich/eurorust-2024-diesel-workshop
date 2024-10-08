pub mod diesel_ext;
pub mod schema;
pub mod shared_models;
pub mod test_data;

/// The id type of the application
pub type Id = diesel_ext::Uuid;
