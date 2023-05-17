use crate::index::Index;
use crate::metadata::Packet;
use crate::query::query_types::*;
use crate::query::QueryError;

pub fn eval_query(index: Index, query: QueryNode) -> Result<Vec<Packet>, QueryError> {
    match query {
        QueryNode::Latest(inner) => eval_latest(index, inner),
        QueryNode::Lookup(field, value) => eval_lookup(index, field, value),
    }
}

fn eval_latest(index: Index, inner: Option<Box<QueryNode>>) -> Result<Vec<Packet>, QueryError> {
    match inner {
        Some(inner_node) => Ok(vec![eval_query(index, *inner_node)?
            .last()
            .unwrap()
            .clone()]),
        None => Ok(vec![index.packets.last().unwrap().clone()]),
    }
}

fn eval_lookup(
    index: Index,
    lookup_field: LookupLhs,
    value: &str,
) -> Result<Vec<Packet>, QueryError> {
    Ok(index
        .packets
        .into_iter()
        .filter(|packet| match lookup_field {
            LookupLhs::Id => packet.id == value,
            LookupLhs::Name => packet.name == value,
        })
        .collect())
}
