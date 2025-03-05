use super::{AliasedDynProofExpr, ColumnExpr, DynProofExpr, TableExpr};
use crate::base::{
    database::{ColumnRef, ColumnType, LiteralValue, SchemaAccessor, TableRef},
    math::{decimal::Precision, i256::I256},
    scalar::Scalar,
};
use alloc::{string::ToString, vec::Vec};
use proof_of_sql_parser::intermediate_ast::AggregationOperator;
use sqlparser::ast::Ident;

/// Create a `ColumnRef` from namespace, table, column, and column type
#[must_use]
pub fn column_ref(
    namespace: Option<&str>,
    table: &str,
    column: &str,
    column_type: ColumnType,
) -> ColumnRef {
    ColumnRef::new(
        TableRef::from_names(namespace, table),
        column.into(),
        column_type,
    )
}

/// Create a `ColumnExpr` from namespace, table, column, and column type
#[must_use]
pub fn column_expr(
    namespace: Option<&str>,
    table: &str,
    column: &str,
    column_type: ColumnType,
) -> DynProofExpr {
    DynProofExpr::new_column(column_ref(namespace, table, column, column_type))
}

/// Get a `ColumnRef` from table ref, column name and accessor
///
/// # Panics
/// Panics if:
/// - `accessor.lookup_column()` returns `None`, indicating the column is not found.
pub fn col_ref(tab: &TableRef, name: &str, accessor: &impl SchemaAccessor) -> ColumnRef {
    let name: Ident = name.into();
    let type_col = accessor.lookup_column(tab.clone(), name.clone()).unwrap();
    ColumnRef::new(tab.clone(), name, type_col)
}

/// Create a `DynProofExpr::Column` from table ref, column name and accessor
///
/// # Panics
/// Panics if:
/// - `accessor.lookup_column()` returns `None`, indicating the column is not found.
pub fn column(tab: &TableRef, name: &str, accessor: &impl SchemaAccessor) -> DynProofExpr {
    let name: Ident = name.into();
    let type_col = accessor.lookup_column(tab.clone(), name.clone()).unwrap();
    DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(tab.clone(), name, type_col)))
}

/// A = B
///
/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_equals()` returns an error.
#[must_use]
pub fn equal(left: DynProofExpr, right: DynProofExpr) -> DynProofExpr {
    DynProofExpr::try_new_equals(left, right).unwrap()
}

/// A < B
///
/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_inequality()` returns an error.
#[must_use]
pub fn lt(left: DynProofExpr, right: DynProofExpr) -> DynProofExpr {
    DynProofExpr::try_new_inequality(left, right, true).unwrap()
}

/// A > B
///
/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_inequality()` returns an error.
#[must_use]
pub fn gt(left: DynProofExpr, right: DynProofExpr) -> DynProofExpr {
    DynProofExpr::try_new_inequality(left, right, false).unwrap()
}

/// A <= B
///
/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_inequality()` returns an error.
#[must_use]
pub fn lte(left: DynProofExpr, right: DynProofExpr) -> DynProofExpr {
    not(DynProofExpr::try_new_inequality(left, right, false).unwrap())
}

/// A >= B
///
/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_inequality()` returns an error.
#[must_use]
pub fn gte(left: DynProofExpr, right: DynProofExpr) -> DynProofExpr {
    not(DynProofExpr::try_new_inequality(left, right, true).unwrap())
}

/// NOT P
///
/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_not()` returns an error.
#[must_use]
pub fn not(expr: DynProofExpr) -> DynProofExpr {
    DynProofExpr::try_new_not(expr).unwrap()
}

/// P AND Q
///
/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_and()` returns an error.
#[must_use]
pub fn and(left: DynProofExpr, right: DynProofExpr) -> DynProofExpr {
    DynProofExpr::try_new_and(left, right).unwrap()
}

/// P OR Q
///
/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_or()` returns an error.
#[must_use]
pub fn or(left: DynProofExpr, right: DynProofExpr) -> DynProofExpr {
    DynProofExpr::try_new_or(left, right).unwrap()
}

/// A + B
///
/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_add()` returns an error.
#[must_use]
pub fn add(left: DynProofExpr, right: DynProofExpr) -> DynProofExpr {
    DynProofExpr::try_new_add(left, right).unwrap()
}

/// A - B
///
/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_subtract()` returns an error.
#[must_use]
pub fn subtract(left: DynProofExpr, right: DynProofExpr) -> DynProofExpr {
    DynProofExpr::try_new_subtract(left, right).unwrap()
}

/// A * B
///
/// # Panics
/// Panics if:
/// - `DynProofExpr::try_new_multiply()` returns an error.
#[must_use]
pub fn multiply(left: DynProofExpr, right: DynProofExpr) -> DynProofExpr {
    DynProofExpr::try_new_multiply(left, right).unwrap()
}

/// Get a `DynProofExpr::Literal` from a literal boolean value
#[must_use]
pub fn const_bool(val: bool) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::Boolean(val))
}

/// Get a `DynProofExpr::Literal` from a literal i8 value
#[must_use]
pub fn const_smallint(val: i16) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::SmallInt(val))
}

/// Get a `DynProofExpr::Literal` from a literal i32 value
#[must_use]
pub fn const_int(val: i32) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::Int(val))
}

/// Get a `DynProofExpr::Literal` from a literal i64 value
#[must_use]
pub fn const_bigint(val: i64) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::BigInt(val))
}

/// Get a `DynProofExpr::Literal` from a literal i128 value
#[must_use]
pub fn const_int128(val: i128) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::Int128(val))
}

/// Get a `DynProofExpr::Literal` from a literal &str
#[must_use]
pub fn const_varchar(val: &str) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::VarChar(val.to_string()))
}

/// Create a constant scalar value. Used if we don't want to specify column types.
pub fn const_scalar<S: Scalar, T: Into<S>>(val: T) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::Scalar(val.into().into()))
}

/// Get a `DynProofExpr::Literal` from precision, scale, and something that can be converted to i256
///
/// # Panics
/// Panics if:
/// - `Precision::new(precision)` fails, meaning the provided precision is invalid.
pub fn const_decimal75<T: Into<I256>>(precision: u8, scale: i8, val: T) -> DynProofExpr {
    DynProofExpr::new_literal(LiteralValue::Decimal75(
        Precision::new(precision).unwrap(),
        scale,
        val.into(),
    ))
}

/// Get a `TableExpr` from a `TableRef`
#[must_use]
pub fn tab(tab: &TableRef) -> TableExpr {
    TableExpr {
        table_ref: tab.clone(),
    }
}

/// Get an `AliasedDynProofExpr` from an expression and an alias
///
/// # Panics
/// Panics if:
/// - `alias.parse()` fails to parse the provided alias string.
#[must_use]
pub fn aliased_plan(expr: DynProofExpr, alias: &str) -> AliasedDynProofExpr {
    AliasedDynProofExpr {
        expr,
        alias: alias.into(),
    }
}

/// Get an `AliasedDynProofExpr` from a `TableRef`, column name, alias and accessor
///
/// # Panics
/// Panics if:
/// - `old_name.parse()` or `new_name.parse()` fails to parse the provided column names.
/// - `col_ref()` fails to find the referenced column, leading to a panic in the column accessor.
pub fn aliased_col_expr_plan(
    tab: &TableRef,
    old_name: &str,
    new_name: &str,
    accessor: &impl SchemaAccessor,
) -> AliasedDynProofExpr {
    AliasedDynProofExpr {
        expr: DynProofExpr::Column(ColumnExpr::new(col_ref(tab, old_name, accessor))),
        alias: new_name.into(),
    }
}

/// Get an `AliasedDynProofExpr` from a `TableRef`, column name and accessor
///
/// # Panics
/// Panics if:
/// - `name.parse()` fails to parse the provided column name.
/// - `col_ref()` fails to find the referenced column, leading to a panic in the column accessor.
pub fn col_expr_plan(
    tab: &TableRef,
    name: &str,
    accessor: &impl SchemaAccessor,
) -> AliasedDynProofExpr {
    AliasedDynProofExpr {
        expr: DynProofExpr::Column(ColumnExpr::new(col_ref(tab, name, accessor))),
        alias: name.into(),
    }
}

/// Get a vector of `AliasedDynProofExpr` from a `TableRef`, a vector of column name and alias pairs, and an accessor
pub fn aliased_cols_expr_plan(
    tab: &TableRef,
    names: &[(&str, &str)],
    accessor: &impl SchemaAccessor,
) -> Vec<AliasedDynProofExpr> {
    names
        .iter()
        .map(|(old_name, new_name)| aliased_col_expr_plan(tab, old_name, new_name, accessor))
        .collect()
}

/// Get a vector of `AliasedDynProofExpr` from a `TableRef`, a vector of column names, and an accessor
pub fn cols_expr_plan(
    tab: &TableRef,
    names: &[&str],
    accessor: &impl SchemaAccessor,
) -> Vec<AliasedDynProofExpr> {
    names
        .iter()
        .map(|name| col_expr_plan(tab, name, accessor))
        .collect()
}

/// Get a `ColumnExpr` from a `TableRef`, a column name and an accessor
pub fn col_expr(tab: &TableRef, name: &str, accessor: &impl SchemaAccessor) -> ColumnExpr {
    ColumnExpr::new(col_ref(tab, name, accessor))
}

/// Get a vector of `ColumnExpr` from a `TableRef`, a vector of column names, and an accessor
pub fn cols_expr(
    tab: &TableRef,
    names: &[&str],
    accessor: &impl SchemaAccessor,
) -> Vec<ColumnExpr> {
    names
        .iter()
        .map(|name| col_expr(tab, name, accessor))
        .collect()
}

/// SUM(expr) AS alias
///
/// # Panics
/// Panics if:
/// - `alias.parse()` fails to parse the provided alias string.
#[must_use]
pub fn sum_expr(expr: DynProofExpr, alias: &str) -> AliasedDynProofExpr {
    AliasedDynProofExpr {
        expr: DynProofExpr::new_aggregate(AggregationOperator::Sum, expr),
        alias: alias.into(),
    }
}
