use std::io;
use crate::metadata::get_ids;

pub struct Index {
    pub packets: Vec<Packet>
}

pub struct Packet {
    pub id: String
}

pub fn get_packet_index(root_path: &str) -> io::Result<Index> {
    let mut ids = get_ids(root_path)?;
    ids.sort();
    let index = ids.into_iter().map(|id| Packet {id}).collect::<Vec<Packet>>();
    Ok(Index {packets: index})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_get_packet_index() {
        let index = get_packet_index("tests/example").unwrap();
        assert_eq!(index.packets.len(), 2);
        let ids: Vec<String> = index.packets.iter()
            .map(|packet| packet.id.clone())
            .collect();
        assert_eq!(ids[0], "20170818-164847-7574883b");
        assert_eq!(ids[1], "20170818-164847-7574883c");
    }
}
