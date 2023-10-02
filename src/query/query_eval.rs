use std::collections::HashSet;

use serde_json::value::Value as JsonValue;

use crate::index::Index;
use crate::metadata::Packet;
use crate::query::query_types::*;
use crate::query::QueryError;

pub fn eval_query<'a>(index: &'a Index, query: QueryNode) -> Result<Vec<&'a Packet>, QueryError> {
    match query {
        QueryNode::Latest(inner) => eval_latest(index, inner),
        QueryNode::Single(inner) => eval_single(index, *inner),
        QueryNode::Test(test, lhs, rhs) => eval_test(index, test, lhs, rhs),
        QueryNode::Negation(inner) => eval_negation(index, *inner),
        QueryNode::Brackets(inner) => eval_brackets(index, *inner),
        QueryNode::BooleanOperator(op, lhs, rhs) => eval_boolean_op(index, op, *lhs, *rhs),
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

fn eval_single<'a>(index: &'a Index, inner: QueryNode) -> Result<Vec<&'a Packet>, QueryError> {
    let packets = eval_query(index, inner)?;
    if packets.len() != 1 {
        Err(QueryError::EvalError(format!("Query found {} packets, but expected exactly one",
                                          packets.len())))
    } else {
        Ok(packets)
    }
}

fn eval_negation<'a>(
    index: &'a Index,
    inner: QueryNode,
) -> Result<Vec<&'a Packet>, QueryError> {
    let packets = eval_query(index, inner)?;
    Ok(index
        .packets
        .iter()
        .filter(|packet| !packets.contains(packet))
        .collect())
}


fn eval_brackets<'a>(
    index: &'a Index,
    inner: QueryNode,
) -> Result<Vec<&'a Packet>, QueryError> {
    eval_query(index, inner)
}

fn eval_test<'a>(
    index: &'a Index,
    test: Test,
    lhs: TestValue,
    rhs: TestValue,
) -> Result<Vec<&'a Packet>, QueryError> {
    Ok(index
        .packets
        .iter()
        .filter(|packet| lookup_filter(packet, &test, &lhs, &rhs))
        .collect())
}

fn lookup_filter(packet: &Packet, test: &Test, lhs: &TestValue, rhs: &TestValue) -> bool {
    let lhs_literal = match lhs {
        TestValue::Literal(value) => Some(value.clone()),
        TestValue::Lookup(lookup) => packet.lookup_value(lookup)
    };
    let rhs_literal = match rhs {
        TestValue::Literal(value) => Some(value.clone()),
        TestValue::Lookup(lookup) => packet.lookup_value(lookup)
    };

    match (test, lhs_literal, rhs_literal) {
        (test, Some(Literal::Number(l)), Some(Literal::Number(r))) => {
            match test {
                Test::Equal => l == r,
                Test::NotEqual => l != r,
                Test::LessThan => l < r,
                Test::LessThanOrEqual => l <= r,
                Test::GreaterThan => l > r,
                Test::GreaterThanOrEqual => l >= r,
            }
        }
        (Test::Equal, Some(l), Some(r)) => l == r,
        (Test::NotEqual, Some(l), Some(r)) => l != r,
        (_, _, _) => false
    }
}

impl Packet {
    pub fn lookup_value(&self, lookup: &Lookup) -> Option<Literal> {
        match lookup {
            Lookup::Id => Some(Literal::String(&self.id)),
            Lookup::Name => Some(Literal::String(&self.name)),
            Lookup::Parameter(param_name) => self.get_parameter(param_name),
        }
    }

    pub fn get_parameter(&self, param_name: &str) -> Option<Literal> {
        if let Some(params) = &self.parameters {
            match params.get(param_name)? {
                JsonValue::Number(number) => {
                    Some(Literal::Number(number.as_f64()?))
                }
                JsonValue::Bool(bool) => {
                    Some(Literal::Bool(*bool))
                }
                JsonValue::String(string) => {
                    Some(Literal::String(string))
                }
                _ => None // Parameters must be number, bool or string
            }
        } else {
            None
        }
    }
}

fn eval_boolean_op<'a>(
    index: &'a Index,
    op: Operator,
    lhs: QueryNode,
    rhs: QueryNode,
) -> Result<Vec<&'a Packet>, QueryError> {
    let lhs_res = eval_query(index, lhs)?;
    let rhs_res = eval_query(index, rhs)?;
    let lhs_set: HashSet<&Packet> = HashSet::from_iter(lhs_res.iter().cloned());
    let rhs_set: HashSet<&Packet> = HashSet::from_iter(rhs_res.iter().cloned());
    match op {
        Operator::And => {
            Ok(lhs_set.intersection(&rhs_set).copied().collect())
        }
        Operator::Or => {
            Ok(lhs_set.union(&rhs_set).copied().collect())
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::metadata::get_metadata_from_date;
    use crate::test_utils::tests::assert_packet_ids_eq;

    use super::*;

    #[test]
    fn query_lookup_works() {
        let index = crate::index::get_packet_index("tests/example").unwrap();

        let query = QueryNode::Test(Test::Equal, TestValue::Lookup(Lookup::Id),
                                    TestValue::Literal(Literal::String("20180818-164043-7cdcde4b")));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180818-164043-7cdcde4b"]);

        let query = QueryNode::Test(Test::Equal, TestValue::Lookup(Lookup::Name),
                                    TestValue::Literal(Literal::String("modup-201707-queries1")));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(
            res,
            vec![
                "20170818-164830-33e0ab01",
                "20170818-164847-7574883b",
                "20180818-164043-7cdcde4b",
            ],
        );

        let query = QueryNode::Test(Test::Equal, TestValue::Lookup(Lookup::Id), TestValue::Literal(Literal::String("123")));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);

        let query = QueryNode::Test(Test::Equal, TestValue::Lookup(Lookup::Parameter("disease")), TestValue::Literal(Literal::String("YF")));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 3);

        let query = QueryNode::Test(Test::Equal, TestValue::Lookup(Lookup::Parameter("foo")), TestValue::Literal(Literal::String("bar")));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);
    }

    #[test]
    fn query_latest_works() {
        let index = crate::index::get_packet_index("tests/example").unwrap();

        let query = QueryNode::Latest(None);
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180818-164043-7cdcde4b"]);

        let inner_query = QueryNode::Test(Test::Equal, TestValue::Lookup(Lookup::Name), TestValue::Literal(Literal::String("modup-201707-queries1")));
        let query = QueryNode::Latest(Some(Box::new(inner_query)));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180818-164043-7cdcde4b"]);

        let inner_query = QueryNode::Test(Test::Equal, TestValue::Lookup(Lookup::Name), TestValue::Literal(Literal::String("123")));
        let query = QueryNode::Latest(Some(Box::new(inner_query)));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);
    }

    #[test]
    fn can_get_parameter_as_literal() {
        let packets = get_metadata_from_date("tests/example", None)
            .unwrap();
        assert_eq!(packets.len(), 4);

        let matching_packets: Vec<Packet> = packets
            .into_iter()
            .filter(|e| e.id == "20180220-095832-16a4bbed")
            .collect();
        assert_eq!(matching_packets.len(), 1);

        let packet = matching_packets.first().unwrap();
        assert_eq!(packet.id, "20180220-095832-16a4bbed");

        assert_eq!(packet.get_parameter("missing"), None);
        assert_eq!(packet.get_parameter("disease"), Some(Literal::String("YF")));
        assert_eq!(packet.get_parameter("pull_data"), Some(Literal::Bool(true)));
        assert_eq!(packet.get_parameter("tolerance"), Some(Literal::Number(0.001)));
        assert_eq!(packet.get_parameter("size"), Some(Literal::Number(10f64)));
    }

    #[test]
    fn can_test_lookup_filter() {
        let packets = get_metadata_from_date("tests/example", None)
            .unwrap();
        assert_eq!(packets.len(), 4);

        let matching_packets: Vec<Packet> = packets
            .into_iter()
            .filter(|e| e.id == "20180220-095832-16a4bbed")
            .collect();
        assert_eq!(matching_packets.len(), 1);

        let packet = matching_packets.first().unwrap();
        assert_eq!(packet.id, "20180220-095832-16a4bbed");
        assert_eq!(packet.name, "modup-201707-params1");
        assert!(packet.parameters.is_some());

        let params = packet.parameters.clone().unwrap();
        assert_eq!(params.len(), 4);
        assert_eq!(params.get("tolerance").unwrap(),
                   &(serde_json::Value::Number(serde_json::Number::from_f64(0.001).unwrap())));
        assert_eq!(params.get("size").unwrap(),
                   &(serde_json::Value::Number(serde_json::Number::from(10))));
        assert_eq!(params.get("disease").unwrap(),
                   &(serde_json::Value::String(String::from("YF"))));
        assert_eq!(params.get("pull_data").unwrap(),
                   &(serde_json::Value::Bool(true)));

        macro_rules! test_param {
            ( $( $test:expr, $lhs:expr, $rhs:expr => $result:literal )* ) => {
                $(
                if $result {
                    assert!(lookup_filter(&packet, $test, $lhs, $rhs));
                } else {
                    assert!(!lookup_filter(&packet, $test, $lhs, $rhs));
                }
                )*
            };
        }

        test_param!(
            &Test::Equal, &TestValue::Lookup(Lookup::Parameter("tolerance")), &TestValue::Literal(Literal::Number(0.001))   => true
            &Test::Equal, &TestValue::Lookup(Lookup::Parameter("tolerance")), &TestValue::Literal(Literal::Number(0.002))   => false
            &Test::Equal, &TestValue::Lookup(Lookup::Parameter("tolerance")), &TestValue::Literal(Literal::String("0.001")) => false

            &Test::Equal, &TestValue::Lookup(Lookup::Parameter("disease")), &TestValue::Literal(Literal::String("YF"))   => true
            &Test::Equal, &TestValue::Lookup(Lookup::Parameter("disease")), &TestValue::Literal(Literal::String("HepB")) => false
            &Test::Equal, &TestValue::Lookup(Lookup::Parameter("disease")), &TestValue::Literal(Literal::Number(0.5))    => false

            &Test::Equal, &TestValue::Lookup(Lookup::Parameter("size")), &TestValue::Literal(Literal::Number(10f64)) => true
            &Test::Equal, &TestValue::Lookup(Lookup::Parameter("size")), &TestValue::Literal(Literal::Number(10.0))  => true
            &Test::Equal, &TestValue::Lookup(Lookup::Parameter("size")), &TestValue::Literal(Literal::Number(9f64))  => false
            &Test::Equal, &TestValue::Lookup(Lookup::Parameter("size")), &TestValue::Literal(Literal::Bool(true))    => false

            &Test::Equal, &TestValue::Lookup(Lookup::Parameter("pull_data")), &TestValue::Literal(Literal::Bool(true))     => true
            &Test::Equal, &TestValue::Lookup(Lookup::Parameter("pull_data")), &TestValue::Literal(Literal::Bool(false))    => false
            &Test::Equal, &TestValue::Lookup(Lookup::Parameter("pull_data")), &TestValue::Literal(Literal::String("true")) => false

            &Test::NotEqual,           &TestValue::Lookup(Lookup::Parameter("tolerance")), &TestValue::Literal(Literal::Number(0.002)) => true
            &Test::LessThan,           &TestValue::Lookup(Lookup::Parameter("tolerance")), &TestValue::Literal(Literal::Number(0.002)) => true
            &Test::LessThanOrEqual,    &TestValue::Lookup(Lookup::Parameter("tolerance")), &TestValue::Literal(Literal::Number(0.002)) => true
            &Test::GreaterThan,        &TestValue::Lookup(Lookup::Parameter("tolerance")), &TestValue::Literal(Literal::Number(0.000)) => true
            &Test::GreaterThanOrEqual, &TestValue::Lookup(Lookup::Parameter("tolerance")), &TestValue::Literal(Literal::Number(0.000)) => true
            &Test::LessThan,           &TestValue::Lookup(Lookup::Parameter("tolerance")), &TestValue::Literal(Literal::Number(0.000)) => false
            &Test::LessThanOrEqual,    &TestValue::Lookup(Lookup::Parameter("tolerance")), &TestValue::Literal(Literal::Number(0.000)) => false

            &Test::LessThan, &TestValue::Lookup(Lookup::Parameter("pull_data")), &TestValue::Literal(Literal::Bool(true))  => false
            &Test::LessThan, &TestValue::Lookup(Lookup::Parameter("pull_data")), &TestValue::Literal(Literal::Bool(false)) => false

            &Test::LessThan,           &TestValue::Lookup(Lookup::Parameter("disease")), &TestValue::Literal(Literal::String("YF")) => false
            &Test::LessThanOrEqual,    &TestValue::Lookup(Lookup::Parameter("disease")), &TestValue::Literal(Literal::String("YF")) => false
            &Test::GreaterThan,        &TestValue::Lookup(Lookup::Parameter("disease")), &TestValue::Literal(Literal::String("YF")) => false
            &Test::GreaterThanOrEqual, &TestValue::Lookup(Lookup::Parameter("disease")), &TestValue::Literal(Literal::String("YF")) => false

            &Test::Equal, &TestValue::Literal(Literal::Number(0.001)), &TestValue::Literal(Literal::Number(0.001))    => true
            &Test::Equal, &TestValue::Literal(Literal::Number(0.001)), &TestValue::Literal(Literal::Number(0.002))    => false
            &Test::Equal, &TestValue::Literal(Literal::Number(0.001)), &TestValue::Literal(Literal::String("0.002"))  => false
            &Test::NotEqual, &TestValue::Literal(Literal::Number(0.001)), &TestValue::Literal(Literal::Number(0.001)) => false

            &Test::Equal, &TestValue::Lookup(Lookup::Parameter("tolerance")), &TestValue::Lookup(Lookup::Parameter("tolerance"))           => true
            &Test::NotEqual, &TestValue::Lookup(Lookup::Parameter("tolerance")), &TestValue::Lookup(Lookup::Parameter("tolerance"))        => false
            &Test::GreaterThan, &TestValue::Lookup(Lookup::Parameter("tolerance")), &TestValue::Lookup(Lookup::Parameter("tolerance"))     => false
            &Test::LessThanOrEqual, &TestValue::Lookup(Lookup::Parameter("tolerance")), &TestValue::Lookup(Lookup::Parameter("tolerance")) => true
        );
    }

    #[test]
    fn can_use_different_test_types() {
        let index = crate::index::get_packet_index("tests/example").unwrap();

        let query = QueryNode::Test(Test::Equal, TestValue::Lookup(Lookup::Name),
                                    TestValue::Literal(Literal::String("modup-201707-params1")));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180220-095832-16a4bbed"]);
        let query = QueryNode::Test(Test::Equal, TestValue::Lookup(Lookup::Parameter("size")),
                                    TestValue::Literal(Literal::Number(10f64)));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180220-095832-16a4bbed"]);

        let query = QueryNode::Test(Test::LessThan, TestValue::Lookup(Lookup::Parameter("size")),
                                    TestValue::Literal(Literal::Number(10.1f64)));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180220-095832-16a4bbed"]);
        let query = QueryNode::Test(Test::GreaterThan, TestValue::Lookup(Lookup::Parameter("size")),
                                    TestValue::Literal(Literal::Number(9.4f64)));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180220-095832-16a4bbed"]);
        let query = QueryNode::Test(Test::GreaterThan, TestValue::Lookup(Lookup::Parameter("size")),
                                    TestValue::Literal(Literal::Number(10f64)));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);
        let query = QueryNode::Test(Test::GreaterThanOrEqual, TestValue::Lookup(Lookup::Parameter("size")),
                                    TestValue::Literal(Literal::Number(10f64)));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180220-095832-16a4bbed"]);
        let query = QueryNode::Test(Test::LessThanOrEqual, TestValue::Lookup(Lookup::Parameter("size")),
                                    TestValue::Literal(Literal::Number(10f64)));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180220-095832-16a4bbed"]);

        let query = QueryNode::Test(Test::NotEqual, TestValue::Lookup(Lookup::Parameter("pull_data")),
                                    TestValue::Literal(Literal::Bool(false)));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180220-095832-16a4bbed"]);
        let query = QueryNode::Test(Test::NotEqual, TestValue::Lookup(Lookup::Parameter("pull_data")),
                                    TestValue::Literal(Literal::Bool(true)));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);
    }

    #[test]
    fn invalid_comparisons_dont_match() {
        let index = crate::index::get_packet_index("tests/example").unwrap();

        let query = QueryNode::Test(Test::GreaterThan, TestValue::Lookup(Lookup::Parameter("disease")),
                                    TestValue::Literal(Literal::String("ABC")));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);
        let query = QueryNode::Test(Test::LessThan, TestValue::Lookup(Lookup::Parameter("disease")),
                                    TestValue::Literal(Literal::String("ABC")));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);
        let query = QueryNode::Test(Test::GreaterThanOrEqual, TestValue::Lookup(Lookup::Parameter("disease")),
                                    TestValue::Literal(Literal::String("YF")));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);
        let query = QueryNode::Test(Test::LessThanOrEqual, TestValue::Lookup(Lookup::Parameter("disease")),
                                    TestValue::Literal(Literal::String("YF")));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);

        let query = QueryNode::Test(Test::GreaterThanOrEqual, TestValue::Lookup(Lookup::Parameter("pull_data")),
                                    TestValue::Literal(Literal::Bool(true)));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);
        let query = QueryNode::Test(Test::LessThanOrEqual, TestValue::Lookup(Lookup::Parameter("pull_data")),
                                    TestValue::Literal(Literal::Bool(false)));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);
    }

    #[test]
    fn query_does_no_type_coersion() {
        let index = crate::index::get_packet_index("tests/example").unwrap();

        let query = QueryNode::Test(Test::Equal, TestValue::Lookup(Lookup::Parameter("pull_data")),
                                    TestValue::Literal(Literal::String("TRUE")));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);
        let query = QueryNode::Test(Test::Equal, TestValue::Lookup(Lookup::Parameter("pull_data")),
                                    TestValue::Literal(Literal::String("true")));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);
        let query = QueryNode::Test(Test::Equal, TestValue::Lookup(Lookup::Parameter("pull_data")),
                                    TestValue::Literal(Literal::String("T")));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);
        let query = QueryNode::Test(Test::Equal, TestValue::Lookup(Lookup::Parameter("pull_data")),
                                    TestValue::Literal(Literal::Number(1f64)));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);
    }

    #[test]
    fn query_with_negation_works() {
        let index = crate::index::get_packet_index("tests/example").unwrap();

        let query = QueryNode::Negation(Box::new(QueryNode::Latest(None)));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20170818-164830-33e0ab01",
                                       "20170818-164847-7574883b",
                                       "20180220-095832-16a4bbed"]);

        let query = QueryNode::Negation(Box::new(
            QueryNode::Negation(Box::new(QueryNode::Latest(None)))));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180818-164043-7cdcde4b"]);
    }

    #[test]
    fn query_with_brackets_works() {
        let index = crate::index::get_packet_index("tests/example").unwrap();

        let query = QueryNode::Brackets(Box::new(QueryNode::Latest(None)));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180818-164043-7cdcde4b"]);

        let query = QueryNode::Brackets(Box::new(
            QueryNode::Brackets(Box::new(QueryNode::Latest(None)))));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180818-164043-7cdcde4b"]);

        let query = QueryNode::Brackets(Box::new(
            QueryNode::Negation(Box::new(QueryNode::Latest(None)))));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20170818-164830-33e0ab01",
                                       "20170818-164847-7574883b",
                                       "20180220-095832-16a4bbed"]);
    }

    #[test]
    fn query_with_boolean_operators_works() {
        let index = crate::index::get_packet_index("tests/example").unwrap();

        let query = QueryNode::BooleanOperator(
            Operator::Or,
            Box::new(QueryNode::Latest(None)),
            Box::new(QueryNode::Test(Test::Equal, TestValue::Lookup(Lookup::Name),
                                     TestValue::Literal(Literal::String("modup-201707-params1")))));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180818-164043-7cdcde4b", "20180220-095832-16a4bbed"]);

        let query = QueryNode::BooleanOperator(
            Operator::And,
            Box::new(QueryNode::Negation(Box::new(QueryNode::Latest(None)))),
            Box::new(QueryNode::Test(Test::Equal, TestValue::Lookup(Lookup::Name),
                                     TestValue::Literal(Literal::String("modup-201707-params1")))));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180220-095832-16a4bbed"]);
    }

    #[test]
    fn query_with_single_works() {
        let index = crate::index::get_packet_index("tests/example").unwrap();

        let query = QueryNode::Single(Box::new(QueryNode::Latest(None)));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180818-164043-7cdcde4b"]);

        let query = QueryNode::Single(Box::new(
            QueryNode::Negation(Box::new(QueryNode::Latest(None)))));
        let e = eval_query(&index, query).unwrap_err();
        assert!(matches!(e, QueryError::EvalError(..)));
        assert!(e.to_string().contains("Query found 3 packets, but expected exactly one"));
    }
}
