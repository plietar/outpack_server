use crate::index::Index;
use crate::metadata::Packet;
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
    value: LookupRhs,
) -> Result<Vec<&'a Packet>, QueryError> {
    Ok(index
        .packets
        .iter()
        .filter(|packet| match lookup_field {
            LookupLhs::Id => match value {
                LookupRhs::String(str) => packet.id == str,
                _ => false
            },
            LookupLhs::Name => match value {
                LookupRhs::String(str) => packet.name == str,
                _ => false
            },
            LookupLhs::Parameter(param_name) => packet.parameter_equals(param_name, &value)
        })
        .collect())
}

impl Packet {
    /// Check if a packet parameter is equal to a test value
    ///
    /// This will get the parameter `param_name` from the current Packet and then test for
    /// equality of the test value `value`.
    ///
    /// # Arguments
    /// * `param_name` - A string slice that holds the name of the parameter to test
    /// * `value` - A LookupRhs which holds the value to check equality for, can be a boolean, a
    ///             string, an integer or a float
    ///
    /// # Return
    /// * bool - true if the current packet has a parameter called `param_name` and its value
    ///          is equal to the LookupRhs.
    fn parameter_equals(&self, param_name: &str, value: &LookupRhs) -> bool {
        if let Some(json_value) = self.get_parameter(param_name) {
            match (json_value, value) {
                (serde_json::value::Value::Bool(json_val), LookupRhs::Bool(test_val)) => {
                    *json_val == *test_val
                },
                (serde_json::value::Value::Number(json_val), LookupRhs::Float(test_val)) => {
                    if json_val.is_f64() {
                        let test_number = serde_json::Number::from_f64(*test_val);
                        match test_number {
                            Some(number) => *json_val == number,
                            None => false,
                        }
                    } else {
                        *json_val == serde_json::Number::from(*test_val as i32)
                    }
                }
                (serde_json::value::Value::Number(json_val), LookupRhs::Integer(test_val)) => {
                    *json_val == serde_json::Number::from(*test_val)
                }
                (serde_json::value::Value::String(json_val), LookupRhs::String(test_val)) => {
                    *json_val == **test_val
                }
                (_, _) => false,
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

        let query = QueryNode::Lookup(LookupLhs::Id, LookupRhs::String("20180818-164043-7cdcde4b"));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180818-164043-7cdcde4b"]);

        let query = QueryNode::Lookup(LookupLhs::Name, LookupRhs::String("modup-201707-queries1"));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(
            res,
            vec![
                "20170818-164830-33e0ab01",
                "20170818-164847-7574883b",
                "20180818-164043-7cdcde4b",
            ],
        );

        let query = QueryNode::Lookup(LookupLhs::Id, LookupRhs::String("123"));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);

        let query = QueryNode::Lookup(LookupLhs::Parameter("disease"), LookupRhs::String("YF"));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 3);

        let query = QueryNode::Lookup(LookupLhs::Parameter("foo"), LookupRhs::String("bar"));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);
    }

    #[test]
    fn query_latest_works() {
        let index = crate::index::get_packet_index("tests/example").unwrap();

        let query = QueryNode::Latest(None);
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180818-164043-7cdcde4b"]);

        let inner_query = QueryNode::Lookup(LookupLhs::Name, LookupRhs::String("modup-201707-queries1"));
        let query = QueryNode::Latest(Some(Box::new(inner_query)));
        let res = eval_query(&index, query).unwrap();
        assert_packet_ids_eq(res, vec!["20180818-164043-7cdcde4b"]);

        let inner_query = QueryNode::Lookup(LookupLhs::Name, LookupRhs::String("123"));
        let query = QueryNode::Latest(Some(Box::new(inner_query)));
        let res = eval_query(&index, query).unwrap();
        assert_eq!(res.len(), 0);
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

        assert!(packet.parameter_equals("tolerance",
                                        &LookupRhs::Float(0.001)));
        assert!(!packet.parameter_equals("tolerance",
                                         &LookupRhs::Float(0.002)));
        assert!(!packet.parameter_equals("tolerance",
                                         &LookupRhs::Integer(10)));
        assert!(!packet.parameter_equals("tolerance",
                                         &LookupRhs::String("0.001")));

        assert!(packet.parameter_equals("disease",
                                        &LookupRhs::String("YF")));
        assert!(!packet.parameter_equals("disease",
                                         &LookupRhs::String("HepB")));
        assert!(!packet.parameter_equals("disease",
                                         &LookupRhs::Float(0.5)));

        assert!(packet.parameter_equals("size",
                                        &LookupRhs::Integer(10)));
        assert!(packet.parameter_equals("size",
                                        &LookupRhs::Float(10.0)));
        assert!(!packet.parameter_equals("size",
                                         &LookupRhs::Integer(9)));
        assert!(!packet.parameter_equals("size",
                                         &LookupRhs::Bool(true)));

        assert!(packet.parameter_equals("pull_data",
                                        &LookupRhs::Bool(true)));
        assert!(!packet.parameter_equals("pull_data",
                                         &LookupRhs::Bool(false)));
        assert!(!packet.parameter_equals("pull_data",
                                         &LookupRhs::String("true")));
    }
}
