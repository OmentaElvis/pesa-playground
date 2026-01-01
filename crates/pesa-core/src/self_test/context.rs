use crate::{AppContext, db::Database};
use anyhow::{Context, anyhow};
use dashmap::DashMap;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tempfile::{TempDir, tempdir};

use super::{
    api_client::TestApiClient,
    emitter::MainUiEmitter,
    events::{EventWatcher, TestEventManager},
};

pub const SELF_TEST_PROGRESS_LOG: &str = "self_test_progress_log";

/// Specifies how the test database should be initialized.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum TestMode {
    /// Clones the main application database for testing.
    Clone,
    /// Creates a fresh, empty database and applies all migrations.
    Fresh,
}

/// The main context object for a self-test run.
///
/// This struct is passed to every `TestStep` and provides access to:
/// - A temporary, isolated `AppContext` for interacting with the application's core logic.
/// - A mechanism for logging progress back to the UI.
/// - A shared key-value store (`state`) for passing data between sequential tests.
#[derive(Clone)]
pub struct TestContext {
    /// A temporary application context for this test run.
    pub app_context: AppContext,
    /// A key-value store for sharing serializable data between test steps.
    pub state: HashMap<String, Value>,
    /// The name of the test currently being executed.
    pub current_test: Option<(String, usize)>,
    /// The manager for capturing events emitted by the application core.
    pub event_manager: Arc<TestEventManager>,
    main_ui_emitter: MainUiEmitter,
    callback_timeout: Duration,
    /// Link lifetime to TestContext
    _tmp_dir: Arc<TempDir>,
    /// API client for making HTTP requests during tests.
    pub api_client: TestApiClient,
}

impl TestContext {
    pub async fn new(
        mode: TestMode,
        main_ui_emitter: MainUiEmitter,
        callback_timeout: Duration,
        app_root: PathBuf,
    ) -> anyhow::Result<Self> {
        let temp_dir = tempdir().context("Failed to create temporary directory")?;

        let temp_path = temp_dir.path().to_path_buf();

        main_ui_emitter.log_runner(&format!(
            "Initializing test context: tempdir {}",
            temp_path.display()
        ));
        let db_path = Self::setup_test_db_path(&mode, &app_root, &temp_path)
            .await
            .context(format!("Failed to setup db path for mode: {:#?}", mode))?;

        main_ui_emitter.log_runner(&format!(
            "Initializing test context: Temp db setup complete {}",
            db_path.display()
        ));
        let settings_path = Self::setup_test_settings_path(&mode, &app_root, &temp_path)
            .await
            .context(format!(
                "Failed to setup test settings path for mode: {:#?}",
                mode
            ))?;

        let app_db = Database::new(&db_path).await?;

        main_ui_emitter.log_runner(
            "Initializing test context: Installing database, migrations and default value",
        );
        app_db.init().await?;

        let settings_manager = crate::settings::SettingsManager::new(settings_path)
            .await
            .context("Failed to initialize settings manager")?;

        let test_event_manager = Arc::new(TestEventManager::default());

        let app_context = AppContext {
            db: app_db.conn,
            settings: settings_manager,
            event_manager: test_event_manager.clone(), // Use the test manager for the app
            running: Arc::new(DashMap::new()),
            app_root: temp_path,
        };

        Ok(Self {
            app_context,
            state: HashMap::new(),
            event_manager: test_event_manager,
            main_ui_emitter,
            callback_timeout,
            current_test: None,
            _tmp_dir: Arc::new(temp_dir),
            api_client: TestApiClient::new(),
        })
    }

    /// Emits a log message to the UI, associated with the currently running test.
    pub async fn log(&self, message: &str) {
        if let Some((current, index)) = &self.current_test {
            self.main_ui_emitter.log_step(*index, current, message);
        } else {
            self.main_ui_emitter.log_runner(message);
        }
    }

    /// Registers a one-time watcher for a specific application event.
    ///
    /// This returns an `EventWatcher` future. The test can `.await` this future
    /// to wait for the application to emit the event. If the event is not
    /// emitted within the configured callback timeout, the future will
    /// resolve to an error.
    ///
    /// The generic `T` is the type that the test expects the event's payload
    /// to deserialize into.
    pub fn watch_event<T: DeserializeOwned + Unpin>(
        &self,
        event_name: &str,
    ) -> anyhow::Result<EventWatcher<T>> {
        Ok(self
            .event_manager
            .listen_for(event_name, self.callback_timeout))
    }

    /// Stores a serializable value in the shared state, making it available
    /// to subsequent test steps.
    ///
    /// # Example
    /// ```rust
    /// # use pesa_core::self_test::context::TestContext;
    /// # async fn example(context: &mut TestContext) -> anyhow::Result<()> {
    /// let user_id = "user-123".to_string();
    /// context.set("new_user_id", &user_id)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set<T: Serialize>(&mut self, key: &str, value: &T) -> anyhow::Result<()> {
        let serialized_value = serde_json::to_value(value).context("Failed to serialize value")?;
        self.state.insert(key.to_string(), serialized_value);
        Ok(())
    }

    /// Retrieves and deserializes a value from the shared state that was
    /// stored by a previous test step.
    ///
    /// # Example
    /// ```rust
    /// # use pesa_core::self_test::context::TestContext;
    /// # async fn example(context: &mut TestContext) -> anyhow::Result<()> {
    /// let user_id: Option<String> = context.get("new_user_id")?;
    /// if let Some(id) = user_id {
    ///     // ... use the id
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> anyhow::Result<Option<T>> {
        if let Some(value) = self.state.get(key) {
            serde_json::from_value(value.clone())
                .context("Failed to deserialize value from test state")
        } else {
            Ok(None)
        }
    }
    async fn setup_test_settings_path(
        mode: &TestMode,
        app_root: &Path,
        temp_dir: &Path,
    ) -> anyhow::Result<PathBuf> {
        let dest = temp_dir.join("settings.json");
        match mode {
            TestMode::Clone => {
                let settings = app_root.join("settings.json");
                if settings.exists() {
                    tokio::fs::copy(settings, &dest).await?;
                }
            }
            TestMode::Fresh => {}
        }
        Ok(dest.to_path_buf())
    }

    async fn setup_test_db_path(
        mode: &TestMode,
        app_root: &Path,
        temp_dir: &Path,
    ) -> anyhow::Result<PathBuf> {
        match mode {
            TestMode::Fresh => {
                let db_path = temp_dir.join("database.sqlite");
                Ok(db_path)
            }
            TestMode::Clone => {
                let main_db_path = app_root.join("database.sqlite");
                if !main_db_path.exists() {
                    return Err(anyhow!(
                        "Main database not found at {:?}. Cannot clone.",
                        main_db_path
                    ));
                }

                let cloned_db_path = temp_dir.join("database.sqlite");

                // Copy the main database file
                tokio::fs::copy(&main_db_path, &cloned_db_path)
                    .await
                    .context("Failed to clone main database")?;

                // Copy WAL file if it exists
                let wal_path = app_root.join("database.sqlite-wal");
                if wal_path.exists() {
                    tokio::fs::copy(&wal_path, temp_dir.join("database.sqlite-wal"))
                        .await
                        .context("Failed to clone main database WAL file")?;
                }

                // Copy SHM file if it exists
                let shm_path = app_root.join("database.sqlite-shm");
                if shm_path.exists() {
                    tokio::fs::copy(&shm_path, temp_dir.join("database.sqlite-shm"))
                        .await
                        .context("Failed to clone main database SHM file")?;
                }

                Ok(cloned_db_path)
            }
        }
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        let keys: Vec<u32> = self
            .app_context
            .running
            .iter()
            .map(|entry| *entry.key())
            .collect();

        for id in keys {
            if let Some((_, sandbox)) = self.app_context.running.remove(&id) {
                let _ = sandbox.shutdown.send(());
                sandbox.handle.abort();
            }
        }
    }
}

// Re-expose TestContext
pub use TestContext as TestFrameworkContext;
