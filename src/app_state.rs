use crate::axum_ext::AcceptLanguage;
use crate::database::diesel_ext;
use crate::errors::Result;
use crate::service_config::Config;
use axum::response::Html;
use axum_extra::TypedHeader;
use deadpool_diesel::sqlite::{Hook, HookError};
use deadpool_sync::SyncWrapper;
use diesel::connection::InstrumentationEvent;
use diesel::{Connection, QueryResult, RunQueryDsl, SqliteConnection};
use fluent_templates::Loader;
use minijinja::value::ViaDeserialize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use time::format_description;
use time::macros::format_description;

// Localization data loaded at compile time
fluent_templates::static_loader! {
    static LOCALES = {
        locales: "locales",
        fallback_language: "en-US",
    };
}

/// Application state
#[derive(Clone)]
pub struct State {
    /// database connection pool
    pub pool: deadpool_diesel::sqlite::Pool,
    /// template context used for rendering HTML pages
    pub templates: minijinja::Environment<'static>,
    /// base url path the application is served at
    pub base_url: Arc<str>,
}

impl State {
    /// Setup the application state
    pub fn from_config(config: &Config) -> Self {
        let is_test = config.is_test;
        let manager_config = deadpool_diesel::ManagerConfig {
            recycling_method: deadpool_diesel::RecyclingMethod::Fast,
        };
        let manager = deadpool_diesel::Manager::from_config(
            &config.database_url,
            deadpool_diesel::Runtime::Tokio1,
            manager_config,
        );
        let mut templates = minijinja::Environment::new();
        templates.set_loader(minijinja::path_loader(&config.template_dir));
        templates.add_filter("format_date", format_date);
        templates.add_filter("format_timestamp", format_timestamp);
        templates.add_function("translate", translate);
        let mut builder = deadpool_diesel::Pool::builder(manager);
        if is_test {
            // for tests set the poolsize to 1
            builder = builder.max_size(1);
        }

        let pool = builder
            .post_create(Hook::async_fn(move |conn, _metrics| {
                Box::pin(custom_connection_setup(conn, is_test))
            }))
            .build()
            .expect("Could not build the connection pool");
        Self {
            pool,
            templates,
            base_url: config.base_url.clone().into(),
        }
    }

    pub async fn with_connection<T: Send + 'static>(
        &self,
        callback: impl FnOnce(&mut SqliteConnection) -> QueryResult<T> + Send + 'static,
    ) -> Result<T> {
        Ok(self.pool.get().await?.interact(callback).await??)
    }
}

/// apply various custom settings to each database connection
///
/// 1. Call `Connection::begin_test_transaction` if the is_test flag is set
/// 2. Register custom SQL functions
/// 3. Setup instrumentation for logging
/// 4. Setup various configs to make SQLite a suitable solution for hosting a web application
async fn custom_connection_setup(
    conn: &mut SyncWrapper<SqliteConnection>,
    is_test: bool,
) -> Result<(), HookError> {
    let _ = conn
        .interact(move |conn| {
            // setup test configurations
            if is_test {
                // not required as we use `:memory:` databases for testing
                // otherwise this would be what you want
                //conn.begin_test_transaction()?;
            }
            // register custom SQL functions
            diesel_ext::register_functions(conn)?;
            // setup instrumentation to log every query
            conn.set_instrumentation(|event: InstrumentationEvent<'_>| {
                // This is a really simple setup that just logs every query
                // A real world implementation might also want to record more events
                // and some timings here
                if let InstrumentationEvent::StartQuery { query, .. } = event {
                    tracing::debug!(?query)
                }
            });
            // see https://fractaledmind.github.io/2023/09/07/enhancing-rails-sqlite-fine-tuning/
            // sleep if the database is busy
            // this corresponds to 2 seconds
            // if we ever see errors regarding busy_timeout in production
            // we might want to consider to increase this time
            diesel::sql_query("PRAGMA busy_timeout = 2000;").execute(conn)?;
            // better write-concurrency
            diesel::sql_query("PRAGMA journal_mode = WAL;").execute(conn)?;
            // fsync only in critical moments
            diesel::sql_query("PRAGMA synchronous = NORMAL;").execute(conn)?;
            // write WAL changes back every 1000 pages, for an in average 1MB WAL file. May affect readers if number is increased
            diesel::sql_query("PRAGMA wal_autocheckpoint = 1000;").execute(conn)?;
            // free some space by truncating possibly massive WAL files from the last run
            diesel::sql_query("PRAGMA wal_checkpoint(TRUNCATE);").execute(conn)?;
            // maximum size of the WAL file, corresponds to 64MB
            diesel::sql_query("PRAGMA journal_size_limit = 67108864;").execute(conn)?;
            // maximum size of the internal mmap pool. Corresponds to 128MB, matches postgres default settings
            diesel::sql_query("PRAGMA mmap_size = 134217728;").execute(conn)?;
            // maximum number of database disk pages that will be hold in memory. Corresponds to ~8MB
            diesel::sql_query("PRAGMA cache_size = 2000;").execute(conn)?;
            //enforce foreign keys
            diesel::sql_query("PRAGMA foreign_keys = ON;").execute(conn)?;
            QueryResult::Ok(())
        })
        .await;
    Ok(())
}

fn format_date(arg: ViaDeserialize<time::PrimitiveDateTime>) -> String {
    arg.0
        .format(
            &format_description::parse("[hour]:[minute]:[second]").expect("static correct format"),
        )
        .expect("Can format this timestamp")
}

fn format_timestamp(arg: ViaDeserialize<time::PrimitiveDateTime>) -> String {
    let format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
    arg.0.format(&format).expect("Can format this timestamp")
}

fn translate<'a>(state: &minijinja::State<'_, 'a>, key: &'a str) -> String {
    let lang_keys = state
        .lookup("lang_keys")
        .expect("This is set by the render template method above");
    let lang_keys = AcceptLanguage::deserialize(lang_keys)
        .expect("This is set by the render template method above");
    lookup_translation(&lang_keys, key, HashMap::new())
}

fn lookup_translation(lang_keys: &AcceptLanguage, key: &str, args: HashMap<&str, &str>) -> String {
    let args = args
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v.into()))
        .collect::<HashMap<_, _>>();

    let ret = lang_keys
        .as_lang_ident_iter()
        .find_map(|l| LOCALES.lookup_no_default_fallback::<String>(&l, key, Some(&args)))
        .unwrap_or_else(|| LOCALES.lookup_with_args(LOCALES.fallback(), key, &args));
    ret
}

/// A wrapper for our State, that also includes the relevant AcceptLanguage header
/// from the request. This enables automatic translation of the rendered HTML
pub struct AppState {
    state: State,
    lang_keys: AcceptLanguage,
}

#[async_trait::async_trait]
impl axum::extract::FromRequestParts<State> for AppState {
    type Rejection =
        <Option<TypedHeader<AcceptLanguage>> as axum::extract::FromRequestParts<State>>::Rejection;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &State,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(lang_keys) = <Option<TypedHeader<AcceptLanguage>> as axum::extract::FromRequestParts<_>>::from_request_parts(parts, state).await?.unwrap_or_else(|| TypedHeader(AcceptLanguage::default()));
        Ok(Self {
            state: state.clone(),
            lang_keys,
        })
    }
}

#[derive(Serialize)]
struct TemplateData<'a, T> {
    base_url: &'a str,
    lang_keys: &'a AcceptLanguage,
    #[serde(flatten)]
    inner: T,
}

impl AppState {
    /// render a template with a given name and the given data as HTML
    pub fn render_template(
        &self,
        name: &'static str,
        data: impl Serialize,
    ) -> Result<Html<String>> {
        let template = self.state.templates.get_template(name)?;
        let base_url = &self.state.base_url;
        Ok(Html(template.render(TemplateData {
            base_url,
            lang_keys: &self.lang_keys,
            inner: data,
        })?))
    }

    /// Interact with a database connection
    pub async fn with_connection<T: Send + 'static>(
        &self,
        callback: impl FnOnce(&mut SqliteConnection) -> QueryResult<T> + Send + 'static,
    ) -> Result<T> {
        Ok(self.state.pool.get().await?.interact(callback).await??)
    }

    pub fn base_url(&self) -> &str {
        &self.state.base_url
    }

    pub fn translation(&self, key: &str) -> String {
        lookup_translation(&self.lang_keys, key, HashMap::new())
    }

    pub fn translation_with_params(&self, key: &str, params: HashMap<&str, &str>) -> String {
        lookup_translation(&self.lang_keys, key, params)
    }
}
