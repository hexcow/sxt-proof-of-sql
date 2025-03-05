use super::{table_reference_to_table_ref, PlannerResult};
use alloc::vec::Vec;
use datafusion::{
    common::DFSchema,
    logical_expr::{BinaryExpr, Expr, LogicalPlan, Operator, TableScan},
    sql::{sqlparser::ast::Ident, TableReference},
};
use indexmap::IndexMap;
use proof_of_sql::{
    base::database::{ColumnRef, ColumnType, LiteralValue},
    sql::{
        proof_exprs::{AliasedDynProofExpr, DynProofExpr, TableExpr},
        proof_plans::{DynProofPlan, EmptyExec, FilterExec},
    },
};

/// Visit a [`datafusion::logical_plan::LogicalPlan`] and return a [`DynProofPlan`]
pub fn logical_plan_to_proof_plan(
    plan: &LogicalPlan,
    schemas: &IndexMap<TableReference, DFSchema>,
) -> PlannerResult<DynProofPlan> {
    match plan {
        LogicalPlan::EmptyRelation { .. } => Ok(DynProofPlan::Empty(EmptyExec {})),
        LogicalPlan::TableScan(TableScan {
            table_name,
            projection,
            projected_schema,
            filters,
            ..
        }) => {
            // Check if the table exists
            let table_ref = table_reference_to_table_ref(table_name);
            let table_expr = TableExpr {
                table_ref: table_ref.clone(),
            };
            let input_schema =
                schemas
                    .get(table_name)
                    .ok_or_else(|| PlannerError::TableNotFound {
                        table_name: table_name.to_string(),
                    })?;
            // Get the aliased dyn proof exprs
            let num_input_columns = input_schema.columns().len();
            let projection_indexes = projection
                .clone()
                .unwrap_or_else(|| (0..num_input_columns).collect::<Vec<_>>());
            let aliased_dyn_proof_exprs = projection_indexes
                .iter()
                .enumerate()
                .map(
                    |(output_index, input_index)| -> PlannerResult<AliasedDynProofExpr> {
                        // Get output column name / alias
                        let alias: Ident =
                            projected_schema.field(output_index).name().as_str().into();
                        let input_column_name: Ident =
                            input_schema.field(*input_index).name().as_str().into();
                        let data_type = input_schema.field(*input_index).data_type();
                        let expr = DynProofExpr::new_column(ColumnRef::new(
                            table_ref.clone(),
                            input_column_name,
                            ColumnType::try_from(data_type.clone()).map_err(|_e| {
                                PlannerError::UnsupportedDataType {
                                    data_type: data_type.clone(),
                                }
                            })?,
                        ));
                        Ok(AliasedDynProofExpr { expr, alias })
                    },
                )
                .collect::<PlannerResult<Vec<_>>>()?;
            // Process filter
            let filter_proof_exprs = filters
                .iter()
                .map(|f| expr_to_proof_expr(f, input_schema))
                .collect::<PlannerResult<Vec<_>>>()?;
            let num_filters = filter_proof_exprs.len();
            // Filter
            let mut consolidated_filter_proof_expr = if num_filters == 0 {
                DynProofExpr::new_literal(LiteralValue::Boolean(true))
            } else {
                filter_proof_exprs[0].clone()
            };
            if num_filters > 0 {
                for i in 0..num_filters - 1 {
                    consolidated_filter_proof_expr = DynProofExpr::try_new_and(
                        consolidated_filter_proof_expr,
                        filter_proof_exprs[i + 1].clone(),
                    )?;
                }
            }
            Ok(DynProofPlan::Filter(FilterExec::new(
                aliased_dyn_proof_exprs,
                table_expr,
                consolidated_filter_proof_expr,
            )))
        }
        _ => Err(PlannerError::UnsupportedLogicalPlan { plan }),
    }
}
