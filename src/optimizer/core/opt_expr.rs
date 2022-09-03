use crate::optimizer::{Dummy, PlanRef};

pub type OptExprNodeId = usize;

#[derive(Clone, Debug)]
pub enum OptExprNode {
    /// Raw plan node with dummy children.
    PlanRef(PlanRef),
    /// Existing OptExprNode in graph.
    OptExpr(OptExprNodeId),
}

/// A sub-plan-tree representation used in Rule and Matcher. Every root node could be new node or
/// existing graph node. For new node, it will be added in graph, for existing node, it will be
/// reconnect in graph later.
///
/// It constructed by `PatternMatcher` when optimizer to match a rule, and consumed by `Rule` to do
/// transformation, and `Rule` return new `OptExpr` to replace the matched sub-tree.
#[derive(Clone, Debug)]
pub struct OptExpr {
    /// The root of the tree.
    pub root: OptExprNode,
    /// The root's children expressions.
    pub children: Vec<OptExpr>,
}

impl OptExpr {
    pub fn new(root: OptExprNode, children: Vec<OptExpr>) -> Self {
        Self { root, children }
    }

    /// Create OptExpr tree from PlanRef tree, it will change all nodes' children to dummy nodes.
    pub fn new_from_plan_ref(plan: &PlanRef) -> Self {
        OptExpr::build_opt_expr_internal(plan)
    }

    fn build_opt_expr_internal(input: &PlanRef) -> OptExpr {
        let root = OptExprNode::PlanRef(input.clone_with_dummy());
        let children = input
            .children()
            .iter()
            .map(OptExpr::build_opt_expr_internal)
            .collect::<Vec<_>>();
        OptExpr { root, children }
    }

    pub fn to_plan_ref(&self) -> PlanRef {
        match &self.root {
            OptExprNode::PlanRef(p) => {
                let children = self
                    .children
                    .iter()
                    .map(|c| c.to_plan_ref())
                    .collect::<Vec<_>>();
                p.clone_with_children(children)
            }
            OptExprNode::OptExpr(_) => Dummy::new_ref(),
        }
    }
}
