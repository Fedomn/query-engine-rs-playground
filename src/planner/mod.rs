mod select;
mod util;

use crate::binder::BoundStatement;
use crate::optimizer::PlanRef;

pub struct Planner {}

impl Planner {
    pub fn plan(&self, stmt: BoundStatement) -> Result<PlanRef, LogicalPlanError> {
        match stmt {
            BoundStatement::Select(stmt) => self.plan_select(stmt),
        }
    }
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum LogicalPlanError {}

#[cfg(test)]
mod planner_test {
    use std::collections::BTreeMap;

    use arrow::datatypes::DataType::{self, Int32};
    use sqlparser::ast::BinaryOperator;

    use super::*;
    use crate::binder::{
        BoundBinaryOp, BoundColumnRef, BoundExpr, BoundSelect, BoundStatement, BoundTableRef,
    };
    use crate::catalog::{ColumnCatalog, ColumnDesc, TableCatalog};
    use crate::optimizer::PlanNodeType;
    use crate::types::ScalarValue;

    fn build_test_column(table_id: String, column_name: String) -> BoundExpr {
        BoundExpr::ColumnRef(BoundColumnRef {
            column_catalog: ColumnCatalog {
                table_id,
                column_id: column_name.clone(),
                desc: ColumnDesc {
                    name: column_name,
                    data_type: Int32,
                },
            },
        })
    }

    fn build_test_table(table_name: String, columns: Vec<String>) -> Option<BoundTableRef> {
        let mut column_map = BTreeMap::new();
        let mut column_ids = Vec::new();
        for column in columns {
            column_ids.push(column.clone());
            column_map.insert(
                column.clone(),
                ColumnCatalog {
                    table_id: table_name.clone(),
                    column_id: column.clone(),
                    desc: ColumnDesc {
                        name: column,
                        data_type: Int32,
                    },
                },
            );
        }
        Some(BoundTableRef::Table {
            table_catalog: TableCatalog {
                id: table_name.clone(),
                name: table_name,
                columns: column_map,
                column_ids,
            },
        })
    }

    fn build_test_select_stmt() -> BoundStatement {
        let table_id = "t".to_string();
        let c1 = build_test_column(table_id.clone(), "c1".to_string());
        let t = build_test_table(table_id.clone(), vec!["c1".to_string(), "c2".to_string()]);

        let where_clause = BoundExpr::BinaryOp(BoundBinaryOp {
            op: BinaryOperator::Eq,
            left: Box::new(build_test_column(table_id, "c2".to_string())),
            right: Box::new(BoundExpr::Constant(ScalarValue::Int32(Some(2)))),
            return_type: Some(DataType::Boolean),
        });

        BoundStatement::Select(BoundSelect {
            select_list: vec![c1],
            from_table: t,
            where_clause: Some(where_clause),
            group_by: vec![],
            limit: Some(BoundExpr::Constant(10.into())),
            offset: None,
            order_by: vec![],
        })
    }

    #[test]
    fn test_plan_select_works() {
        let stmt = build_test_select_stmt();
        let p = Planner {};
        let node = p.plan(stmt);
        assert!(node.is_ok());
        let plan_ref = node.unwrap();
        assert_eq!(plan_ref.node_type(), PlanNodeType::LogicalLimit);
        assert_eq!(plan_ref.schema().len(), 2);
        dbg!(plan_ref);
    }
}
