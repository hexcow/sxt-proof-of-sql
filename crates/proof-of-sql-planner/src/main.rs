// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

//! This example shows how to use DataFusion's SQL planner to parse SQL text and
//! build, analyze and optimize `LogicalPlan`s, convert them to `ProofPlan`s, and provably execute them
use bumpalo::Bump;
use datafusion::{
    logical_expr::LogicalPlan,
    optimizer::{
        Analyzer, AnalyzerRule, Optimizer, OptimizerConfig, OptimizerContext, OptimizerRule,
    },
    sql::planner::SqlToRel,
};
use indexmap::indexmap;
use proof_of_sql::{
    base::{
        commitment::InnerProductProof,
        database::{
            owned_table_utility::*, table_utility::*, TableRef, TableTestAccessor, TestAccessor,
        },
    },
    sql::proof::VerifiableQueryResult,
};
use proof_of_sql_planner::{logical_plan_to_proof_plan, PlannerResult, PoSqlContextProvider};
use sqlparser::{dialect::GenericDialect, parser::Parser};

/// This example shows how to use DataFusion's SQL planner to parse SQL text and
/// build `LogicalPlan`s, convert them to `ProofPlan`s, and provably execute them.
pub fn main() -> PlannerResult<()> {
    // Create an `OwnedTableTestAccessor` with a single table
    let alloc = Bump::new();
    let tables = indexmap! {
        TableRef::new("sxt", "tab") =>
        table([
            borrowed_boolean("a", [true, false, true], &alloc),
            borrowed_tinyint("b", [1, 2, 3], &alloc),
            borrowed_varchar("c", ["Space", "and", "Time"], &alloc),
        ])
    };
    let mut accessor = TableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    for (table_ref, table) in tables.iter() {
        accessor.add_table(table_ref.clone(), table.clone(), 0);
    }

    let dialect = GenericDialect {};
    let sql = "SELECT a, c from sxt.tab where a and b > 1";
    let statements = Parser::parse_sql(&dialect, sql)?;
    let context_provider = PoSqlContextProvider::new(tables);
    let schemas = context_provider.try_get_df_schemas()?;
    let sql_to_rel = SqlToRel::new(&context_provider);
    let raw_plan = sql_to_rel.sql_statement_to_plan(statements[0].clone())?;
    let config = OptimizerContext::default().with_skip_failing_rules(false);
    let analyzed_plan =
        Analyzer::new().execute_and_check(raw_plan, config.options(), observe_analyzer)?;
    let optimized_plan = Optimizer::new().optimize(analyzed_plan, &config, observe_optimizer)?;
    let proof_plan = logical_plan_to_proof_plan(&optimized_plan, &schemas)?;
    let verifiable_res =
        VerifiableQueryResult::<InnerProductProof>::new(&proof_plan, &accessor, &());
    let res = verifiable_res
        .verify(&proof_plan, &accessor, &())
        .unwrap()
        .table;
    let expected_res = owned_table([boolean("a", [true]), varchar("c", ["Time"])]);
    assert_eq!(res, expected_res);
    Ok(())
}

// Note that both the optimizer and the analyzer take a callback, called an
// "observer" that is invoked after each pass. We do not do anything with these
// callbacks in this example

fn observe_analyzer(_plan: &LogicalPlan, _rule: &dyn AnalyzerRule) {}
fn observe_optimizer(_plan: &LogicalPlan, _rule: &dyn OptimizerRule) {}
