use anyhow::{Context, Result, anyhow, bail};
use appwrite::{
    Client, InputFile,
    enums::{ExecutionMethod, Runtime},
    id::ID,
    permission::Permission,
    query::Query,
    role::Role,
    services::{Functions, Health, Storage, TablesDB, Users},
};
use console::{Emoji, Style, style};
use dialoguer::MultiSelect;
use indicatif::{ProgressBar, ProgressStyle};
use serde::Serialize;
use serde_json::json;
use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
    time::Duration as StdDuration,
};
use tokio::time::{Duration, sleep};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Demo {
    Health,
    TablesDB,
    Storage,
    Users,
    Functions,
}

impl Demo {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "health" => Some(Self::Health),
            "tablesdb" => Some(Self::TablesDB),
            "storage" => Some(Self::Storage),
            "users" => Some(Self::Users),
            "functions" => Some(Self::Functions),
            _ => None,
        }
    }

    fn all() -> Vec<Self> {
        vec![
            Self::Health,
            Self::TablesDB,
            Self::Storage,
            Self::Users,
            Self::Functions,
        ]
    }

    fn label(self) -> &'static str {
        match self {
            Self::Health => "Health",
            Self::TablesDB => "TablesDB",
            Self::Storage => "Storage",
            Self::Users => "Users",
            Self::Functions => "Functions",
        }
    }

    fn emoji(self) -> Emoji<'static, 'static> {
        match self {
            Self::Health => Emoji("💚 ", ""),
            Self::TablesDB => Emoji("🗃️  ", ""),
            Self::Storage => Emoji("📦 ", ""),
            Self::Users => Emoji("👤 ", ""),
            Self::Functions => Emoji("⚡ ", ""),
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Health => "Check server, database, storage & time endpoints",
            Self::TablesDB => "Full CRUD on databases, tables, columns, indexes & rows",
            Self::Storage => "Bucket management, file upload/download/rename",
            Self::Users => "Create, update, manage preferences & delete users",
            Self::Functions => "Deploy a Node.js function, set variables, run sync & async executions",
        }
    }
}

impl std::fmt::Display for Demo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{} — {}", self.emoji(), self.label(), self.description())
    }
}

struct Config {
    endpoint: String,
    project_id: String,
    api_key: String,
    self_signed: bool,
    sample_file: PathBuf,
    function_source_dir: PathBuf,
}

impl Config {
    fn from_env() -> Result<Self> {
        let mut missing = Vec::new();

        let endpoint = read_required_env("APPWRITE_ENDPOINT", &mut missing);
        let project_id = read_required_env("APPWRITE_PROJECT_ID", &mut missing);
        let api_key = read_required_env("APPWRITE_API_KEY", &mut missing);

        if !missing.is_empty() {
            bail!(
                "Missing required environment variables: {}\n\n{}",
                missing.join(", "),
                config_help()
            );
        }

        Ok(Self {
            endpoint: endpoint.expect("validated APPWRITE_ENDPOINT"),
            project_id: project_id.expect("validated APPWRITE_PROJECT_ID"),
            api_key: api_key.expect("validated APPWRITE_API_KEY"),
            self_signed: read_bool_env("APPWRITE_SELF_SIGNED"),
            sample_file: read_path_env(
                "APPWRITE_SAMPLE_FILE",
                manifest_path(["resources", "sample-upload.txt"]),
            ),
            function_source_dir: read_path_env(
                "APPWRITE_FUNCTION_SOURCE_DIR",
                manifest_path(["resources", "functions", "hello-node"]),
            ),
        })
    }
}

struct Playground {
    client: Client,
    config: Config,
}

impl Playground {
    fn new(config: Config) -> Self {
        let client = Client::new()
            .set_endpoint(config.endpoint.clone())
            .set_project(config.project_id.clone())
            .set_key(config.api_key.clone())
            .set_self_signed(config.self_signed);

        Self { client, config }
    }

    async fn run(&self, demos: &[Demo]) -> Result<()> {
        let total = demos.len();
        for (i, demo) in demos.iter().enumerate() {
            heading(&format!(
                "{}{} ({}/{})",
                demo.emoji(),
                demo.label(),
                i + 1,
                total
            ));

            let result = match demo {
                Demo::Health => self.run_health_demo().await,
                Demo::TablesDB => self.run_tablesdb_demo().await,
                Demo::Storage => self.run_storage_demo().await,
                Demo::Users => self.run_users_demo().await,
                Demo::Functions => self.run_functions_demo().await,
            };

            match &result {
                Ok(()) => {
                    println!(
                        "  {} {}",
                        style("✔").green().bold(),
                        style(format!("{} demo completed", demo.label())).green()
                    );
                }
                Err(err) => {
                    println!(
                        "  {} {} — {}",
                        style("✘").red().bold(),
                        style(format!("{} demo failed", demo.label())).red(),
                        err,
                    );
                }
            }

            result?;
        }

        println!();
        println!(
            "{} {}",
            Emoji("🎉 ", ""),
            style("All demos completed successfully!").green().bold()
        );
        Ok(())
    }

    async fn run_health_demo(&self) -> Result<()> {
        let health = Health::new(&self.client);

        let overall = health.get().await.context("health.get() failed")?;
        print_json("GET /health", &overall)?;

        let db = health.get_db().await.context("health.get_db() failed")?;
        print_json("GET /health/db", &db)?;

        let storage = health
            .get_storage()
            .await
            .context("health.get_storage() failed")?;
        print_json("GET /health/storage", &storage)?;

        let time = health
            .get_time()
            .await
            .context("health.get_time() failed")?;
        print_json("GET /health/time", &time)?;

        Ok(())
    }

    async fn run_tablesdb_demo(&self) -> Result<()> {
        let tablesdb = TablesDB::new(&self.client);
        let mut database_id: Option<String> = None;
        let mut table_id: Option<String> = None;
        let mut row_id: Option<String> = None;

        let result: Result<()> = async {
            // --- Database CRUD ---
            let created_database = tablesdb
                .create(ID::unique(), "Rust Playground Database", Some(true))
                .await
                .context("tablesdb.create() failed")?;
            database_id = Some(extract_id(&created_database)?);
            print_json("Created database", &created_database)?;

            let database_id_ref = required_id(&database_id, "database")?;

            let listed_databases = tablesdb
                .list(None, None, Some(true))
                .await
                .context("tablesdb.list() failed")?;
            print_json("Listed databases", &listed_databases)?;

            let fetched_database = tablesdb
                .get(database_id_ref)
                .await
                .context("tablesdb.get() failed")?;
            print_json("Fetched database", &fetched_database)?;

            let updated_database = tablesdb
                .update(
                    database_id_ref,
                    Some("Rust Playground Database Updated"),
                    Some(true),
                )
                .await
                .context("tablesdb.update() failed")?;
            print_json("Updated database", &updated_database)?;

            // --- Table CRUD ---
            let created_table = tablesdb
                .create_table(
                    database_id_ref,
                    ID::unique(),
                    "Movies",
                    Some(default_table_permissions()),
                    Some(true),
                    Some(true),
                    None,
                    None,
                )
                .await
                .context("tablesdb.create_table() failed")?;
            table_id = Some(extract_id(&created_table)?);
            print_json("Created table", &created_table)?;

            let table_id_ref = required_id(&table_id, "table")?;

            let listed_tables = tablesdb
                .list_tables(database_id_ref, None, None, Some(true))
                .await
                .context("tablesdb.list_tables() failed")?;
            print_json("Listed tables", &listed_tables)?;

            let fetched_table = tablesdb
                .get_table(database_id_ref, table_id_ref)
                .await
                .context("tablesdb.get_table() failed")?;
            print_json("Fetched table", &fetched_table)?;

            let updated_table = tablesdb
                .update_table(
                    database_id_ref,
                    table_id_ref,
                    Some("Movies Updated"),
                    Some(default_table_permissions()),
                    Some(true),
                    Some(true),
                )
                .await
                .context("tablesdb.update_table() failed")?;
            print_json("Updated table", &updated_table)?;

            // --- Columns ---
            let title_column = tablesdb
                .create_string_column(
                    database_id_ref,
                    table_id_ref,
                    "title",
                    255,
                    true,
                    None,
                    Some(false),
                    Some(false),
                )
                .await
                .context("tablesdb.create_string_column() failed")?;
            print_json("Created string column", &title_column)?;

            let year_column = tablesdb
                .create_integer_column(
                    database_id_ref,
                    table_id_ref,
                    "release_year",
                    false,
                    Some(1900),
                    Some(2100),
                    Some(1999),
                    Some(false),
                )
                .await
                .context("tablesdb.create_integer_column() failed")?;
            print_json("Created integer column", &year_column)?;

            spin_wait("Waiting for columns to become available...", Duration::from_secs(2)).await;

            // TODO(sdk-fix): ColumnList model defines `columns: Vec<String>` but the
            // API returns full column objects (maps with key, type, status, etc.).
            //   Fix: In models/column_list.rs, change the field type from
            //        `pub columns: Vec<String>` to `pub columns: Vec<serde_json::Value>`
            //        (or better, a `Column` enum that can deserialize any column type).
            //   Same bug exists in models/attribute_list.rs (`attributes: Vec<String>`).
            let listed_columns: serde_json::Value = tablesdb
                .client()
                .call(
                    reqwest::Method::GET,
                    &format!(
                        "/tablesdb/{}/tables/{}/columns",
                        database_id_ref, table_id_ref
                    ),
                    None,
                    Some(HashMap::from([
                        ("total".to_string(), json!(true)),
                    ])),
                )
                .await
                .context("tablesdb.list_columns() failed")?;
            print_json("Listed columns", &listed_columns)?;

            // --- Indexes ---
            // TODO(sdk-fix): `create_index` in services/tables_db.rs (and services/databases.rs)
            // types `orders` as `Option<OrderBy>` (single enum) and serializes it with
            // `json!(value)` → `"asc"`. The API expects an array: `["asc"]`.
            //   Fix: Change the parameter type from `Option<crate::enums::OrderBy>` to
            //        `Option<Vec<crate::enums::OrderBy>>` (or `Option<Vec<String>>`)
            //        so it serializes as `json!(["asc"])` matching the API spec.
            let created_index: serde_json::Value = tablesdb
                .client()
                .call(
                    reqwest::Method::POST,
                    &format!(
                        "/tablesdb/{}/tables/{}/indexes",
                        database_id_ref, table_id_ref
                    ),
                    Some(HashMap::from([(
                        "content-type".to_string(),
                        "application/json".to_string(),
                    )])),
                    Some(HashMap::from([
                        ("key".to_string(), json!("idx_release_year")),
                        ("type".to_string(), json!("key")),
                        ("columns".to_string(), json!(["release_year"])),
                        ("orders".to_string(), json!(["asc"])),
                    ])),
                )
                .await
                .context("tablesdb.create_index() failed")?;
            print_json("Created index", &created_index)?;

            spin_wait("Waiting for index to become available...", Duration::from_secs(2)).await;

            let listed_indexes = tablesdb
                .list_indexes(database_id_ref, table_id_ref, None, Some(true))
                .await
                .context("tablesdb.list_indexes() failed")?;
            print_json("Listed indexes", &listed_indexes)?;

            // --- Rows ---
            let created_row = tablesdb
                .create_row(
                    database_id_ref,
                    table_id_ref,
                    ID::unique(),
                    json!({
                        "title": "Inception",
                        "release_year": 2010
                    }),
                    Some(default_row_permissions()),
                    None,
                )
                .await
                .context("tablesdb.create_row() failed")?;
            row_id = Some(extract_id(&created_row)?);
            print_json("Created row", &created_row)?;

            let listed_rows = tablesdb
                .list_rows(
                    database_id_ref,
                    table_id_ref,
                    Some(vec![Query::equal("release_year", 2010).to_string()]),
                    None,
                    Some(true),
                    None,
                )
                .await
                .context("tablesdb.list_rows() failed")?;
            print_json("Listed rows", &listed_rows)?;

            let row_id_ref = required_id(&row_id, "row")?;

            let fetched_row = tablesdb
                .get_row(
                    database_id_ref,
                    table_id_ref,
                    row_id_ref,
                    None,
                    None,
                )
                .await
                .context("tablesdb.get_row() failed")?;
            print_json("Fetched row", &fetched_row)?;

            let updated_row = tablesdb
                .update_row(
                    database_id_ref,
                    table_id_ref,
                    row_id_ref,
                    Some(json!({ "release_year": 2014 })),
                    None,
                    None,
                )
                .await
                .context("tablesdb.update_row() failed")?;
            print_json("Updated row", &updated_row)?;

            Ok(())
        }
        .await;

        cleanup_tablesdb_resources(&tablesdb, &database_id, &table_id, &row_id).await;

        result
    }

    async fn run_storage_demo(&self) -> Result<()> {
        ensure_path_exists(&self.config.sample_file, "sample upload file")?;

        let storage = Storage::new(&self.client);
        let mut bucket_id: Option<String> = None;
        let mut file_id: Option<String> = None;

        let result: Result<()> = async {
            let created_bucket = storage
                .create_bucket(
                    ID::unique(),
                    "Rust Playground Bucket",
                    Some(default_bucket_permissions()),
                    Some(true),
                    Some(true),
                    None,
                    None,
                    None,
                    Some(true),
                    Some(true),
                    Some(true),
                )
                .await
                .context("storage.create_bucket() failed")?;
            bucket_id = Some(extract_id(&created_bucket)?);
            print_json("Created bucket", &created_bucket)?;

            let bucket_id_ref = required_id(&bucket_id, "bucket")?;

            let listed_buckets = storage
                .list_buckets(None, None, Some(true))
                .await
                .context("storage.list_buckets() failed")?;
            print_json("Listed buckets", &listed_buckets)?;

            let fetched_bucket = storage
                .get_bucket(bucket_id_ref)
                .await
                .context("storage.get_bucket() failed")?;
            print_json("Fetched bucket", &fetched_bucket)?;

            let updated_bucket = storage
                .update_bucket(
                    bucket_id_ref,
                    "Rust Playground Bucket Updated",
                    Some(default_bucket_permissions()),
                    Some(true),
                    Some(true),
                    None,
                    None,
                    None,
                    Some(true),
                    Some(true),
                    Some(true),
                )
                .await
                .context("storage.update_bucket() failed")?;
            print_json("Updated bucket", &updated_bucket)?;

            let upload = InputFile::from_path(&self.config.sample_file, None)
                .await
                .with_context(|| {
                    format!(
                        "failed to prepare upload from {}",
                        self.config.sample_file.display()
                    )
                })?;

            // TODO(sdk-fix): `Storage::create_file` in services/storage.rs passes
            // `upload_id: Some(file_id_str)` to `client.file_upload`. In client.rs
            // line ~622, when `upload_id` is Some, the URL becomes
            // `{endpoint}{path}/{id}` → POST /storage/buckets/{b}/files/{fileId}
            // which is a non-existent route (create is POST /storage/buckets/{b}/files).
            //   Fix: In `Storage::create_file`, pass `None` for the upload_id param
            //        (it's only needed for resumable/chunked uploads, not initial create).
            //        Or change `file_upload` to not append the id for the initial request.
            let file_id_str: String = ID::unique().into();
            let mut file_params = HashMap::new();
            file_params.insert("fileId".to_string(), json!(file_id_str));
            file_params.insert(
                "permissions".to_string(),
                json!(default_file_permissions()),
            );
            let created_file: serde_json::Value = storage
                .client()
                .file_upload(
                    &format!("/storage/buckets/{}/files", bucket_id_ref),
                    Some(HashMap::from([(
                        "content-type".to_string(),
                        "multipart/form-data".to_string(),
                    )])),
                    file_params,
                    "file",
                    upload,
                    None,
                )
                .await
                .context("storage.create_file() failed")?;
            file_id = Some(extract_id(&created_file)?);
            print_json("Uploaded file", &created_file)?;

            let listed_files = storage
                .list_files(bucket_id_ref, None, None, Some(true))
                .await
                .context("storage.list_files() failed")?;
            print_json("Listed files", &listed_files)?;

            let file_id_ref = required_id(&file_id, "file")?;

            let fetched_file = storage
                .get_file(bucket_id_ref, file_id_ref)
                .await
                .context("storage.get_file() failed")?;
            print_json("Fetched file", &fetched_file)?;

            let updated_file = storage
                .update_file(
                    bucket_id_ref,
                    file_id_ref,
                    Some("sample-upload-renamed.txt"),
                    Some(default_file_permissions()),
                )
                .await
                .context("storage.update_file() failed")?;
            print_json("Updated file", &updated_file)?;

            let downloaded = storage
                .get_file_download(bucket_id_ref, file_id_ref, None)
                .await
                .context("storage.get_file_download() failed")?;
            println!(
                "\n  {} {} {} bytes",
                style("→").cyan().bold(),
                style("Downloaded file:").bold(),
                style(downloaded.len()).yellow()
            );

            Ok(())
        }
        .await;

        cleanup_storage_resources(&storage, &bucket_id, &file_id).await;

        result
    }

    async fn run_users_demo(&self) -> Result<()> {
        let users = Users::new(&self.client);
        let mut user_id: Option<String> = None;

        let result: Result<()> = async {
            let email = format!("{}@example.com", ID::unique());

            let created_user = users
                .create(
                    ID::unique(),
                    Some(&email),
                    None,
                    Some("user@123456"),
                    Some("Rust Playground User"),
                )
                .await
                .context("users.create() failed")?;
            user_id = Some(extract_id(&created_user)?);
            print_json("Created user", &created_user)?;

            let user_id_ref = required_id(&user_id, "user")?;

            let listed_users = users
                .list(None, None, Some(true))
                .await
                .context("users.list() failed")?;
            print_json("Listed users", &listed_users)?;

            let fetched_user = users.get(user_id_ref).await.context("users.get() failed")?;
            print_json("Fetched user", &fetched_user)?;

            let updated_name = users
                .update_name(user_id_ref, "Rust Playground User Updated")
                .await
                .context("users.update_name() failed")?;
            print_json("Updated user name", &updated_name)?;

            let prefs_before = users
                .get_prefs(user_id_ref)
                .await
                .context("users.get_prefs() failed")?;
            print_json("Fetched user prefs", &prefs_before)?;

            let prefs_after = users
                .update_prefs(
                    user_id_ref,
                    json!({
                        "theme": "dark",
                        "language": "en",
                        "source": "appwrite-rust-playground"
                    }),
                )
                .await
                .context("users.update_prefs() failed")?;
            print_json("Updated user prefs", &prefs_after)?;

            Ok(())
        }
        .await;

        cleanup_user_resources(&users, &user_id).await;

        result
    }

    async fn run_functions_demo(&self) -> Result<()> {
        ensure_path_exists(
            &self.config.function_source_dir,
            "function source directory",
        )?;

        let functions = Functions::new(&self.client);
        let mut function_id: Option<String> = None;
        let mut variable_id: Option<String> = None;
        let mut archive_path: Option<PathBuf> = None;

        let result: Result<()> = async {
            let created_function = functions
                .create(
                    ID::unique(),
                    "Rust Playground Function",
                    Runtime::Node22,
                    Some(vec![Role::any().to_string()]),
                    None,
                    None,
                    Some(15),
                    Some(true),
                    Some(true),
                    Some("index.js"),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .await
                .context("functions.create() failed")?;
            function_id = Some(extract_id(&created_function)?);
            print_json("Created function", &created_function)?;

            let function_id_ref = required_id(&function_id, "function")?;

            let listed_functions = functions
                .list(None, None, Some(true))
                .await
                .context("functions.list() failed")?;
            print_json("Listed functions", &listed_functions)?;

            let fetched_function = functions
                .get(function_id_ref)
                .await
                .context("functions.get() failed")?;
            print_json("Fetched function", &fetched_function)?;

            let updated_function = functions
                .update(
                    function_id_ref,
                    "Rust Playground Function Updated",
                    Some(Runtime::Node22),
                    Some(vec![Role::any().to_string()]),
                    None,
                    None,
                    Some(30),
                    Some(true),
                    Some(true),
                    Some("index.js"),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .await
                .context("functions.update() failed")?;
            print_json("Updated function", &updated_function)?;

            archive_path = Some(package_function_source(&self.config.function_source_dir)?);
            let archive_ref = archive_path
                .as_ref()
                .ok_or_else(|| anyhow!("function archive was not created"))?;
            let code = InputFile::from_path(archive_ref, Some("application/gzip"))
                .await
                .with_context(|| format!("failed to open {}", archive_ref.display()))?;

            let created_deployment = functions
                .create_deployment(function_id_ref, code, true, Some("index.js"), None)
                .await
                .context("functions.create_deployment() failed")?;
            print_json("Created deployment", &created_deployment)?;

            let deployment_id = extract_id(&created_deployment)?;
            wait_for_deployment_ready(&functions, function_id_ref, &deployment_id).await?;

            let listed_deployments = functions
                .list_deployments(function_id_ref, None, None, Some(true))
                .await
                .context("functions.list_deployments() failed")?;
            print_json("Listed deployments", &listed_deployments)?;

            let created_variable = functions
                .create_variable(function_id_ref, "PLAYGROUND_SOURCE", "rust", Some(false))
                .await
                .context("functions.create_variable() failed")?;
            variable_id = Some(extract_id(&created_variable)?);
            print_json("Created variable", &created_variable)?;

            let listed_variables = functions
                .list_variables(function_id_ref)
                .await
                .context("functions.list_variables() failed")?;
            print_json("Listed variables", &listed_variables)?;

            let variable_id_ref = required_id(&variable_id, "variable")?;

            let fetched_variable = functions
                .get_variable(function_id_ref, variable_id_ref)
                .await
                .context("functions.get_variable() failed")?;
            print_json("Fetched variable", &fetched_variable)?;

            let updated_variable = functions
                .update_variable(
                    function_id_ref,
                    variable_id_ref,
                    "PLAYGROUND_SOURCE",
                    Some("rust-updated"),
                    Some(false),
                )
                .await
                .context("functions.update_variable() failed")?;
            print_json("Updated variable", &updated_variable)?;

            let sync_execution = functions
                .create_execution(
                    function_id_ref,
                    Some("{\"mode\":\"sync\"}"),
                    Some(false),
                    Some("/"),
                    Some(ExecutionMethod::POST),
                    Some(json!({ "content-type": "application/json" })),
                    None,
                )
                .await
                .context("functions.create_execution(sync) failed")?;
            print_json("Created sync execution", &sync_execution)?;

            let async_execution = functions
                .create_execution(
                    function_id_ref,
                    Some("{\"mode\":\"async\"}"),
                    Some(true),
                    Some("/"),
                    Some(ExecutionMethod::POST),
                    Some(json!({ "content-type": "application/json" })),
                    None,
                )
                .await
                .context("functions.create_execution(async) failed")?;
            print_json("Created async execution", &async_execution)?;

            let execution_id = extract_id(&async_execution)?;
            spin_wait("Waiting for async execution to finish...", Duration::from_secs(2)).await;

            let fetched_execution = functions
                .get_execution(function_id_ref, &execution_id)
                .await
                .context("functions.get_execution() failed")?;
            print_json("Fetched async execution", &fetched_execution)?;

            let listed_executions = functions
                .list_executions(function_id_ref, None, Some(true))
                .await
                .context("functions.list_executions() failed")?;
            print_json("Listed executions", &listed_executions)?;

            Ok(())
        }
        .await;

        cleanup_function_resources(&functions, &function_id, &variable_id).await;

        if let Some(path) = archive_path {
            if let Err(error) = fs::remove_file(&path) {
                eprintln!(
                    "Cleanup warning: failed to remove temporary archive {}: {}",
                    path.display(),
                    error
                );
            }
        }

        result
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    match real_main().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error:#}");
            ExitCode::from(1)
        }
    }
}

async fn real_main() -> Result<()> {
    print_banner();

    let args: Vec<String> = env::args().skip(1).collect();
    let demos = parse_requested_demos(&args)?;
    let config = Config::from_env()?;

    println!(
        "  {} {}",
        style("Endpoint:").dim(),
        style(&config.endpoint).cyan()
    );
    println!(
        "  {} {}",
        style("Project: ").dim(),
        style(&config.project_id).cyan()
    );
    println!();

    let playground = Playground::new(config);
    playground.run(&demos).await
}

fn parse_requested_demos(args: &[String]) -> Result<Vec<Demo>> {
    if args.len() == 1 && matches!(args[0].as_str(), "help" | "--help" | "-h") {
        print_usage();
        std::process::exit(0);
    }

    if !args.is_empty() {
        let mut demos = Vec::new();

        for raw in args {
            if raw == "all" {
                for demo in Demo::all() {
                    push_unique_demo(&mut demos, demo);
                }
                continue;
            }

            let demo = Demo::parse(raw).ok_or_else(|| {
                anyhow!(
                    "Unknown command `{}`.\n\nAvailable commands: health, tablesdb, storage, users, functions, all",
                    raw
                )
            })?;
            push_unique_demo(&mut demos, demo);
        }

        return Ok(demos);
    }

    // Interactive selection
    let all_demos = Demo::all();
    let labels: Vec<String> = all_demos.iter().map(|d| d.to_string()).collect();

    println!(
        "  {} {}\n",
        Emoji("👇 ", ""),
        style("Pick the demos you want to run (Space to toggle, Enter to confirm):")
            .bold()
    );

    let selected = MultiSelect::new()
        .items(&labels)
        .defaults(&[true, false, false, false, false])
        .interact()
        .context("failed to read selection")?;

    if selected.is_empty() {
        bail!("No demos selected. Run with --help for usage info.");
    }

    println!();
    Ok(selected.into_iter().map(|i| all_demos[i]).collect())
}

fn push_unique_demo(demos: &mut Vec<Demo>, demo: Demo) {
    if !demos.contains(&demo) {
        demos.push(demo);
    }
}

fn print_usage() {
    print_banner();
    println!("{}",   style("Usage:").bold().underlined());
    println!("  cargo run                          Interactive demo picker");
    println!("  cargo run -- health                Run a specific demo");
    println!("  cargo run -- databases storage     Run multiple demos");
    println!("  cargo run -- all                   Run every demo");
    println!();
    println!("{}", style("Available demos:").bold().underlined());
    for demo in Demo::all() {
        println!("  {:<12} {}", style(demo.label().to_lowercase()).cyan(), demo.description());
    }
    println!();
    println!("{}", style("Required env vars:").bold().underlined());
    println!("  APPWRITE_ENDPOINT     APPWRITE_PROJECT_ID");
    println!("  APPWRITE_API_KEY      APPWRITE_SELF_SIGNED");
    println!();
    println!("{}", style("Optional env vars:").bold().underlined());
    println!("  APPWRITE_SAMPLE_FILE              APPWRITE_FUNCTION_SOURCE_DIR");
}

fn config_help() -> &'static str {
    "APPWRITE_ENDPOINT\nAPPWRITE_PROJECT_ID\nAPPWRITE_API_KEY\nAPPWRITE_SELF_SIGNED"
}

fn print_banner() {
    let title = style("Appwrite Rust Playground").magenta().bold();
    let border = style("─".repeat(40)).dim();
    println!();
    println!("  {border}");
    println!("  {}", title);
    println!("  {border}");
    println!();
}

fn heading(title: &str) {
    let heading_style = Style::new().bold().cyan();
    let bar = style("━".repeat(title.len() + 4)).cyan().dim();
    println!();
    println!("  {bar}");
    println!("  {} {} {}", style("┃").cyan().dim(), heading_style.apply_to(title), style("┃").cyan().dim());
    println!("  {bar}");
}

fn print_json<T>(label: &str, value: &T) -> Result<()>
where
    T: Serialize + ?Sized,
{
    println!();
    println!("  {} {}", style("→").cyan().bold(), style(label).bold());

    let json_str = serde_json::to_string_pretty(value)?;
    for line in json_str.lines() {
        println!("    {}", colorize_json_line(line));
    }
    Ok(())
}

fn colorize_json_line(line: &str) -> String {
    let trimmed = line.trim();

    // Key-value lines like `"key": value`
    if let Some(colon_pos) = trimmed.find("\": ") {
        let leading_ws = &line[..line.len() - line.trim_start().len()];
        let key_part = &trimmed[..colon_pos + 1]; // includes closing quote
        let value_part = &trimmed[colon_pos + 3..];
        return format!(
            "{}{}  {}",
            leading_ws,
            style(key_part).cyan(),
            colorize_json_value(value_part),
        );
    }

    // Pure values (array elements, etc.)
    colorize_json_value(line).to_string()
}

fn colorize_json_value(value: &str) -> String {
    let trimmed = value.trim().trim_end_matches(',');
    if trimmed.starts_with('"') {
        style(value).green().to_string()
    } else if trimmed == "true" || trimmed == "false" {
        style(value).yellow().to_string()
    } else if trimmed == "null" {
        style(value).dim().to_string()
    } else if trimmed.parse::<f64>().is_ok() {
        style(value).yellow().to_string()
    } else {
        value.to_string() // braces, brackets, etc.
    }
}

async fn spin_wait(message: &str, duration: Duration) {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("  {spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "✔"]),
    );
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(StdDuration::from_millis(80));
    sleep(duration).await;
    spinner.finish_with_message(format!("{} done", message));
}

fn extract_id<T>(value: &T) -> Result<String>
where
    T: Serialize + ?Sized,
{
    string_field(value, "$id").ok_or_else(|| anyhow!("response did not include `$id`"))
}

fn string_field<T>(value: &T, key: &str) -> Option<String>
where
    T: Serialize + ?Sized,
{
    let json_value = serde_json::to_value(value).ok()?;
    json_value.get(key)?.as_str().map(ToOwned::to_owned)
}

fn required_id<'a>(value: &'a Option<String>, label: &str) -> Result<&'a str> {
    value
        .as_deref()
        .ok_or_else(|| anyhow!("missing {label} id from previous step"))
}

fn default_table_permissions() -> Vec<String> {
    vec![
        Permission::read(Role::any()).to_string(),
        Permission::create(Role::users(None::<&str>)).to_string(),
        Permission::update(Role::users(None::<&str>)).to_string(),
        Permission::delete(Role::users(None::<&str>)).to_string(),
    ]
}

fn default_row_permissions() -> Vec<String> {
    vec![
        Permission::read(Role::any()).to_string(),
        Permission::update(Role::users(None::<&str>)).to_string(),
        Permission::delete(Role::users(None::<&str>)).to_string(),
    ]
}

fn default_bucket_permissions() -> Vec<String> {
    vec![
        Permission::read(Role::any()).to_string(),
        Permission::create(Role::users(None::<&str>)).to_string(),
        Permission::update(Role::users(None::<&str>)).to_string(),
        Permission::delete(Role::users(None::<&str>)).to_string(),
    ]
}

fn default_file_permissions() -> Vec<String> {
    vec![
        Permission::read(Role::any()).to_string(),
        Permission::update(Role::users(None::<&str>)).to_string(),
        Permission::delete(Role::users(None::<&str>)).to_string(),
    ]
}

fn read_required_env(key: &str, missing: &mut Vec<String>) -> Option<String> {
    match env::var(key) {
        Ok(value) if !value.trim().is_empty() => Some(value),
        _ => {
            missing.push(key.to_string());
            None
        }
    }
}

fn read_bool_env(key: &str) -> bool {
    match env::var(key) {
        Ok(value) => matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => false,
    }
}

fn read_path_env(key: &str, default: PathBuf) -> PathBuf {
    match env::var(key) {
        Ok(value) if !value.trim().is_empty() => PathBuf::from(value),
        _ => default,
    }
}

fn manifest_path<const N: usize>(parts: [&str; N]) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for part in parts {
        path.push(part);
    }
    path
}

fn ensure_path_exists(path: &Path, label: &str) -> Result<()> {
    if path.exists() {
        Ok(())
    } else {
        bail!("{} not found at {}", label, path.display())
    }
}

fn package_function_source(source_dir: &Path) -> Result<PathBuf> {
    let archive_path = env::temp_dir().join(format!(
        "appwrite-rust-playground-function-{}.tar.gz",
        ID::unique()
    ));

    let output = Command::new("tar")
        .arg("-czf")
        .arg(&archive_path)
        .arg("-C")
        .arg(source_dir)
        .arg(".")
        .output()
        .context("failed to execute `tar` while packaging function source")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "`tar` failed while packaging function source: {}",
            stderr.trim()
        );
    }

    Ok(archive_path)
}

async fn wait_for_deployment_ready(
    functions: &Functions,
    function_id: &str,
    deployment_id: &str,
) -> Result<()> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("  {spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "✔"]),
    );
    spinner.enable_steady_tick(StdDuration::from_millis(80));

    for attempt in 1..=30 {
        spinner.set_message(format!("Waiting for deployment to become ready [{attempt}/30]..."));
        sleep(Duration::from_secs(2)).await;

        let deployment = functions
            .get_deployment(function_id, deployment_id)
            .await
            .with_context(|| format!("functions.get_deployment() failed on attempt {attempt}"))?;

        let status = string_field(&deployment, "status").unwrap_or_else(|| "unknown".to_string());

        match status.as_str() {
            "ready" => {
                spinner.finish_with_message(format!(
                    "{} Deployment ready!",
                    style("✔").green().bold()
                ));
                return Ok(());
            }
            "failed" => {
                spinner.finish_with_message(format!(
                    "{} Deployment failed",
                    style("✘").red().bold()
                ));
                bail!("deployment failed to build");
            }
            _ => {}
        }
    }

    spinner.finish_with_message(format!("{} Deployment timed out", style("✘").red().bold()));
    bail!("deployment timed out waiting for `ready` status")
}

async fn cleanup_tablesdb_resources(
    tablesdb: &TablesDB,
    database_id: &Option<String>,
    table_id: &Option<String>,
    row_id: &Option<String>,
) {
    if let (Some(database_id), Some(table_id), Some(row_id)) = (
        database_id.as_deref(),
        table_id.as_deref(),
        row_id.as_deref(),
    ) {
        if let Err(error) = tablesdb
            .delete_row(database_id, table_id, row_id, None)
            .await
        {
            cleanup_warning("delete row", error);
        }
    }

    if let (Some(database_id), Some(table_id)) =
        (database_id.as_deref(), table_id.as_deref())
    {
        if let Err(error) = tablesdb
            .delete_table(database_id, table_id)
            .await
        {
            cleanup_warning("delete table", error);
        }
    }

    if let Some(database_id) = database_id.as_deref() {
        if let Err(error) = tablesdb.delete(database_id).await {
            cleanup_warning("delete database", error);
        }
    }
}

async fn cleanup_storage_resources(
    storage: &Storage,
    bucket_id: &Option<String>,
    file_id: &Option<String>,
) {
    if let (Some(bucket_id), Some(file_id)) = (bucket_id.as_deref(), file_id.as_deref()) {
        if let Err(error) = storage.delete_file(bucket_id, file_id).await {
            cleanup_warning("delete file", error);
        }
    }

    if let Some(bucket_id) = bucket_id.as_deref() {
        if let Err(error) = storage.delete_bucket(bucket_id).await {
            cleanup_warning("delete bucket", error);
        }
    }
}

async fn cleanup_user_resources(users: &Users, user_id: &Option<String>) {
    if let Some(user_id) = user_id.as_deref() {
        if let Err(error) = users.delete(user_id).await {
            cleanup_warning("delete user", error);
        }
    }
}

async fn cleanup_function_resources(
    functions: &Functions,
    function_id: &Option<String>,
    variable_id: &Option<String>,
) {
    if let (Some(function_id), Some(variable_id)) = (function_id.as_deref(), variable_id.as_deref())
    {
        if let Err(error) = functions.delete_variable(function_id, variable_id).await {
            cleanup_warning("delete function variable", error);
        }
    }

    if let Some(function_id) = function_id.as_deref() {
        if let Err(error) = functions.delete(function_id).await {
            cleanup_warning("delete function", error);
        }
    }
}

fn cleanup_warning(label: &str, error: impl std::fmt::Display) {
    eprintln!(
        "  {} {}",
        style("⚠").yellow(),
        style(format!("Cleanup: failed to {label}: {error}")).yellow().dim()
    );
}
