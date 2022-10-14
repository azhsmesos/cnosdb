//! CatalogProvider:            ---> namespace
//! - SchemeProvider #1         ---> db
//!     - dyn tableProvider #1  ---> table
//!         - field #1
//!         - Column #2
//!     - dyn TableProvider #2
//!         - Column #3
//!         - Column #4

use std::any::Any;
use std::collections::HashMap;
use std::{collections::BTreeMap, sync::Arc};

use std::mem::size_of_val;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use datafusion::arrow::datatypes::{DataType as ArrowDataType, Field, Schema, SchemaRef, TimeUnit};
use datafusion::catalog::schema::SchemaProvider;
use datafusion::datasource::{TableProvider, TableType};
use datafusion::execution::context::SessionState;
use datafusion::logical_expr::Expr;
use datafusion::physical_plan::ExecutionPlan;

use crate::{SchemaFieldId, ValueType};

pub type TableSchemaRef = Arc<TableSchema>;

pub const TIME_FIELD_NAME: &str = "time";

pub const FIELD_ID: &str = "_field_id";
pub const TAG: &str = "_tag";
pub const TIME_FIELD: &str = "time";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TableSchema {
    pub db: String,
    pub name: String,
    pub schema_id: u32,
    pub fields: BTreeMap<String, TableFiled>,
}

impl Default for TableSchema {
    fn default() -> Self {
        Self {
            db: "public".to_string(),
            name: "".to_string(),
            schema_id: 0,
            fields: std::default::Default::default(),
        }
    }
}

impl TableSchema {
    pub fn to_arrow_schema(&self) -> SchemaRef {
        let fields: Vec<Field> = self
            .fields
            .iter()
            .map(|(name, schema)| {
                let mut f = Field::new(name, schema.column_type.into(), true);
                let mut map = BTreeMap::new();
                map.insert(FIELD_ID.to_string(), schema.id.to_string());
                map.insert(TAG.to_string(), schema.column_type.is_tag().to_string());
                f.set_metadata(Some(map));
                f
            })
            .collect();

        Arc::new(Schema::new(fields))
    }

    pub fn new(db: String, name: String, fields: BTreeMap<String, TableFiled>) -> Self {
        Self {
            db,
            name,
            schema_id: 0,
            fields,
        }
    }
    pub fn fields(&self) -> &BTreeMap<String, TableFiled> {
        &self.fields
    }

    pub fn field_fields_num(&self) -> usize {
        let mut ans = 0;
        for i in self.fields.iter() {
            if i.1.column_type != ColumnType::Tag && i.1.column_type != ColumnType::Time {
                ans += 1;
            }
        }
        ans
    }

    // return (table_field_id, index), index mean field location which column
    pub fn fields_id(&self) -> HashMap<SchemaFieldId, usize> {
        let mut ans = vec![];
        for i in self.fields.iter() {
            if i.1.column_type != ColumnType::Tag && i.1.column_type != ColumnType::Time {
                ans.push(i.1.id);
            }
        }
        ans.sort();
        let mut map = HashMap::new();
        for (i, id) in ans.iter().enumerate() {
            map.insert(*id, i);
        }
        map
    }

    pub fn size(&self) -> usize {
        let mut size = 0;
        for i in self.fields.iter() {
            size += i.0.capacity() + size_of_val(&i.1) + size_of_val(&i);
        }
        size += size_of_val(&self);
        size
    }
}

#[async_trait]
impl TableProvider for TableSchema {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn schema(&self) -> SchemaRef {
        todo!()
    }

    fn table_type(&self) -> TableType {
        todo!()
    }

    async fn scan(
        &self,
        _ctx: &SessionState,
        _projection: &Option<Vec<usize>>,
        _filters: &[Expr],
        _limit: Option<usize>,
    ) -> datafusion::common::Result<Arc<dyn ExecutionPlan>> {
        todo!()
    }
}

pub fn is_time_column(field: &Field) -> bool {
    TIME_FIELD_NAME == field.name()
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TableFiled {
    pub id: SchemaFieldId,
    pub name: String,
    pub column_type: ColumnType,
    pub codec: u8,
}

impl TableFiled {
    pub fn new(id: SchemaFieldId, name: String, column_type: ColumnType, codec: u8) -> Self {
        Self {
            id,
            name,
            column_type,
            codec,
        }
    }
    pub fn new_with_default(name: String, column_type: ColumnType) -> Self {
        Self {
            id: 0,
            name,
            column_type,
            codec: 0,
        }
    }
    pub fn time_field(codec: u8) -> TableFiled {
        TableFiled {
            id: 0,
            name: TIME_FIELD_NAME.to_string(),
            column_type: ColumnType::Time,
            codec,
        }
    }
}

impl From<ColumnType> for ArrowDataType {
    fn from(t: ColumnType) -> Self {
        match t {
            ColumnType::Tag => Self::Utf8,
            ColumnType::Time => Self::Timestamp(TimeUnit::Nanosecond, None),
            ColumnType::Field(ValueType::Float) => Self::Float64,
            ColumnType::Field(ValueType::Integer) => Self::Int64,
            ColumnType::Field(ValueType::Unsigned) => Self::UInt64,
            ColumnType::Field(ValueType::String) => Self::Utf8,
            ColumnType::Field(ValueType::Boolean) => Self::Boolean,
            _ => Self::Null,
        }
    }
}

impl TryFrom<ArrowDataType> for ColumnType {
    type Error = &'static str;

    fn try_from(value: ArrowDataType) -> Result<Self, Self::Error> {
        match value {
            ArrowDataType::Float64 => Ok(Self::Field(ValueType::Float)),
            ArrowDataType::Int64 => Ok(Self::Field(ValueType::Integer)),
            ArrowDataType::UInt64 => Ok(Self::Field(ValueType::Unsigned)),
            ArrowDataType::Utf8 => Ok(Self::Field(ValueType::String)),
            ArrowDataType::Boolean => Ok(Self::Field(ValueType::Boolean)),
            _ => Err("Error field type not supported"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ColumnType {
    Tag,
    Time,
    Field(ValueType),
}

impl ColumnType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Tag => "tag",
            Self::Time => "time",
            Self::Field(ValueType::Integer) => "i64",
            Self::Field(ValueType::Unsigned) => "u64",
            Self::Field(ValueType::Float) => "f64",
            Self::Field(ValueType::Boolean) => "bool",
            Self::Field(ValueType::String) => "string",
            _ => "Error filed type not supported",
        }
    }
    pub fn field_type(&self) -> u8 {
        match self {
            Self::Field(ValueType::Float) => 0,
            Self::Field(ValueType::Integer) => 1,
            Self::Field(ValueType::Unsigned) => 2,
            Self::Field(ValueType::Boolean) => 3,
            Self::Field(ValueType::String) => 4,
            _ => 0,
        }
    }

    pub fn from_i32(field_type: i32) -> Self {
        match field_type {
            0 => Self::Field(ValueType::Float),
            1 => Self::Field(ValueType::Integer),
            2 => Self::Field(ValueType::Unsigned),
            3 => Self::Field(ValueType::Boolean),
            4 => Self::Field(ValueType::String),
            5 => Self::Time,
            _ => Self::Field(ValueType::Unknown),
        }
    }
}

impl std::fmt::Display for ColumnType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.as_str();
        write!(f, "{}", s)
    }
}

impl ColumnType {
    pub fn is_tag(&self) -> bool {
        matches!(self, ColumnType::Tag)
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseSchema {
    pub name: String,
    pub config: DatabaseOptions,
}

impl SchemaProvider for DatabaseSchema {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn table_names(&self) -> Vec<String> {
        todo!()
    }

    fn table(&self, _name: &str) -> Option<Arc<dyn TableProvider>> {
        todo!()
    }

    fn table_exist(&self, _name: &str) -> bool {
        todo!()
    }
}

impl DatabaseSchema {
    pub fn new(name: &str) -> Self {
        DatabaseSchema {
            name: name.to_string(),
            config: DatabaseOptions::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseOptions {
    // data keep time
    pub ttl: Duration,

    pub shard_num: u64,
    // shard coverage time range
    pub vnode_duration: Duration,

    pub replica: u64,
    // timestamp percision
    pub precision: Precision,
}

impl Default for DatabaseOptions {
    fn default() -> Self {
        Self {
            ttl: Duration {
                time_num: 365,
                unit: DurationUnit::Day,
            },
            shard_num: 1,
            vnode_duration: Duration {
                time_num: 365,
                unit: DurationUnit::Day,
            },
            replica: 1,
            precision: Precision::NS,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Precision {
    MS,
    US,
    NS,
}

impl Precision {
    pub fn new(text: &str) -> Option<Self> {
        match text.to_uppercase().as_str() {
            "'MS'" => Some(Precision::MS),
            "'US'" => Some(Precision::US),
            "'NS'" => Some(Precision::NS),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DurationUnit {
    Minutes,
    Hour,
    Day,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Duration {
    pub time_num: u64,
    pub unit: DurationUnit,
}

impl Duration {
    // text should have ', such as '10d', the len of '10d' is 5
    pub fn new(text: &str) -> Option<Self> {
        let len = text.len();
        if len < 4 {
            return None;
        }
        let time = &text[1..len - 2];
        let unit = &text[len - 2..len - 1];
        let time_num = match time.parse::<u64>() {
            Ok(v) => v,
            Err(_) => {
                return None;
            }
        };
        let time_unit = match unit.to_uppercase().as_str() {
            "D" => DurationUnit::Day,
            "H" => DurationUnit::Hour,
            "M" => DurationUnit::Minutes,
            _ => return None,
        };
        Some(Duration {
            time_num,
            unit: time_unit,
        })
    }
}