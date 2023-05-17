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
