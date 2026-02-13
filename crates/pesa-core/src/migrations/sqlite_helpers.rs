/// SQLite foreign key helpers for SeaORM migrations.
///
/// This module provides utilities to work around SQLite's limitations with foreign key constraints
/// by implementing table recreation operations that preserve existing data while adding or
/// removing foreign keys.
///
/// # Example
///
/// ```ignore
/// use sea_orm_migration::prelude::*;
/// use crate::migrations::sqlite_helpers::{AddForeignKeyBuilder, DropForeignKeyBuilder};
///
/// // Add a foreign key to an existing table
/// AddForeignKeyBuilder::create()
///     .from(Users::Table)
///     .to(Profiles::Table)
///     .with_column(Users::ProfileId, Profiles::Id)
///     .on_delete(ForeignKeyAction::SetNull)
///     .execute(&manager, Users::Table)
///     .await?;
///
/// // Drop a foreign key from an existing table
/// DropForeignKeyBuilder::create()
///     .from_table(Users::Table)
///     .to(Profiles::Table)
///     .with_column(Users::ProfileId, Profiles::Id)
///     .execute(&manager, Users::Table)
///     .await?;
/// ```
///
/// # Safety
///
/// The helpers ensure that:
/// - All existing data is preserved during table recreation
/// - Indexes and constraints are maintained
/// - Foreign key constraints continue to work after operations
/// - Transactions are used properly to prevent corruption
///
/// # Limitations
///
/// - SQLite does not support direct foreign key modification
/// - Tables are completely recreated, which may be slow for large tables
/// - Only foreign key columns can be modified during recreation
///
use std::collections::HashMap;

use sea_orm::{
    ConnectionTrait, DbBackend, DbErr, QueryResult, Statement,
    sea_query::{ColumnDef, *},
};
use sea_orm_migration::prelude::*;

/// Represents all details of a foreign key constraint for migration purposes.
///
/// This is used internally because SQLite's `PRAGMA foreign_key_list` does not provide
/// constraint names, so we identify constraints by their characteristics.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MigrationForeignKeyInfo {
    /// Internal ID from PRAGMA, used for grouping
    pub id: u32,
    /// The table on which the foreign key is defined (the one being recreated)
    pub from_table: String,
    /// The table being referenced
    pub to_table: String,
    /// Ordered pairs of (from_column, to_column)
    pub columns: Vec<(String, String)>,
    /// Action on update, e.g. "NO ACTION", "CASCADE", "SET NULL"
    pub on_update: String,
    /// Action on delete, e.g. "NO ACTION", "CASCADE", "SET NULL"
    pub on_delete: String,
}

/// Identifier for dropping a foreign key, based on its unique characteristics.
///
/// Since SQLite foreign keys don't have explicit names in PRAGMA output,
/// we identify constraints by their structural properties.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ForeignKeyDropIdentifier {
    /// The target table of the foreign key
    pub to_table: String,
    /// Column pairs that identify the foreign key
    pub columns: Vec<(String, String)>,
}

/// Builder for creating a new foreign key constraint.
///
/// # Examples
///
/// ```ignore
/// use sea_orm_migration::prelude::*;
/// use crate::migrations::sqlite_helpers::AddForeignKeyBuilder;
///
/// let fk = AddForeignKeyBuilder::create()
///     .from(Users::Table)
///     .to(Profiles::Table)
///     .with_column(Users::ProfileId, Profiles::Id)
///     .on_delete(ForeignKeyAction::SetNull)
///     .execute(&manager, Users::Table)
///     .await?;
/// ```
pub struct AddForeignKeyBuilder {
    /// Source table of the foreign key
    from_table: String,
    /// Target table of the foreign key
    to_table: String,
    /// Column pairs defining the relationship
    columns: Vec<(String, String)>,
    /// Action on update of referenced rows
    on_update: String,
    /// Action on delete of referenced rows
    on_delete: String,
}

impl AddForeignKeyBuilder {
    pub fn create() -> Self {
        Self {
            from_table: String::new(),
            to_table: String::new(),
            columns: Vec::new(),
            on_update: "NO ACTION".to_string(),
            on_delete: "NO ACTION".to_string(),
        }
    }

    pub fn from<T: Iden>(mut self, table: T) -> Self {
        self.from_table = iden_to_string(&table);
        self
    }

    pub fn to<T: Iden>(mut self, table: T) -> Self {
        self.to_table = iden_to_string(&table);
        self
    }

    pub fn with_columns<FC: Iden, TC: Iden>(mut self, column_pairs: Vec<(FC, TC)>) -> Self {
        self.columns = column_pairs
            .into_iter()
            .map(|(fc, tc)| (iden_to_string(&fc), iden_to_string(&tc)))
            .collect();
        self
    }

    pub fn with_column<FC: Iden, TC: Iden>(mut self, from_col: FC, to_col: TC) -> Self {
        self.columns
            .push((iden_to_string(&from_col), iden_to_string(&to_col)));
        self
    }

    pub fn on_update(mut self, action: ForeignKeyAction) -> Self {
        self.on_update = foreign_key_action_to_string(&action);
        self
    }

    pub fn on_delete(mut self, action: ForeignKeyAction) -> Self {
        self.on_delete = foreign_key_action_to_string(&action);
        self
    }

    pub async fn execute<R: IntoTableRef>(
        self,
        manager: &SchemaManager<'_>,
        table: R,
    ) -> Result<(), DbErr> {
        let fk_info = MigrationForeignKeyInfo {
            id: 0, // New FK, doesn't need an ID
            from_table: self.from_table,
            to_table: self.to_table,
            columns: self.columns,
            on_update: self.on_update,
            on_delete: self.on_delete,
        };

        add_foreign_key_internal(manager, &table.into_table_ref(), fk_info).await
    }
}

/// Builder for dropping a foreign key constraint.
///
/// # Examples
///
/// ```ignore
/// use sea_orm_migration::prelude::*;
/// use pesa_core::migrations::sqlite_helpers::DropForeignKeyBuilder;
///
///
/// DropForeignKeyBuilder::create()
///     .from_table(Users::Table)
///     .to(Profiles::Table)
///     .with_column(Users::ProfileId, Profiles::Id)
///     .execute(&manager, Users::Table)
///     .await?;
/// ```
pub struct DropForeignKeyBuilder {
    /// Target table of the foreign key being dropped
    to_table: String,
    /// Column pairs that identify the foreign key to drop
    columns: Vec<(String, String)>,
}

impl DropForeignKeyBuilder {
    pub fn create() -> Self {
        Self {
            to_table: String::new(),
            columns: Vec::new(),
        }
    }

    pub fn from_table<T: Iden>(self, _table: T) -> Self {
        // For drop FK, we mainly need the target table and columns
        // The from table context comes from the execute() call
        self
    }

    pub fn to<T: Iden>(mut self, table: T) -> Self {
        self.to_table = iden_to_string(&table);
        self
    }

    pub fn with_column<FC: Iden, TC: Iden>(mut self, from_col: FC, to_col: TC) -> Self {
        self.columns
            .push((iden_to_string(&from_col), iden_to_string(&to_col)));
        self
    }

    pub fn with_columns<FC: Iden, TC: Iden>(mut self, column_pairs: Vec<(FC, TC)>) -> Self {
        self.columns = column_pairs
            .into_iter()
            .map(|(fc, tc)| (iden_to_string(&fc), iden_to_string(&tc)))
            .collect();
        self
    }

    pub async fn execute<R: IntoTableRef>(
        self,
        manager: &SchemaManager<'_>,
        table: R,
    ) -> Result<(), DbErr> {
        let identifier = ForeignKeyDropIdentifier {
            to_table: self.to_table,
            columns: self.columns,
        };

        drop_foreign_key_internal(manager, &table.into_table_ref(), identifier).await
    }
}

/// Helper function to convert Iden to string
fn iden_to_string<T: Iden>(iden: &T) -> String {
    let mut buf = String::new();
    iden.unquoted(&mut buf);
    buf
}

/// Helper function to convert ForeignKeyAction enum to string
fn foreign_key_action_to_string(action: &ForeignKeyAction) -> String {
    match action {
        ForeignKeyAction::Restrict => "RESTRICT".to_string(),
        ForeignKeyAction::Cascade => "CASCADE".to_string(),
        ForeignKeyAction::SetNull => "SET NULL".to_string(),
        ForeignKeyAction::SetDefault => "SET DEFAULT".to_string(),
        ForeignKeyAction::NoAction => "NO ACTION".to_string(),
    }
}

/// Internal helper to add a foreign key to an existing table in SQLite.
async fn add_foreign_key_internal(
    manager: &SchemaManager<'_>,
    table: &TableRef,
    new_fk_info: MigrationForeignKeyInfo,
) -> Result<(), DbErr> {
    let db = manager.get_connection();
    let table_name_str = table_ref_to_string(table);
    let mut fks = get_foreign_keys(db, &table_name_str).await?;
    fks.push(new_fk_info);
    recreate_table(manager, table, &fks).await
}

/// Internal helper to drop a foreign key from an existing table in SQLite.
async fn drop_foreign_key_internal(
    manager: &SchemaManager<'_>,
    table: &TableRef,
    identifier: ForeignKeyDropIdentifier,
) -> Result<(), DbErr> {
    let db = manager.get_connection();
    let table_name_str = table_ref_to_string(table);
    let fks = get_foreign_keys(db, &table_name_str).await?;
    let fks_to_keep: Vec<MigrationForeignKeyInfo> = fks
        .into_iter()
        .filter(|fk_info| {
            // Compare the properties of MigrationForeignKeyInfo with the identifier
            !(fk_info.to_table == identifier.to_table && fk_info.columns == identifier.columns)
        })
        .collect();
    recreate_table(manager, table, &fks_to_keep).await
}

/// Helper function to convert TableRef to string
fn table_ref_to_string(table_ref: &TableRef) -> String {
    match table_ref {
        TableRef::Table(iden) => dyn_iden_to_string(&**iden),
        TableRef::SchemaTable(_, iden) => dyn_iden_to_string(&**iden),
        TableRef::DatabaseSchemaTable(_, _, iden) => dyn_iden_to_string(&**iden),
        TableRef::TableAlias(iden, _) => dyn_iden_to_string(&**iden),
        TableRef::SchemaTableAlias(_, iden, _) => dyn_iden_to_string(&**iden),
        TableRef::DatabaseSchemaTableAlias(_, _, iden, _) => dyn_iden_to_string(&**iden),
        TableRef::SubQuery(_, iden) => dyn_iden_to_string(&**iden),
        TableRef::ValuesList(_, iden) => dyn_iden_to_string(&**iden),
        TableRef::FunctionCall(_, iden) => dyn_iden_to_string(&**iden),
    }
}

/// Helper function to convert dyn Iden to string
fn dyn_iden_to_string(iden: &dyn Iden) -> String {
    let mut buf = String::new();
    iden.unquoted(&mut buf);
    buf
}

async fn recreate_table(
    manager: &SchemaManager<'_>,
    table_ref: &TableRef,
    fks_to_create: &[MigrationForeignKeyInfo],
) -> Result<(), DbErr> {
    let db = manager.get_connection();
    let table_name_str = table_ref_to_string(table_ref);

    let (columns, indexes) = introspect_table(db, &table_name_str).await?;

    db.execute_unprepared("PRAGMA foreign_keys=OFF;").await?;

    let temp_table_name = format!("_new_{}", &table_name_str);
    let temp_table_ref = TableRef::Table(Alias::new(&temp_table_name).into_iden());

    let mut new_table_stmt = Table::create();
    new_table_stmt.table(temp_table_ref.clone()).if_not_exists();

    let mut column_names = Vec::new();
    for col_info in &columns {
        column_names.push(col_info.name.clone());
        let mut col_def = ColumnDef::new(Alias::new(&col_info.name));
        apply_column_attributes(&mut col_def, col_info);
        new_table_stmt.col(&mut col_def);
    }

    // Convert MigrationForeignKeyInfo to ForeignKeyCreateStatement
    for fk_info in fks_to_create {
        let mut fk = ForeignKey::create();

        // The from_tbl is implicit when defining a foreign key inside a CREATE TABLE statement.
        // Calling it was causing issues with self-referencing tables.
        // fk.from_tbl(Alias::new(&fk_info.from_table));

        // Add from columns
        for (from_column, _) in &fk_info.columns {
            fk.from_col(Alias::new(from_column));
        }

        // Handle self-referencing foreign keys. The "to" table must be the new
        // temporary table, not the old one.
        if fk_info.to_table == table_name_str {
            fk.to_tbl(Alias::new(&temp_table_name));
        } else {
            fk.to_tbl(Alias::new(&fk_info.to_table));
        }

        // Add to columns
        for (_, to_column) in &fk_info.columns {
            fk.to_col(Alias::new(to_column));
        }

        // Apply on_update and on_delete actions
        match fk_info.on_update.as_str() {
            "RESTRICT" => {
                fk.on_update(ForeignKeyAction::Restrict);
            }
            "CASCADE" => {
                fk.on_update(ForeignKeyAction::Cascade);
            }
            "SET NULL" => {
                fk.on_update(ForeignKeyAction::SetNull);
            }
            "SET DEFAULT" => {
                fk.on_update(ForeignKeyAction::SetDefault);
            }
            _ => {
                fk.on_update(ForeignKeyAction::NoAction);
            }
        }
        match fk_info.on_delete.as_str() {
            "RESTRICT" => {
                fk.on_delete(ForeignKeyAction::Restrict);
            }
            "CASCADE" => {
                fk.on_delete(ForeignKeyAction::Cascade);
            }
            "SET NULL" => {
                fk.on_delete(ForeignKeyAction::SetNull);
            }
            "SET DEFAULT" => {
                fk.on_delete(ForeignKeyAction::SetDefault);
            }
            _ => {
                fk.on_delete(ForeignKeyAction::NoAction);
            }
        }
        new_table_stmt.foreign_key(&mut fk);
    }

    manager.create_table(new_table_stmt).await?;

    let insert_stmt = Query::insert()
        .into_table(temp_table_ref.clone())
        .columns(column_names.iter().map(|c| Alias::new(c.as_str())))
        .select_from(
            Query::select()
                .columns(column_names.iter().map(|c| Alias::new(c.as_str())))
                .from(table_ref.clone())
                .to_owned(),
        )
        .map_err(|e| DbErr::Migration(e.to_string()))?
        .to_owned();
    manager.exec_stmt(insert_stmt).await?;

    manager
        .drop_table(Table::drop().table(table_ref.clone()).to_owned())
        .await?;
    let rename_stmt = Table::rename()
        .table(temp_table_ref, table_ref.clone())
        .to_owned();
    manager.rename_table(rename_stmt).await?;

    for index_info in &indexes {
        if !index_info.name.starts_with("sqlite_autoindex") {
            let mut index_stmt = Index::create();
            index_stmt.name(&index_info.name).table(table_ref.clone());
            if index_info.unique {
                index_stmt.unique();
            }
            for col_name in &index_info.columns {
                index_stmt.col(Alias::new(col_name));
            }
            manager.create_index(index_stmt).await?;
        }
    }

    db.execute_unprepared("PRAGMA foreign_keys=ON;").await?;
    Ok(())
}

#[derive(Debug)]
struct ColumnInfo {
    name: String,
    col_type: String,
    not_null: bool,
    default_value: Option<String>,
    is_pk: bool,
}
#[derive(Debug)]
struct IndexInfo {
    name: String,
    unique: bool,
    columns: Vec<String>,
}

async fn get_foreign_keys(
    db: &impl ConnectionTrait,
    table_name: &str,
) -> Result<Vec<MigrationForeignKeyInfo>, DbErr> {
    // Intermediate struct to build up MigrationForeignKeyInfo, handling multiple columns per FK
    #[derive(Debug)]
    struct FKBuilder {
        id: u32,
        from_table: String,
        to_table: String,
        /// (seq, from_col, to_col)
        columns_with_seq: Vec<(u32, String, String)>,
        on_update: String,
        on_delete: String,
    }

    let pragma_fks: Vec<QueryResult> = db
        .query_all(Statement::from_string(
            DbBackend::Sqlite,
            format!("PRAGMA foreign_key_list('{}');", table_name),
        ))
        .await?;

    let mut fk_builders: HashMap<u32, FKBuilder> = HashMap::new();

    for row in pragma_fks {
        let id: u32 = row.try_get("", "id")?;
        let seq: u32 = row.try_get("", "seq")?;
        let foreign_table: String = row.try_get("", "table")?;
        let from_col: String = row.try_get("", "from")?;
        let to_col: String = row.try_get("", "to")?;
        let on_update: String = row.try_get("", "on_update")?;
        let on_delete: String = row.try_get("", "on_delete")?;

        fk_builders
            .entry(id)
            .and_modify(|builder| {
                builder
                    .columns_with_seq
                    .push((seq, from_col.clone(), to_col.clone()));
            })
            .or_insert_with(|| FKBuilder {
                id,
                from_table: table_name.to_string(), // The table we are introspecting
                to_table: foreign_table,
                columns_with_seq: vec![(seq, from_col, to_col)],
                on_update,
                on_delete,
            });
    }

    let mut fk_infos = Vec::new();
    for (_id, mut builder) in fk_builders {
        // Sort columns by sequence number to ensure correct order for multi-column FKs
        builder.columns_with_seq.sort_by_key(|k| k.0);
        let columns = builder
            .columns_with_seq
            .into_iter()
            .map(|(_, from, to)| (from, to))
            .collect();

        fk_infos.push(MigrationForeignKeyInfo {
            id: builder.id,
            from_table: builder.from_table,
            to_table: builder.to_table,
            columns,
            on_update: builder.on_update,
            on_delete: builder.on_delete,
        });
    }

    Ok(fk_infos)
}

async fn introspect_table(
    db: &impl ConnectionTrait,
    table_name: &str,
) -> Result<(Vec<ColumnInfo>, Vec<IndexInfo>), DbErr> {
    let pragma_cols: Vec<QueryResult> = db
        .query_all(Statement::from_string(
            DbBackend::Sqlite,
            format!("PRAGMA table_info('{}');", table_name),
        ))
        .await?;
    let columns: Vec<ColumnInfo> = pragma_cols
        .into_iter()
        .map(|row| ColumnInfo {
            name: row.try_get("", "name").unwrap(),
            col_type: row.try_get("", "type").unwrap(),
            not_null: row.try_get::<i32>("", "notnull").unwrap() == 1,
            default_value: row.try_get("", "dflt_value").unwrap(),
            is_pk: row.try_get::<i32>("", "pk").unwrap() > 0,
        })
        .collect();

    let pragma_indexes: Vec<QueryResult> = db
        .query_all(Statement::from_string(
            DbBackend::Sqlite,
            format!("PRAGMA index_list('{}');", table_name),
        ))
        .await?;
    let mut indexes = Vec::new();
    for index_row in pragma_indexes {
        let index_name: String = index_row.try_get("", "name")?;
        let unique: bool = index_row.try_get::<i32>("", "unique")? == 1;
        let pragma_index_cols: Vec<QueryResult> = db
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                format!("PRAGMA index_info('{}');", index_name),
            ))
            .await?;
        let index_columns: Vec<String> = pragma_index_cols
            .into_iter()
            .map(|r| r.try_get("", "name").unwrap())
            .collect();
        indexes.push(IndexInfo {
            name: index_name,
            unique,
            columns: index_columns,
        });
    }
    Ok((columns, indexes))
}

fn apply_column_attributes(col_def: &mut ColumnDef, col_info: &ColumnInfo) {
    match col_info.col_type.to_uppercase().as_str() {
        "INTEGER" => {
            col_def.integer();
        }
        "TEXT" | "VARCHAR" => {
            col_def.string();
        }
        "BLOB" => {
            col_def.binary();
        }
        "REAL" | "FLOAT" => {
            col_def.float();
        }
        "DOUBLE" => {
            col_def.double();
        }
        "BOOLEAN" => {
            col_def.boolean();
        }
        s if s.contains("TIMESTAMP") => {
            col_def.timestamp_with_time_zone();
        }
        _ => {
            col_def.string();
        }
    }
    if col_info.not_null {
        col_def.not_null();
    }
    if let Some(default) = &col_info.default_value {
        col_def.default(Expr::cust(default.clone()));
    }
    if col_info.is_pk {
        col_def.primary_key();
        if col_info.col_type.to_uppercase() == "INTEGER" {
            col_def.auto_increment();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{Database, DbErr, Statement, TransactionTrait};
    use sea_orm_migration::SchemaManager;
    use sea_query::{Alias, Table};
    use std::collections::HashSet;

    #[derive(Iden)]
    enum Users {
        Table,
        Id,
        Name,
        Email,
        ProfileId,
    }

    #[derive(Iden)]
    enum Profiles {
        Table,
        Id,
        Bio,
    }

    #[derive(Iden)]
    enum Posts {
        Table,
        Id,
        Title,
        UserId,
    }

    #[derive(Iden)]
    enum Tags {
        Table,
        Id,
        Name,
    }

    #[derive(Iden)]
    enum PostTags {
        Table,
        PostId,
        TagId,
    }

    async fn setup_test_db() -> Result<sea_orm::DatabaseConnection, DbErr> {
        let mut opts = sea_orm::ConnectOptions::from("sqlite::memory:");
        opts.max_connections(10)
            .connect_timeout(std::time::Duration::from_secs(10));
        let db = Database::connect(opts).await?;

        // Enable foreign keys and verify they are active
        db.execute_unprepared("PRAGMA foreign_keys = ON").await?;

        let fk_check = db
            .query_one(Statement::from_string(
                DbBackend::Sqlite,
                "PRAGMA foreign_keys".to_string(),
            ))
            .await?;
        assert!(fk_check.is_some());
        let fk_enabled: i32 = fk_check.unwrap().try_get("", "foreign_keys")?;
        assert_eq!(fk_enabled, 1, "Foreign keys must be enabled");

        Ok(db)
    }

    async fn create_test_tables(db: &sea_orm::DatabaseConnection) -> Result<(), DbErr> {
        let manager = SchemaManager::new(db);

        // Create users table
        let users_table = Table::create()
            .table(Users::Table)
            .col(
                ColumnDef::new(Users::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(Users::Name).string().not_null())
            .col(ColumnDef::new(Users::Email).string().null())
            .col(ColumnDef::new(Users::ProfileId).integer().null())
            .to_owned();
        manager.create_table(users_table).await?;

        // Create profiles table
        let profiles_table = Table::create()
            .table(Profiles::Table)
            .col(
                ColumnDef::new(Profiles::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(Profiles::Bio).string().null())
            .to_owned();
        manager.create_table(profiles_table).await?;

        // Create posts table
        let posts_table = Table::create()
            .table(Posts::Table)
            .col(
                ColumnDef::new(Posts::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(Posts::Title).string().not_null())
            .col(ColumnDef::new(Posts::UserId).integer().not_null())
            .to_owned();
        manager.create_table(posts_table).await?;

        // Create tags table
        let tags_table = Table::create()
            .table(Tags::Table)
            .col(
                ColumnDef::new(Tags::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(Tags::Name).string().not_null())
            .to_owned();
        manager.create_table(tags_table).await?;

        // Create post_tags junction table
        let post_tags_table = Table::create()
            .table(PostTags::Table)
            .col(ColumnDef::new(PostTags::PostId).integer().not_null())
            .col(ColumnDef::new(PostTags::TagId).integer().not_null())
            .to_owned();
        manager.create_table(post_tags_table).await?;

        Ok(())
    }

    async fn insert_test_data(db: &sea_orm::DatabaseConnection) -> Result<(), DbErr> {
        // Insert profiles
        db.execute_unprepared("INSERT INTO profiles (bio) VALUES ('Test bio 1'), ('Test bio 2')")
            .await?;

        // Insert users
        db.execute_unprepared(
            "INSERT INTO users (name, email, profile_id) VALUES 
             ('Alice', 'alice@example.com', 1),
             ('Bob', 'bob@example.com', NULL),
             ('Charlie', 'charlie@example.com', 2)",
        )
        .await?;

        // Insert posts
        db.execute_unprepared(
            "INSERT INTO posts (title, user_id) VALUES 
             ('Post 1', 1),
             ('Post 2', 1),
             ('Post 3', 2)",
        )
        .await?;

        // Insert tags
        db.execute_unprepared("INSERT INTO tags (name) VALUES ('tech'), ('rust'), ('database')")
            .await?;

        // Insert post_tags relationships
        db.execute_unprepared(
            "INSERT INTO post_tags (post_id, tag_id) VALUES 
             (1, 1), (1, 2), (2, 2), (2, 3), (3, 1)",
        )
        .await?;

        Ok(())
    }

    #[derive(Debug, Clone, PartialEq)]
    struct TestDataUser {
        id: i32,
        name: String,
        email: Option<String>,
        profile_id: Option<i32>,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct TestDataPost {
        id: i32,
        title: String,
        user_id: i32,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct TestDataPostTag {
        post_id: i32,
        tag_id: i32,
    }

    async fn get_all_users<C>(db: &C) -> Result<Vec<TestDataUser>, DbErr>
    where
        C: ConnectionTrait,
    {
        let results = db
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT id, name, email, profile_id FROM users ORDER BY id".to_string(),
            ))
            .await?;

        let mut users = Vec::new();
        for row in results {
            users.push(TestDataUser {
                id: row.try_get("", "id")?,
                name: row.try_get("", "name")?,
                email: row.try_get("", "email")?,
                profile_id: row.try_get("", "profile_id")?,
            });
        }
        Ok(users)
    }

    async fn get_all_posts<C>(db: &C) -> Result<Vec<TestDataPost>, DbErr>
    where
        C: ConnectionTrait,
    {
        let results = db
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT id, title, user_id FROM posts ORDER BY id".to_string(),
            ))
            .await?;

        let mut posts = Vec::new();
        for row in results {
            posts.push(TestDataPost {
                id: row.try_get("", "id")?,
                title: row.try_get("", "title")?,
                user_id: row.try_get("", "user_id")?,
            });
        }
        Ok(posts)
    }

    async fn get_all_post_tags<C>(db: &C) -> Result<Vec<TestDataPostTag>, DbErr>
    where
        C: ConnectionTrait,
    {
        let results = db
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT post_id, tag_id FROM post_tags ORDER BY post_id, tag_id".to_string(),
            ))
            .await?;

        let mut post_tags = Vec::new();
        for row in results {
            post_tags.push(TestDataPostTag {
                post_id: row.try_get("", "post_id")?,
                tag_id: row.try_get("", "tag_id")?,
            });
        }
        Ok(post_tags)
    }

    async fn verify_exact_data_integrity<C>(
        db: &C,
    ) -> Result<(Vec<TestDataUser>, Vec<TestDataPost>, Vec<TestDataPostTag>), DbErr>
    where
        C: ConnectionTrait,
    {
        let users = get_all_users(db).await?;
        let posts = get_all_posts(db).await?;
        let post_tags = get_all_post_tags(db).await?;

        // Verify expected data
        assert_eq!(users.len(), 3);
        assert_eq!(posts.len(), 3);
        assert_eq!(post_tags.len(), 5);

        // Verify specific data integrity
        assert_eq!(
            users[0],
            TestDataUser {
                id: 1,
                name: "Alice".to_string(),
                email: Some("alice@example.com".to_string()),
                profile_id: Some(1)
            }
        );
        assert_eq!(
            users[1],
            TestDataUser {
                id: 2,
                name: "Bob".to_string(),
                email: Some("bob@example.com".to_string()),
                profile_id: None
            }
        );
        assert_eq!(
            users[2],
            TestDataUser {
                id: 3,
                name: "Charlie".to_string(),
                email: Some("charlie@example.com".to_string()),
                profile_id: Some(2)
            }
        );

        assert_eq!(
            posts[0],
            TestDataPost {
                id: 1,
                title: "Post 1".to_string(),
                user_id: 1
            }
        );
        assert_eq!(
            posts[1],
            TestDataPost {
                id: 2,
                title: "Post 2".to_string(),
                user_id: 1
            }
        );
        assert_eq!(
            posts[2],
            TestDataPost {
                id: 3,
                title: "Post 3".to_string(),
                user_id: 2
            }
        );

        assert_eq!(
            post_tags[0],
            TestDataPostTag {
                post_id: 1,
                tag_id: 1
            }
        );
        assert_eq!(
            post_tags[1],
            TestDataPostTag {
                post_id: 1,
                tag_id: 2
            }
        );
        assert_eq!(
            post_tags[2],
            TestDataPostTag {
                post_id: 2,
                tag_id: 2
            }
        );
        assert_eq!(
            post_tags[3],
            TestDataPostTag {
                post_id: 2,
                tag_id: 3
            }
        );
        assert_eq!(
            post_tags[4],
            TestDataPostTag {
                post_id: 3,
                tag_id: 1
            }
        );

        Ok((users, posts, post_tags))
    }

    #[tokio::test]
    async fn test_table_recreation_preserves_all_data() -> Result<(), DbErr> {
        let db = setup_test_db().await?;
        create_test_tables(&db).await?;
        insert_test_data(&db).await?;

        let txn = db.begin().await?;

        // Get exact data BEFORE foreign key addition (before table recreation)
        let (before_users, before_posts, before_post_tags) =
            verify_exact_data_integrity(&txn).await?;

        let manager = SchemaManager::new(&txn);

        // Add foreign key (triggers table recreation)
        AddForeignKeyBuilder::create()
            .from(Users::Table)
            .to(Profiles::Table)
            .with_column(Users::ProfileId, Profiles::Id)
            .on_delete(ForeignKeyAction::SetNull)
            .execute(&manager, Users::Table)
            .await?;

        // Get exact data AFTER foreign key addition (after table recreation)
        let (after_users, after_posts, after_post_tags) = verify_exact_data_integrity(&txn).await?;

        // Verify ALL data is exactly the same - byte for byte integrity
        assert_eq!(
            before_users, after_users,
            "All user data must be preserved exactly during table recreation"
        );
        assert_eq!(
            before_posts, after_posts,
            "All post data must be preserved exactly during table recreation"
        );
        assert_eq!(
            before_post_tags, after_post_tags,
            "All post_tags data must be preserved exactly during table recreation"
        );

        // Also verify foreign key was created correctly
        let fks = txn
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "PRAGMA foreign_key_list('users')".to_string(),
            ))
            .await?;
        assert!(!fks.is_empty());
        let fk_row = &fks[0];
        let table: String = fk_row.try_get("", "table")?;
        let from_col: String = fk_row.try_get("", "from")?;
        let to_col: String = fk_row.try_get("", "to")?;
        let on_delete: String = fk_row.try_get("", "on_delete")?;
        assert_eq!(table, "profiles");
        assert_eq!(from_col, "profile_id");
        assert_eq!(to_col, "id");
        assert_eq!(on_delete, "SET NULL");

        txn.commit().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_multiple_foreign_keys_preserve_existing_data() -> Result<(), DbErr> {
        let db = setup_test_db().await?;
        create_test_tables(&db).await?;
        insert_test_data(&db).await?;

        let txn = db.begin().await?;

        // Get exact data BEFORE any foreign key operations
        let (before_users, before_posts, before_post_tags) =
            verify_exact_data_integrity(&txn).await?;

        let manager = SchemaManager::new(&txn);

        // Add first foreign key (triggers table recreation)
        AddForeignKeyBuilder::create()
            .from(PostTags::Table)
            .to(Posts::Table)
            .with_column(PostTags::PostId, Posts::Id)
            .execute(&manager, PostTags::Table)
            .await?;

        // Verify data is still exactly the same after first recreation
        let (after_first_users, after_first_posts, after_first_post_tags) =
            verify_exact_data_integrity(&txn).await?;
        assert_eq!(
            before_users, after_first_users,
            "Data must be preserved after first foreign key addition"
        );
        assert_eq!(
            before_posts, after_first_posts,
            "Data must be preserved after first foreign key addition"
        );
        assert_eq!(
            before_post_tags, after_first_post_tags,
            "Data must be preserved after first foreign key addition"
        );

        // Add second foreign key (triggers another table recreation)
        AddForeignKeyBuilder::create()
            .from(PostTags::Table)
            .to(Tags::Table)
            .with_column(PostTags::TagId, Tags::Id)
            .execute(&manager, PostTags::Table)
            .await?;

        // Verify data is still exactly the same after second recreation
        let (after_second_users, after_second_posts, after_second_post_tags) =
            verify_exact_data_integrity(&txn).await?;
        assert_eq!(
            before_users, after_second_users,
            "Data must be preserved after second foreign key addition"
        );
        assert_eq!(
            before_posts, after_second_posts,
            "Data must be preserved after second foreign key addition"
        );
        assert_eq!(
            before_post_tags, after_second_post_tags,
            "Data must be preserved after second foreign key addition"
        );

        // Verify both foreign keys exist and work correctly
        let fks = txn
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "PRAGMA foreign_key_list('post_tags')".to_string(),
            ))
            .await?;
        assert_eq!(fks.len(), 2);

        // Test that foreign key constraints are actually enforced
        let result = txn
            .execute_unprepared("INSERT INTO post_tags (post_id, tag_id) VALUES (999, 999)")
            .await;
        assert!(
            result.is_err(),
            "Foreign key constraint should reject invalid data"
        );

        txn.commit().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_drop_foreign_key_simple() -> Result<(), DbErr> {
        let db = setup_test_db().await?;
        create_test_tables(&db).await?;
        insert_test_data(&db).await?;

        let txn = db.begin().await?;
        let manager = SchemaManager::new(&txn);

        // First add a foreign key
        AddForeignKeyBuilder::create()
            .from(Users::Table)
            .to(Profiles::Table)
            .with_column(Users::ProfileId, Profiles::Id)
            .execute(&manager, Users::Table)
            .await?;

        // Verify it was added
        let fks_before = txn
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "PRAGMA foreign_key_list('users')".to_string(),
            ))
            .await?;
        assert!(!fks_before.is_empty());

        // Verify data before drop
        verify_exact_data_integrity(&txn).await?;

        // Now drop the foreign key
        DropForeignKeyBuilder::create()
            .from_table(Users::Table)
            .to(Profiles::Table)
            .with_column(Users::ProfileId, Profiles::Id)
            .execute(&manager, Users::Table)
            .await?;

        // Verify it was dropped
        let fks_after = txn
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "PRAGMA foreign_key_list('users')".to_string(),
            ))
            .await?;
        assert!(fks_after.is_empty());

        // Verify data integrity is still maintained
        verify_exact_data_integrity(&txn).await?;

        txn.commit().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_drop_foreign_key_multi_column() -> Result<(), DbErr> {
        let db = setup_test_db().await?;
        create_test_tables(&db).await?;
        insert_test_data(&db).await?;

        let txn = db.begin().await?;
        let manager = SchemaManager::new(&txn);

        // Add multiple foreign keys
        AddForeignKeyBuilder::create()
            .from(PostTags::Table)
            .to(Posts::Table)
            .with_column(PostTags::PostId, Posts::Id)
            .execute(&manager, PostTags::Table)
            .await?;

        AddForeignKeyBuilder::create()
            .from(PostTags::Table)
            .to(Tags::Table)
            .with_column(PostTags::TagId, Tags::Id)
            .execute(&manager, PostTags::Table)
            .await?;

        // Verify they were added
        let fks_before = txn
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "PRAGMA foreign_key_list('post_tags')".to_string(),
            ))
            .await?;
        assert_eq!(fks_before.len(), 2);

        // Verify data before drop
        verify_exact_data_integrity(&txn).await?;

        // Drop one of the foreign keys
        DropForeignKeyBuilder::create()
            .from_table(PostTags::Table)
            .to(Posts::Table)
            .with_column(PostTags::PostId, Posts::Id)
            .execute(&manager, PostTags::Table)
            .await?;

        // Verify only one FK remains
        let fks_after = txn
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "PRAGMA foreign_key_list('post_tags')".to_string(),
            ))
            .await?;
        assert_eq!(fks_after.len(), 1);

        // Verify the remaining FK is the one we didn't drop
        let table: String = fks_after[0].try_get("", "table")?;
        assert_eq!(table, "tags");

        // Verify data integrity is still maintained
        verify_exact_data_integrity(&txn).await?;

        txn.commit().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_table_recreation_preserves_data() -> Result<(), DbErr> {
        let db = setup_test_db().await?;
        create_test_tables(&db).await?;
        insert_test_data(&db).await?;

        let txn = db.begin().await?;

        // Get exact data BEFORE foreign key addition (before table recreation)
        let (before_users, before_posts, before_post_tags) =
            verify_exact_data_integrity(&txn).await?;

        let manager = SchemaManager::new(&txn);

        // Add foreign key (triggers table recreation)
        AddForeignKeyBuilder::create()
            .from(Users::Table)
            .to(Profiles::Table)
            .with_column(Users::ProfileId, Profiles::Id)
            .on_delete(ForeignKeyAction::SetNull)
            .execute(&manager, Users::Table)
            .await?;

        // Get exact data AFTER foreign key addition (after table recreation)
        let (after_users, after_posts, after_post_tags) = verify_exact_data_integrity(&txn).await?;

        // Verify ALL data is exactly the same - byte for byte integrity
        assert_eq!(
            before_users, after_users,
            "All user data must be preserved exactly during table recreation"
        );
        assert_eq!(
            before_posts, after_posts,
            "All post data must be preserved exactly during table recreation"
        );
        assert_eq!(
            before_post_tags, after_post_tags,
            "All post_tags data must be preserved exactly during table recreation"
        );

        // Also verify foreign key was created correctly
        let fks = txn
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "PRAGMA foreign_key_list('users')".to_string(),
            ))
            .await?;
        assert!(!fks.is_empty(), "Foreign key should exist");
        let fk_row = &fks[0];
        let table: String = fk_row.try_get("", "table")?;
        let from_col: String = fk_row.try_get("", "from")?;
        let to_col: String = fk_row.try_get("", "to")?;
        let on_delete: String = fk_row.try_get("", "on_delete")?;
        assert_eq!(table, "profiles");
        assert_eq!(from_col, "profile_id");
        assert_eq!(to_col, "id");
        assert_eq!(on_delete, "SET NULL");

        txn.commit().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_index_preservation() -> Result<(), DbErr> {
        let db = setup_test_db().await?;
        create_test_tables(&db).await?;
        insert_test_data(&db).await?;

        let txn = db.begin().await?;
        let manager = SchemaManager::new(&txn);

        // Add an index to the users table
        manager
            .create_index(
                Index::create()
                    .name("idx-users-name")
                    .table(Users::Table)
                    .col(Users::Name)
                    .to_owned(),
            )
            .await?;

        // Verify index exists
        let indexes_before = txn
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "PRAGMA index_list('users')".to_string(),
            ))
            .await?;
        let index_names_before: HashSet<String> = indexes_before
            .iter()
            .map(|row| row.try_get("", "name").unwrap_or_default())
            .collect();
        assert!(index_names_before.contains("idx-users-name"));

        // Add foreign key
        AddForeignKeyBuilder::create()
            .from(Users::Table)
            .to(Profiles::Table)
            .with_column(Users::ProfileId, Profiles::Id)
            .execute(&manager, Users::Table)
            .await?;

        // Verify index still exists after foreign key addition
        let indexes_after = txn
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "PRAGMA index_list('users')".to_string(),
            ))
            .await?;
        let index_names_after: HashSet<String> = indexes_after
            .iter()
            .map(|row| row.try_get("", "name").unwrap_or_default())
            .collect();
        assert!(index_names_after.contains("idx-users-name"));

        // Verify data integrity
        verify_exact_data_integrity(&txn).await?;

        txn.commit().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_multiple_foreign_keys_same_table() -> Result<(), DbErr> {
        let db = setup_test_db().await?;
        create_test_tables(&db).await?;
        insert_test_data(&db).await?;

        let txn = db.begin().await?;
        let manager = SchemaManager::new(&txn);

        // Add multiple foreign keys to the posts table
        AddForeignKeyBuilder::create()
            .from(Posts::Table)
            .to(Users::Table)
            .with_column(Posts::UserId, Users::Id)
            .on_delete(ForeignKeyAction::Cascade)
            .execute(&manager, Posts::Table)
            .await?;

        // Add another table and FK to test complex scenarios
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("comments"))
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Alias::new("content")).string().not_null())
                    .col(ColumnDef::new(Alias::new("post_id")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("user_id")).integer().null())
                    .to_owned(),
            )
            .await?;

        AddForeignKeyBuilder::create()
            .from(Alias::new("comments"))
            .to(Posts::Table)
            .with_column(Alias::new("post_id"), Posts::Id)
            .on_delete(ForeignKeyAction::Cascade)
            .execute(&manager, Alias::new("comments"))
            .await?;

        AddForeignKeyBuilder::create()
            .from(Alias::new("comments"))
            .to(Users::Table)
            .with_column(Alias::new("user_id"), Users::Id)
            .on_delete(ForeignKeyAction::SetNull)
            .execute(&manager, Alias::new("comments"))
            .await?;

        // Verify foreign keys exist
        let posts_fks = txn
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "PRAGMA foreign_key_list('posts')".to_string(),
            ))
            .await?;
        assert_eq!(posts_fks.len(), 1);

        let comments_fks = txn
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "PRAGMA foreign_key_list('comments')".to_string(),
            ))
            .await?;
        assert_eq!(comments_fks.len(), 2);

        // Verify data integrity
        verify_exact_data_integrity(&txn).await?;

        txn.commit().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_add_foreign_key_builder_default_values() -> Result<(), DbErr> {
        let db = setup_test_db().await?;
        create_test_tables(&db).await?;
        let txn = db.begin().await?;
        let manager = SchemaManager::new(&txn);

        // Test default values (should be NO ACTION)
        AddForeignKeyBuilder::create()
            .from(Users::Table)
            .to(Profiles::Table)
            .with_column(Users::ProfileId, Profiles::Id)
            .execute(&manager, Users::Table)
            .await?;

        // Verify default actions
        let fks = txn
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "PRAGMA foreign_key_list('users')".to_string(),
            ))
            .await?;

        assert!(!fks.is_empty());
        let fk_row = &fks[0];
        let on_update: String = fk_row.try_get("", "on_update")?;
        let on_delete: String = fk_row.try_get("", "on_delete")?;

        assert_eq!(on_update, "NO ACTION");
        assert_eq!(on_delete, "NO ACTION");

        txn.commit().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_error_handling_nonexistent_table() -> Result<(), DbErr> {
        let db = setup_test_db().await?;
        let manager = SchemaManager::new(&db);

        // Try to add FK to nonexistent table
        let result = AddForeignKeyBuilder::create()
            .from(Alias::new("nonexistent"))
            .to(Users::Table)
            .with_column(Alias::new("user_id"), Users::Id)
            .execute(&manager, Alias::new("nonexistent"))
            .await;

        assert!(result.is_err());

        // Try to drop FK from nonexistent table
        let result = DropForeignKeyBuilder::create()
            .from_table(Alias::new("nonexistent"))
            .to(Users::Table)
            .with_column(Alias::new("user_id"), Users::Id)
            .execute(&manager, Alias::new("nonexistent"))
            .await;

        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_error_handling_nonexistent_column() -> Result<(), DbErr> {
        let db = setup_test_db().await?;
        create_test_tables(&db).await?;

        let manager = SchemaManager::new(&db);

        // Try to add FK with nonexistent column
        let result = AddForeignKeyBuilder::create()
            .from(Users::Table)
            .to(Profiles::Table)
            .with_column(Alias::new("nonexistent_column"), Profiles::Id)
            .execute(&manager, Users::Table)
            .await;

        assert!(result.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_large_dataset_preservation() -> Result<(), DbErr> {
        let db = setup_test_db().await?;
        create_test_tables(&db).await?;
        let txn = db.begin().await?;

        // Insert more test data
        for i in 1..=100 {
            txn.execute_unprepared(&format!("INSERT INTO profiles (bio) VALUES ('Bio {}')", i))
                .await?;

            txn.execute_unprepared(&format!(
                "INSERT INTO users (name, email, profile_id) VALUES ('User {}', 'user{}@example.com', {})",
                i, i, i
            ))
            .await?;
        }

        let manager = SchemaManager::new(&txn);

        // Add foreign key
        AddForeignKeyBuilder::create()
            .from(Users::Table)
            .to(Profiles::Table)
            .with_column(Users::ProfileId, Profiles::Id)
            .execute(&manager, Users::Table)
            .await?;

        // Verify all data is preserved
        let user_count_result = txn
            .query_one(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT COUNT(*) as count FROM users".to_string(),
            ))
            .await?
            .expect("User count should exist");
        let user_count: i64 = user_count_result.try_get("", "count")?;
        assert_eq!(user_count, 100);

        let profile_count_result = txn
            .query_one(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT COUNT(*) as count FROM profiles".to_string(),
            ))
            .await?
            .expect("Profile count should exist");
        let profile_count: i64 = profile_count_result.try_get("", "count")?;
        assert_eq!(profile_count, 100);

        txn.commit().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_multi_column_foreign_key() -> Result<(), DbErr> {
        let db = setup_test_db().await?;
        let txn = db.begin().await?;
        let manager = SchemaManager::new(&txn);

        #[derive(Iden)]
        enum Orders {
            Table,
            TenantId,
            OrderId,
            Details,
        }

        #[derive(Iden)]
        enum OrderItems {
            Table,
            ItemId,
            TenantId,
            OrderId,
            Description,
        }

        // 1. Create tables with composite PK
        manager
            .create_table(
                Table::create()
                    .table(Orders::Table)
                    .col(ColumnDef::new(Orders::TenantId).integer().not_null())
                    .col(ColumnDef::new(Orders::OrderId).integer().not_null())
                    .col(ColumnDef::new(Orders::Details).string())
                    .primary_key(Index::create().col(Orders::TenantId).col(Orders::OrderId))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(OrderItems::Table)
                    .col(
                        ColumnDef::new(OrderItems::ItemId)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(OrderItems::TenantId).integer().not_null())
                    .col(ColumnDef::new(OrderItems::OrderId).integer().not_null())
                    .col(ColumnDef::new(OrderItems::Description).string())
                    .to_owned(),
            )
            .await?;

        // 2. Insert data
        txn.execute_unprepared(
            "INSERT INTO orders (tenant_id, order_id, details) VALUES (1, 100, 'Order A'), (2, 100, 'Order B')",
        )
        .await?;
        txn.execute_unprepared(
            "INSERT INTO order_items (tenant_id, order_id, description) VALUES (1, 100, 'Item 1 for A'), (2, 100, 'Item 2 for B')",
        )
        .await?;

        // 3. Add multi-column FK
        AddForeignKeyBuilder::create()
            .from(OrderItems::Table)
            .to(Orders::Table)
            .with_columns(vec![
                (OrderItems::TenantId, Orders::TenantId),
                (OrderItems::OrderId, Orders::OrderId),
            ])
            .on_delete(ForeignKeyAction::Cascade)
            .execute(&manager, OrderItems::Table)
            .await?;

        // 4. Verify data preservation
        let items: Vec<QueryResult> = txn
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT item_id FROM order_items".to_string(),
            ))
            .await?;
        assert_eq!(items.len(), 2);

        // 5. Verify FK creation
        let fks: Vec<QueryResult> = txn
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "PRAGMA foreign_key_list('order_items')".to_string(),
            ))
            .await?;
        assert_eq!(
            fks.len(),
            2,
            "Should have two entries for one composite key"
        );
        // Ensure they belong to the same composite key
        let id_0: u32 = fks[0].try_get("", "id")?;
        let id_1: u32 = fks[1].try_get("", "id")?;
        assert_eq!(id_0, id_1);

        // 6. Test constraint enforcement
        let result = txn
            .execute_unprepared(
                "INSERT INTO order_items (tenant_id, order_id, description) VALUES (3, 300, 'Invalid Item')",
            )
            .await;
        assert!(
            result.is_err(),
            "FK should prevent insertion of orphaned items"
        );

        // 7. Drop the multi-column FK
        DropForeignKeyBuilder::create()
            .from_table(OrderItems::Table)
            .to(Orders::Table)
            .with_columns(vec![
                (OrderItems::TenantId, Orders::TenantId),
                (OrderItems::OrderId, Orders::OrderId),
            ])
            .execute(&manager, OrderItems::Table)
            .await?;

        // 8. Verify FK was dropped
        let fks_after_drop = txn
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "PRAGMA foreign_key_list('order_items')".to_string(),
            ))
            .await?;
        assert!(
            fks_after_drop.is_empty(),
            "Multi-column FK should have been dropped"
        );

        txn.commit().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_self_referencing_foreign_key() -> Result<(), DbErr> {
        let db = setup_test_db().await?;
        let txn = db.begin().await?;
        let manager = SchemaManager::new(&txn);

        #[derive(Iden)]
        enum Employees {
            Table,
            Id,
            Name,
            ManagerId,
        }

        // 1. Create table
        manager
            .create_table(
                Table::create()
                    .table(Employees::Table)
                    .col(
                        ColumnDef::new(Employees::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Employees::Name).string().not_null())
                    .col(ColumnDef::new(Employees::ManagerId).integer().null())
                    .to_owned(),
            )
            .await?;

        // 2. Insert data
        txn.execute_unprepared(
            "INSERT INTO employees (id, name, manager_id) VALUES (1, 'CEO', NULL), (2, 'Manager', 1), (3, 'Worker', 2)",
        )
        .await?;

        // 3. Add self-referencing FK
        AddForeignKeyBuilder::create()
            .from(Employees::Table)
            .to(Employees::Table)
            .with_column(Employees::ManagerId, Employees::Id)
            .on_delete(ForeignKeyAction::SetNull)
            .execute(&manager, Employees::Table)
            .await?;

        // 4. Verify data preservation
        let results = txn
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT id, name, manager_id FROM employees ORDER BY id".to_string(),
            ))
            .await?;
        assert_eq!(results.len(), 3);
        let ceo_manager: Option<i32> = results[0].try_get("", "manager_id")?;
        assert_eq!(ceo_manager, None);
        let manager_manager: Option<i32> = results[1].try_get("", "manager_id")?;
        assert_eq!(manager_manager, Some(1));

        // 5. Verify FK creation
        let fks = txn
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "PRAGMA foreign_key_list('employees')".to_string(),
            ))
            .await?;
        assert_eq!(fks.len(), 1);
        let to_table: String = fks[0].try_get("", "table")?;
        assert_eq!(to_table, "employees");
        let on_delete: String = fks[0].try_get("", "on_delete")?;
        assert_eq!(on_delete, "SET NULL");

        // 6. Test constraint enforcement
        let result = txn
            .execute_unprepared("INSERT INTO employees (name, manager_id) VALUES ('Rogue', 999)")
            .await;
        assert!(
            result.is_err(),
            "FK constraint should fail for invalid manager_id"
        );

        // 7. Drop the FK
        DropForeignKeyBuilder::create()
            .from_table(Employees::Table)
            .to(Employees::Table)
            .with_column(Employees::ManagerId, Employees::Id)
            .execute(&manager, Employees::Table)
            .await?;

        // 8. Verify data is still preserved
        let results_after_drop = txn
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT id, name, manager_id FROM employees ORDER BY id".to_string(),
            ))
            .await?;
        assert_eq!(results_after_drop.len(), 3);

        // 9. Verify FK was dropped
        let fks_after_drop = txn
            .query_all(Statement::from_string(
                DbBackend::Sqlite,
                "PRAGMA foreign_key_list('employees')".to_string(),
            ))
            .await?;
        assert!(fks_after_drop.is_empty(), "FK should have been dropped");

        txn.commit().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_null_values_preservation() -> Result<(), DbErr> {
        let db = setup_test_db().await?;
        create_test_tables(&db).await?;

        // Insert data with NULL values
        db.execute_unprepared("INSERT INTO profiles (bio) VALUES ('Bio 1'), ('Bio 2')")
            .await?;

        db.execute_unprepared(
            "INSERT INTO users (name, email, profile_id) VALUES 
             ('Alice', 'alice@example.com', 1),
             ('Bob', 'bob@example.com', NULL),
             ('Charlie', 'charlie@example.com', 2)",
        )
        .await?;

        let txn = db.begin().await?;
        let manager = SchemaManager::new(&txn);

        // Add foreign key
        AddForeignKeyBuilder::create()
            .from(Users::Table)
            .to(Profiles::Table)
            .with_column(Users::ProfileId, Profiles::Id)
            .on_delete(ForeignKeyAction::SetNull)
            .execute(&manager, Users::Table)
            .await?;

        // Verify NULL values are preserved
        let null_check_result = txn
            .query_one(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT COUNT(*) as null_count FROM users WHERE profile_id IS NULL".to_string(),
            ))
            .await?;
        assert!(null_check_result.is_some());
        let null_count: i64 = null_check_result.unwrap().try_get("", "null_count")?;
        assert_eq!(null_count, 1);

        let not_null_check_result = txn
            .query_one(Statement::from_string(
                DbBackend::Sqlite,
                "SELECT COUNT(*) as not_null_count FROM users WHERE profile_id IS NOT NULL"
                    .to_string(),
            ))
            .await?;
        assert!(not_null_check_result.is_some());
        let not_null_count: i64 = not_null_check_result
            .unwrap()
            .try_get("", "not_null_count")?;
        assert_eq!(not_null_count, 2);

        txn.commit().await?;

        Ok(())
    }
}
