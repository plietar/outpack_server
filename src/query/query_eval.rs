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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::tests::assert_packet_ids_eq;

    #[test]
    fn query_lookup_works() {
        let index = crate::index::get_packet_index("tests/example").unwrap();

        let query = QueryNode::Lookup(LookupLhs::Id, "20180818-164043-7cdcde4b");
        let res = eval_query(index.clone(), query).unwrap();
        assert_packet_ids_eq(res, vec!["20180818-164043-7cdcde4b"]);

        let query = QueryNode::Lookup(LookupLhs::Name, "modup-201707-queries1");
        let res = eval_query(index.clone(), query).unwrap();
        assert_packet_ids_eq(
            res,
            vec![
                "20170818-164830-33e0ab01",
                "20170818-164847-7574883b",
                "20180818-164043-7cdcde4b",
            ],
        );
    }

    #[test]
    fn query_latest_works() {
        let index = crate::index::get_packet_index("tests/example").unwrap();

        let query = QueryNode::Latest(None);
        let res = eval_query(index.clone(), query).unwrap();
        assert_packet_ids_eq(res, vec!["20180818-164043-7cdcde4b"]);

        let inner_query = QueryNode::Lookup(LookupLhs::Name, "modup-201707-queries1");
        let query = QueryNode::Latest(Some(Box::new(inner_query)));
        let res = eval_query(index.clone(), query).unwrap();
        assert_packet_ids_eq(res, vec!["20180818-164043-7cdcde4b"]);
    }
}
