#[cfg(test)]
pub mod tests {
    use crate::query::query_types::*;

    pub fn assert_query_node_lookup_number_eq(node: QueryNode, lookup: LookupLhs, test: f64) {
        if let QueryNode::Lookup(lhs, rhs) = node {
            assert_eq!(lhs, lookup);
            match rhs {
                LookupRhs::Number(value) => assert_eq!(value, test),
                _ => panic!("Query parse rhs should have returned a Float")
            }
        } else {
            panic!("Query parse should have returned a Lookup QueryNode")
        }
    }
}
