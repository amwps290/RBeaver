use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::collections::HashMap;

/// 数据库对象类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DatabaseObjectType {
    Schema,
    Extension,
    Table,
    View,
    Index,
    Type,
    Function,
    Procedure,
    Sequence,
    Trigger,
}

impl DatabaseObjectType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DatabaseObjectType::Schema => "schema",
            DatabaseObjectType::Extension => "extension",
            DatabaseObjectType::Table => "table",
            DatabaseObjectType::View => "view",
            DatabaseObjectType::Index => "index",
            DatabaseObjectType::Type => "type",
            DatabaseObjectType::Function => "function",
            DatabaseObjectType::Procedure => "procedure",
            DatabaseObjectType::Sequence => "sequence",
            DatabaseObjectType::Trigger => "trigger",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            DatabaseObjectType::Schema => "Schemas",
            DatabaseObjectType::Extension => "Extensions",
            DatabaseObjectType::Table => "Tables",
            DatabaseObjectType::View => "Views",
            DatabaseObjectType::Index => "Indexes",
            DatabaseObjectType::Type => "Types",
            DatabaseObjectType::Function => "Functions",
            DatabaseObjectType::Procedure => "Procedures",
            DatabaseObjectType::Sequence => "Sequences",
            DatabaseObjectType::Trigger => "Triggers",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            DatabaseObjectType::Schema => "folder",
            DatabaseObjectType::Extension => "package",
            DatabaseObjectType::Table => "table",
            DatabaseObjectType::View => "eye",
            DatabaseObjectType::Index => "search",
            DatabaseObjectType::Type => "type",
            DatabaseObjectType::Function => "function",
            DatabaseObjectType::Procedure => "cog",
            DatabaseObjectType::Sequence => "hash",
            DatabaseObjectType::Trigger => "zap",
        }
    }
}

/// 数据库对象信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseObject {
    pub object_type: DatabaseObjectType,
    pub schema: String,
    pub name: String,
    pub owner: Option<String>,
    pub comment: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl DatabaseObject {
    pub fn new(object_type: DatabaseObjectType, schema: String, name: String) -> Self {
        Self {
            object_type,
            schema,
            name,
            owner: None,
            comment: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_owner(mut self, owner: String) -> Self {
        self.owner = Some(owner);
        self
    }

    pub fn with_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn full_name(&self) -> String {
        if self.schema.is_empty() || self.schema == "public" {
            self.name.clone()
        } else {
            format!("{}.{}", self.schema, self.name)
        }
    }
}

/// 表结构信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbTableInfo {
    pub schema: String,
    pub name: String,
    pub owner: String,
    pub table_type: String, // BASE TABLE, VIEW, etc.
    pub has_indexes: bool,
    pub has_rules: bool,
    pub has_triggers: bool,
    pub row_count: Option<i64>,
    pub size_bytes: Option<i64>,
    pub comment: Option<String>,
}

/// 列信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbColumnInfo {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub default_value: Option<String>,
    pub is_primary_key: bool,
    pub is_foreign_key: bool,
    pub comment: Option<String>,
    pub ordinal_position: i32,
    pub character_maximum_length: Option<i32>,
}

/// 索引信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbIndexInfo {
    pub schema: String,
    pub table_name: String,
    pub index_name: String,
    pub is_unique: bool,
    pub is_primary: bool,
    pub columns: Vec<String>,
    pub index_type: String, // btree, hash, gin, etc.
}

/// 函数信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbFunctionInfo {
    pub schema: String,
    pub name: String,
    pub return_type: String,
    pub parameters: Vec<FunctionParameter>,
    pub language: String,
    pub is_aggregate: bool,
    pub is_trigger: bool,
    pub comment: Option<String>,
}

/// 函数参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionParameter {
    pub name: String,
    pub data_type: String,
    pub mode: String, // IN, OUT, INOUT
    pub default_value: Option<String>,
}

/// 类型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbTypeInfo {
    pub schema: String,
    pub name: String,
    pub type_category: String, // composite, enum, base, etc.
    pub owner: String,
    pub comment: Option<String>,
}

/// 扩展信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbExtensionInfo {
    pub name: String,
    pub version: String,
    pub schema: String,
    pub comment: Option<String>,
    pub installed: bool,
}

/// 数据库结构查询器
pub struct DatabaseStructureQuery;

impl DatabaseStructureQuery {
    /// 获取所有schema
    pub async fn get_schemas(pool: &PgPool) -> Result<Vec<DatabaseObject>> {
        let sql = r#"
            SELECT
                schema_name,
                schema_owner
            FROM information_schema.schemata
            WHERE schema_name NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
            ORDER BY schema_name
        "#;

        let rows = sqlx::query(sql).fetch_all(pool).await?;
        let mut schemas = Vec::new();

        for row in rows {
            let schema_name: String = row.get("schema_name");
            let schema_owner: String = row.get("schema_owner");

            let schema =
                DatabaseObject::new(DatabaseObjectType::Schema, String::new(), schema_name)
                    .with_owner(schema_owner);

            schemas.push(schema);
        }

        Ok(schemas)
    }

    /// 获取所有扩展
    pub async fn get_extensions(pool: &PgPool) -> Result<Vec<DbExtensionInfo>> {
        let sql = r#"
            SELECT
                extname as name,
                extversion as version,
                nspname as schema,
                obj_description(e.oid, 'pg_extension') as comment
            FROM pg_extension e
            JOIN pg_namespace n ON n.oid = e.extnamespace
            ORDER BY extname
        "#;

        let rows = sqlx::query(sql).fetch_all(pool).await?;
        let mut extensions = Vec::new();

        for row in rows {
            let extension = DbExtensionInfo {
                name: row.get("name"),
                version: row.get("version"),
                schema: row.get("schema"),
                comment: row.try_get("comment").ok(),
                installed: true,
            };
            extensions.push(extension);
        }

        Ok(extensions)
    }

    /// 获取表信息
    pub async fn get_tables(pool: &PgPool, schema: Option<&str>) -> Result<Vec<DbTableInfo>> {
        let sql = if let Some(schema) = schema {
            format!(
                r#"
                SELECT
                    t.table_schema as schema,
                    t.table_name as name,
                    c.table_owner as owner,
                    t.table_type,
                    CASE WHEN i.table_name IS NOT NULL THEN true ELSE false END as has_indexes,
                    CASE WHEN r.table_name IS NOT NULL THEN true ELSE false END as has_rules,
                    CASE WHEN tr.table_name IS NOT NULL THEN true ELSE false END as has_triggers,
                    obj_description(c.oid, 'pg_class') as comment
                FROM information_schema.tables t
                LEFT JOIN pg_class c ON c.relname = t.table_name
                LEFT JOIN pg_namespace n ON n.oid = c.relnamespace AND n.nspname = t.table_schema
                LEFT JOIN (
                    SELECT DISTINCT table_name, table_schema
                    FROM information_schema.statistics
                ) i ON i.table_name = t.table_name AND i.table_schema = t.table_schema
                LEFT JOIN (
                    SELECT DISTINCT tablename as table_name, schemaname as table_schema
                    FROM pg_rules
                ) r ON r.table_name = t.table_name AND r.table_schema = t.table_schema
                LEFT JOIN (
                    SELECT DISTINCT event_object_table as table_name, event_object_schema as table_schema
                    FROM information_schema.triggers
                ) tr ON tr.table_name = t.table_name AND tr.table_schema = t.table_schema
                WHERE t.table_schema = '{}' AND t.table_type IN ('BASE TABLE', 'VIEW')
                ORDER BY t.table_name
            "#,
                schema
            )
        } else {
            r#"
                SELECT
                    t.table_schema as schema,
                    t.table_name as name,
                    '' as owner,
                    t.table_type,
                    false as has_indexes,
                    false as has_rules,
                    false as has_triggers,
                    '' as comment
                FROM information_schema.tables t
                WHERE t.table_schema NOT IN ('information_schema', 'pg_catalog')
                ORDER BY t.table_schema, t.table_name
            "#
            .to_string()
        };

        let rows = sqlx::query(&sql).fetch_all(pool).await?;
        let mut tables = Vec::new();

        for row in rows {
            let table = DbTableInfo {
                schema: row.get("schema"),
                name: row.get("name"),
                owner: row.try_get("owner").unwrap_or_default(),
                table_type: row.get("table_type"),
                has_indexes: row.try_get("has_indexes").unwrap_or(false),
                has_rules: row.try_get("has_rules").unwrap_or(false),
                has_triggers: row.try_get("has_triggers").unwrap_or(false),
                row_count: None,  // Will be populated separately if needed
                size_bytes: None, // Will be populated separately if needed
                comment: row.try_get("comment").ok(),
            };
            tables.push(table);
        }

        Ok(tables)
    }

    /// 获取表的列信息
    pub async fn get_columns(
        pool: &PgPool,
        schema: &str,
        table: &str,
    ) -> Result<Vec<DbColumnInfo>> {
        let sql = r#"
            SELECT
                c.column_name,
                c.data_type,
                c.is_nullable = 'YES' as is_nullable,
                c.column_default,
                c.ordinal_position,
                c.character_maximum_length,
                CASE WHEN pk.column_name IS NOT NULL THEN true ELSE false END as is_primary_key,
                CASE WHEN fk.column_name IS NOT NULL THEN true ELSE false END as is_foreign_key,
                col_description(pgc.oid, c.ordinal_position) as comment
            FROM information_schema.columns c
            LEFT JOIN pg_class pgc ON pgc.relname = c.table_name
            LEFT JOIN pg_namespace pgn ON pgn.oid = pgc.relnamespace AND pgn.nspname = c.table_schema
            LEFT JOIN (
                SELECT ku.column_name, ku.table_name, ku.table_schema
                FROM information_schema.table_constraints tc
                JOIN information_schema.key_column_usage ku ON tc.constraint_name = ku.constraint_name
                WHERE tc.constraint_type = 'PRIMARY KEY'
            ) pk ON pk.column_name = c.column_name AND pk.table_name = c.table_name AND pk.table_schema = c.table_schema
            LEFT JOIN (
                SELECT ku.column_name, ku.table_name, ku.table_schema
                FROM information_schema.table_constraints tc
                JOIN information_schema.key_column_usage ku ON tc.constraint_name = ku.constraint_name
                WHERE tc.constraint_type = 'FOREIGN KEY'
            ) fk ON fk.column_name = c.column_name AND fk.table_name = c.table_name AND fk.table_schema = c.table_schema
            WHERE c.table_schema = $1 AND c.table_name = $2
            ORDER BY c.ordinal_position
        "#;

        let rows = sqlx::query(sql)
            .bind(schema)
            .bind(table)
            .fetch_all(pool)
            .await?;

        let mut columns = Vec::new();
        for row in rows {
            let column = DbColumnInfo {
                name: row.get("column_name"),
                data_type: row.get("data_type"),
                is_nullable: row.get("is_nullable"),
                default_value: row.try_get("column_default").ok(),
                is_primary_key: row.get("is_primary_key"),
                is_foreign_key: row.get("is_foreign_key"),
                comment: row.try_get("comment").ok(),
                ordinal_position: row.get("ordinal_position"),
                character_maximum_length: row.try_get("character_maximum_length").ok(),
            };
            columns.push(column);
        }

        Ok(columns)
    }

    /// 获取索引信息
    pub async fn get_indexes(pool: &PgPool, schema: Option<&str>) -> Result<Vec<DbIndexInfo>> {
        let sql = if let Some(schema) = schema {
            format!(
                r#"
                SELECT
                    schemaname as schema,
                    tablename as table_name,
                    indexname as index_name,
                    indexdef,
                    CASE WHEN indexdef LIKE '%UNIQUE%' THEN true ELSE false END as is_unique,
                    CASE WHEN indexname LIKE '%pkey' THEN true ELSE false END as is_primary
                FROM pg_indexes
                WHERE schemaname = '{}'
                ORDER BY tablename, indexname
            "#,
                schema
            )
        } else {
            r#"
                SELECT
                    schemaname as schema,
                    tablename as table_name,
                    indexname as index_name,
                    indexdef,
                    CASE WHEN indexdef LIKE '%UNIQUE%' THEN true ELSE false END as is_unique,
                    CASE WHEN indexname LIKE '%pkey' THEN true ELSE false END as is_primary
                FROM pg_indexes
                WHERE schemaname NOT IN ('information_schema', 'pg_catalog')
                ORDER BY schemaname, tablename, indexname
            "#
            .to_string()
        };

        let rows = sqlx::query(&sql).fetch_all(pool).await?;
        let mut indexes = Vec::new();

        for row in rows {
            let index = DbIndexInfo {
                schema: row.get("schema"),
                table_name: row.get("table_name"),
                index_name: row.get("index_name"),
                is_unique: row.get("is_unique"),
                is_primary: row.get("is_primary"),
                columns: Vec::new(), // Will be parsed from indexdef if needed
                index_type: "btree".to_string(), // Default, can be enhanced
            };
            indexes.push(index);
        }

        Ok(indexes)
    }

    /// 获取函数信息
    pub async fn get_functions(pool: &PgPool, schema: Option<&str>) -> Result<Vec<DbFunctionInfo>> {
        let sql = if let Some(schema) = schema {
            format!(
                r#"
                SELECT
                    n.nspname as schema,
                    p.proname as name,
                    pg_get_function_result(p.oid) as return_type,
                    l.lanname as language,
                    p.proisagg as is_aggregate,
                    p.prorettype = 'trigger'::regtype::oid as is_trigger,
                    obj_description(p.oid, 'pg_proc') as comment
                FROM pg_proc p
                JOIN pg_namespace n ON p.pronamespace = n.oid
                JOIN pg_language l ON p.prolang = l.oid
                WHERE n.nspname = '{}'
                ORDER BY p.proname
            "#,
                schema
            )
        } else {
            r#"
                SELECT
                    n.nspname as schema,
                    p.proname as name,
                    pg_get_function_result(p.oid) as return_type,
                    l.lanname as language,
                    p.proisagg as is_aggregate,
                    p.prorettype = 'trigger'::regtype::oid as is_trigger,
                    obj_description(p.oid, 'pg_proc') as comment
                FROM pg_proc p
                JOIN pg_namespace n ON p.pronamespace = n.oid
                JOIN pg_language l ON p.prolang = l.oid
                WHERE n.nspname NOT IN ('information_schema', 'pg_catalog')
                ORDER BY n.nspname, p.proname
            "#
            .to_string()
        };

        let rows = sqlx::query(&sql).fetch_all(pool).await?;
        let mut functions = Vec::new();

        for row in rows {
            let function = DbFunctionInfo {
                schema: row.get("schema"),
                name: row.get("name"),
                return_type: row.get("return_type"),
                parameters: Vec::new(), // Can be enhanced to parse parameters
                language: row.get("language"),
                is_aggregate: row.get("is_aggregate"),
                is_trigger: row.get("is_trigger"),
                comment: row.try_get("comment").ok(),
            };
            functions.push(function);
        }

        Ok(functions)
    }

    /// 获取自定义类型信息
    pub async fn get_types(pool: &PgPool, schema: Option<&str>) -> Result<Vec<DbTypeInfo>> {
        let sql = if let Some(schema) = schema {
            format!(
                r#"
                SELECT
                    n.nspname as schema,
                    t.typname as name,
                    CASE t.typtype
                        WHEN 'c' THEN 'composite'
                        WHEN 'e' THEN 'enum'
                        WHEN 'b' THEN 'base'
                        WHEN 'd' THEN 'domain'
                        ELSE 'unknown'
                    END as type_category,
                    r.rolname as owner,
                    obj_description(t.oid, 'pg_type') as comment
                FROM pg_type t
                JOIN pg_namespace n ON t.typnamespace = n.oid
                JOIN pg_roles r ON t.typowner = r.oid
                WHERE n.nspname = '{}' AND t.typtype IN ('c', 'e', 'd')
                ORDER BY t.typname
            "#,
                schema
            )
        } else {
            r#"
                SELECT
                    n.nspname as schema,
                    t.typname as name,
                    CASE t.typtype
                        WHEN 'c' THEN 'composite'
                        WHEN 'e' THEN 'enum'
                        WHEN 'b' THEN 'base'
                        WHEN 'd' THEN 'domain'
                        ELSE 'unknown'
                    END as type_category,
                    r.rolname as owner,
                    obj_description(t.oid, 'pg_type') as comment
                FROM pg_type t
                JOIN pg_namespace n ON t.typnamespace = n.oid
                JOIN pg_roles r ON t.typowner = r.oid
                WHERE n.nspname NOT IN ('information_schema', 'pg_catalog') AND t.typtype IN ('c', 'e', 'd')
                ORDER BY n.nspname, t.typname
            "#.to_string()
        };

        let rows = sqlx::query(&sql).fetch_all(pool).await?;
        let mut types = Vec::new();

        for row in rows {
            let type_info = DbTypeInfo {
                schema: row.get("schema"),
                name: row.get("name"),
                type_category: row.get("type_category"),
                owner: row.get("owner"),
                comment: row.try_get("comment").ok(),
            };
            types.push(type_info);
        }

        Ok(types)
    }
}

/// 数据库结构树节点
#[derive(Debug, Clone)]
pub struct DatabaseTreeNode {
    pub id: String,
    pub name: String,
    pub node_type: DatabaseObjectType,
    pub is_expanded: bool,
    pub children: Vec<DatabaseTreeNode>,
    pub metadata: HashMap<String, String>,
}

impl DatabaseTreeNode {
    pub fn new(id: String, name: String, node_type: DatabaseObjectType) -> Self {
        Self {
            id,
            name,
            node_type,
            is_expanded: false,
            children: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_children(mut self, children: Vec<DatabaseTreeNode>) -> Self {
        self.children = children;
        self
    }

    pub fn add_child(&mut self, child: DatabaseTreeNode) {
        self.children.push(child);
    }

    pub fn expand(&mut self) {
        self.is_expanded = true;
    }

    pub fn collapse(&mut self) {
        self.is_expanded = false;
    }

    pub fn toggle_expanded(&mut self) {
        self.is_expanded = !self.is_expanded;
    }
}
