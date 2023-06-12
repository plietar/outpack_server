use crate::metadata::Packet;
use crate::query::QueryError;

pub fn format_query_result(packets: Result<Vec<Packet>, QueryError>) -> Result<String, QueryError> {
    let returned_packets = packets?;
    let mut packets_iter = returned_packets.iter().peekable();
    if packets_iter.peek().is_some() {
        Ok(itertools::Itertools::intersperse(
            packets_iter.map(|packet| packet.id.clone()),
            String::from("\n"),
        )
        .collect())
    } else {
        Ok(String::from("Found no packets"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_result_can_be_formatted() {
        let packets = crate::metadata::get_metadata_from_date("tests/example", None).unwrap();
        let one_packet = vec![packets[0].clone()];

        let res = format_query_result(Ok(packets)).unwrap();
        assert_eq!(
            res,
            "20170818-164830-33e0ab01\n20170818-164847-7574883b\n20180818-164043-7cdcde4b"
        );

        let res = format_query_result(Ok(one_packet)).unwrap();
        assert_eq!(res, "20170818-164830-33e0ab01");

        let res = format_query_result(Ok(vec![])).unwrap();
        assert_eq!(res, "Found no packets")
    }

    #[test]
    fn query_format_propagates_error() {
        let res = format_query_result(Err(QueryError::EvalError(String::from("An error"))));
        match res {
            Ok(_) => panic!("QueryError should be propagated in format"),
            Err(e) => {
                assert!(matches!(e, QueryError::EvalError(..)));
                assert!(e.to_string().contains("An error"));
            }
        };
    }
}
