use crate::index::Index;
use crate::metadata::{Packet, ParameterValue};
use crate::query::query_types::*;
use crate::query::QueryError;

pub fn eval_query<'a>(index: &'a Index, query: QueryNode) -> Result<Vec<&'a Packet>, QueryError> {
    match query {
        QueryNode::Latest(inner) => eval_latest(index, inner),
        QueryNode::Lookup(field, value) => eval_lookup(index, field, value),
    }
}

fn eval_latest<'a>(
    index: &'a Index,
    inner: Option<Box<QueryNode>>,
) -> Result<Vec<&'a Packet>, QueryError> {
    if inner.is_some() {
        let latest = eval_query(index, *inner.unwrap())?;
        let last = latest.last();
        match last {
            Some(packet) => Ok(vec![*packet]),
            None => Ok(vec![]),
        }
    } else {
        let last = index.packets.last();
        match last {
            Some(packet) => Ok(vec![packet]),
            None => Ok(vec![]),
        }
    }
}

fn eval_lookup<'a>(
    index: &'a Index,
    lookup_field: LookupLhs,
    value: &str,
) -> Result<Vec<&'a Packet>, QueryError> {
    Ok(index
        .packets
        .iter()
        .filter(|packet| match lookup_field {
            LookupLhs::Id => packet.id == value,
            LookupLhs::Name => packet.name == value,
            LookupLhs::Parameter(param_name) =>
                packet.parameter_equals(param_name, ParameterValue::String(value))
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
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180818-164043-7cdcde4b"]);

        let query = QueryNode::Lookup(LookupLhs::Name, "modup-201707-queries1");
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(
            res,
            vec![
                "20170818-164830-33e0ab01",
                "20170818-164847-7574883b",
                "20180818-164043-7cdcde4b",
            ],
        );

        let query = QueryNode::Lookup(LookupLhs::Id, "123");
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);

        let query = QueryNode::Lookup(LookupLhs::Parameter("disease"), "YF");
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 3);

        let query = QueryNode::Lookup(LookupLhs::Parameter("foo"), "bar");
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);
    }

    #[test]
    fn query_latest_works() {
        let index = crate::index::get_packet_index("tests/example").unwrap();

        let query = QueryNode::Latest(None);
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180818-164043-7cdcde4b"]);

        let inner_query = QueryNode::Lookup(LookupLhs::Name, "modup-201707-queries1");
        let query = QueryNode::Latest(Some(Box::new(inner_query)));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180818-164043-7cdcde4b"]);

        let inner_query = QueryNode::Lookup(LookupLhs::Name, "123");
        let query = QueryNode::Latest(Some(Box::new(inner_query)));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);
    }
}
