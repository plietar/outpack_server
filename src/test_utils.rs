#[cfg(test)]
pub mod tests {
    use crate::metadata::Packet;
    use std::collections::HashMap;
    use std::hash::Hash;

    pub fn vector_equals<T>(a: &[T], b: &[T]) -> bool
    where
        T: Eq + Hash,
    {
        fn count<T>(items: &[T]) -> HashMap<&T, usize>
        where
            T: Eq + Hash,
        {
            let mut cnt = HashMap::new();
            for i in items {
                *cnt.entry(i).or_insert(0) += 1
            }
            cnt
        }

        count(a) == count(b)
    }

    pub fn assert_packet_ids_eq(packets: Vec<&Packet>, ids: Vec<&str>) {
        let packet_ids: Vec<&str> = packets.iter().map(|packet| &packet.id[..]).collect();
        assert!(
            vector_equals(&packet_ids, &ids),
            "Packet ids differ to expected.\n  Packet ids are:\n  {:?}\n  Expected ids are:\n  {:?}",
            packet_ids,
            ids
        )
    }
}
