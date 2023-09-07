use crate::index::Index;
use crate::metadata::Packet;
use crate::query::query_types::*;
use crate::query::QueryError;
use serde_json::value::Value as JsonValue;

pub fn eval_query<'a>(index: &'a Index, query: QueryNode) -> Result<Vec<&'a Packet>, QueryError> {
    match query {
        QueryNode::Latest(inner) => eval_latest(index, inner),
        QueryNode::Test(test, field, value) => eval_test(index, test, value, field),
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

fn eval_test<'a>(
    index: &'a Index,
    test: Test,
    value: Literal,
    lookup_field: Lookup,
) -> Result<Vec<&'a Packet>, QueryError> {
    Ok(index
        .packets
        .iter()
        .filter(|packet| lookup_filter(packet, &test, &value,&lookup_field))
        .collect())
}

fn lookup_filter(packet: &Packet, test: &Test, value: &Literal, lookup: &Lookup) -> bool {
    match (test, lookup) {
        (Test::Equal, Lookup::Id) => match value {
            Literal::String(str) => packet.id == *str,
            _ => false
        },
        (Test::NotEqual, Lookup::Id) => match value {
            Literal::String(str) => packet.id != *str,
            _ => false
        },
        (_, Lookup::Id) => false,
        (Test::Equal, Lookup::Name) => match value {
            Literal::String(str) => packet.name == *str,
            _ => false
        },
        (Test::NotEqual, Lookup::Name) => match value {
            Literal::String(str) => packet.name != *str,
            _ => false
        },
        (_, Lookup::Name) => false,
        (test, Lookup::Parameter(param_name)) => packet.test_parameter(param_name, test, value),
    }
}

impl Packet {
    pub fn get_parameter(&self, param_name: &str) ->  Option<Literal> {
        if let Some(params) = &self.parameters {
            match params.get(param_name)? {
                JsonValue::Number(number) => {
                    Some(Literal::Number(number.as_f64()?))
                },
                JsonValue::Bool(bool) => {
                    Some(Literal::Bool(*bool))
                },
                JsonValue::String(string) => {
                    Some(Literal::String(string))
                },
                _ => None // Parameters must be number, bool or string
            }
        } else {
            None
        }
    }

    /// Check if a packet parameter is equal to a test value
    ///
    /// This will get the parameter `param_name` from the current Packet and then test for
    /// equality of the test value `value`.
    ///
    /// # Arguments
    /// * `param_name` - A string slice that holds the name of the parameter to test
    /// * `value` - A LookupRhs which holds the value to check equality for, can be a boolean, a
    ///             string or a number
    ///
    /// # Return
    /// * bool - true if the current packet has a parameter called `param_name` and its value
    ///          is equal to the LookupRhs.
    fn test_parameter(&self, param_name: &str, test: &Test, value: &Literal) -> bool {
        if let Some(json_value) = self.get_parameter(param_name) {
            match test {
                Test::Equal => json_value == *value,
                Test::NotEqual => json_value != *value,
                Test::LessThan => json_value < *value,
                Test::LessThanOrEqual => json_value <= *value,
                Test::GreaterThan => json_value > *value,
                Test::GreaterThanOrEqual => json_value >= *value,
            }
        } else {
            false
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::metadata::get_metadata_from_date;
    use super::*;
    use crate::test_utils::tests::assert_packet_ids_eq;

    #[test]
    fn query_lookup_works() {
        let index = crate::index::get_packet_index("tests/example").unwrap();

        let query = QueryNode::Test(Test::Equal, Lookup::Id, Literal::String("20180818-164043-7cdcde4b"));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180818-164043-7cdcde4b"]);

        let query = QueryNode::Test(Test::Equal, Lookup::Name, Literal::String("modup-201707-queries1"));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(
            res,
            vec![
                "20170818-164830-33e0ab01",
                "20170818-164847-7574883b",
                "20180818-164043-7cdcde4b",
            ],
        );

        let query = QueryNode::Test(Test::Equal, Lookup::Id, Literal::String("123"));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);

        let query = QueryNode::Test(Test::Equal, Lookup::Parameter("disease"), Literal::String("YF"));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 3);

        let query = QueryNode::Test(Test::Equal, Lookup::Parameter("foo"), Literal::String("bar"));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);
    }

    #[test]
    fn query_latest_works() {
        let index = crate::index::get_packet_index("tests/example").unwrap();

        let query = QueryNode::Latest(None);
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180818-164043-7cdcde4b"]);

        let inner_query = QueryNode::Test(Test::Equal, Lookup::Name, Literal::String("modup-201707-queries1"));
        let query = QueryNode::Latest(Some(Box::new(inner_query)));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180818-164043-7cdcde4b"]);

        let inner_query = QueryNode::Test(Test::Equal, Lookup::Name, Literal::String("123"));
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
    fn can_test_parameter_equality() {
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

        // assert!(packet.parameter_equals("tolerance",
        //                                 &Literal::Number(0.001)));
        // assert!(!packet.parameter_equals("tolerance",
        //                                  &Literal::Number(0.002)));
        // assert!(!packet.parameter_equals("tolerance",
        //                                  &Literal::String("0.001")));
        //
        // assert!(packet.parameter_equals("disease",
        //                                 &Literal::String("YF")));
        // assert!(!packet.parameter_equals("disease",
        //                                  &Literal::String("HepB")));
        // assert!(!packet.parameter_equals("disease",
        //                                  &Literal::Number(0.5)));
        //
        // assert!(packet.parameter_equals("size",
        //                                 &Literal::Number(10f64)));
        // assert!(packet.parameter_equals("size",
        //                                 &Literal::Number(10.0)));
        // assert!(!packet.parameter_equals("size",
        //                                  &Literal::Number(9f64)));
        // assert!(!packet.parameter_equals("size",
        //                                  &Literal::Bool(true)));
        //
        // assert!(packet.parameter_equals("pull_data",
        //                                 &Literal::Bool(true)));
        // assert!(!packet.parameter_equals("pull_data",
        //                                  &Literal::Bool(false)));
        // assert!(!packet.parameter_equals("pull_data",
        //                                  &Literal::String("true")));
    }

    #[test]
    fn can_use_different_test_types() {
        let index = crate::index::get_packet_index("tests/example").unwrap();

        let query = QueryNode::Test(Test::Equal, Lookup::Name, Literal::String("modup-201707-params1"));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180220-095832-16a4bbed"]);
        let query = QueryNode::Test(Test::Equal, Lookup::Parameter("size"), Literal::Number(10f64));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180220-095832-16a4bbed"]);

        let query = QueryNode::Test(Test::LessThan, Lookup::Parameter("size"), Literal::Number(10.1f64));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180220-095832-16a4bbed"]);
        let query = QueryNode::Test(Test::GreaterThan, Lookup::Parameter("size"), Literal::Number(9.4f64));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180220-095832-16a4bbed"]);
        let query = QueryNode::Test(Test::GreaterThan, Lookup::Parameter("size"), Literal::Number(10f64));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);
        let query = QueryNode::Test(Test::GreaterThanOrEqual, Lookup::Parameter("size"), Literal::Number(10f64));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180220-095832-16a4bbed"]);
        let query = QueryNode::Test(Test::LessThanOrEqual, Lookup::Parameter("size"), Literal::Number(10f64));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180220-095832-16a4bbed"]);

        let query = QueryNode::Test(Test::NotEqual, Lookup::Parameter("pull_data"), Literal::Bool(false));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180220-095832-16a4bbed"]);
        let query = QueryNode::Test(Test::NotEqual, Lookup::Parameter("pull_data"), Literal::Bool(true));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);
    }

    #[test]
    fn invalid_comparisons_dont_match() {
        let index = crate::index::get_packet_index("tests/example").unwrap();

        let query = QueryNode::Test(Test::GreaterThan, Lookup::Parameter("disease"), Literal::String("ABC"));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);
        let query = QueryNode::Test(Test::LessThan, Lookup::Parameter("disease"), Literal::String("ABC"));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);
        let query = QueryNode::Test(Test::GreaterThanOrEqual, Lookup::Parameter("disease"), Literal::String("YF"));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);
        let query = QueryNode::Test(Test::LessThanOrEqual, Lookup::Parameter("disease"), Literal::String("YF"));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);

        let query = QueryNode::Test(Test::GreaterThanOrEqual, Lookup::Parameter("pull_data"), Literal::Bool(true));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);
        let query = QueryNode::Test(Test::LessThanOrEqual, Lookup::Parameter("pull_data"), Literal::Bool(false));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);
    }

    #[test]
    fn query_does_no_type_coersion() {
        let index = crate::index::get_packet_index("tests/example").unwrap();

        let query = QueryNode::Test(Test::Equal, Lookup::Parameter("pull_data"), Literal::String("TRUE"));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);
        let query = QueryNode::Test(Test::Equal, Lookup::Parameter("pull_data"), Literal::String("true"));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);
        let query = QueryNode::Test(Test::Equal, Lookup::Parameter("pull_data"), Literal::String("T"));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);
        let query = QueryNode::Test(Test::Equal, Lookup::Parameter("pull_data"), Literal::Number(1f64));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);
    }
}
