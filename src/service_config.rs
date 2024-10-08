//! Command line arguments / Application configuration definition
//!
//! This is either set at startup via the command line or explicitly while running the tests
//!
//! For a real world application you would likely want to have the ability to also load
//! (parts of this) from environment variables or a configuration file
use std::net::IpAddr;
use std::path::PathBuf;

#[derive(clap::Parser, Clone, Debug)]
pub struct Config {
    /// Port the application is running on
    #[clap(default_value = "8000")]
    pub port: u16,
    /// Address the application is listing on
    #[clap(default_value = "0.0.0.0")]
    pub address: IpAddr,
    /// Path where the database should be stored
    #[clap(default_value = "race_time.db")]
    pub database_url: String,
    /// Whether or not test data should be inserted into the database
    #[clap(long = "insert-test-data")]
    pub insert_test_data: bool,
    /// Base url the application is hosted at
    #[clap(long = "base_url", default_value = "")]
    pub base_url: String,
    /// Path to the template directory
    #[clap(default_value = "templates")]
    pub template_dir: PathBuf,
    /// Internal flag whether or on this config is a test run config
    ///
    /// This cannot be set from the command line
    #[clap(skip)]
    pub is_test: bool,
}
