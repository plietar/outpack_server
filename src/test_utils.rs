#[cfg(test)]
pub mod tests {
    use crate::metadata::Packet;
    use std::collections::HashMap;
    use std::fs::File;
    use std::hash::Hash;
    use std::path::{Path, PathBuf};
    use std::sync::Once;
    use tar::{Archive, Builder};
    use tempdir;

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

    static INIT: Once = Once::new();

    pub fn initialize() {
        INIT.call_once(|| {
            let mut ar = Builder::new(File::create("example.tar")
                .expect("File created"));
            ar.append_dir_all("example", "tests/example").unwrap();
            ar.finish().unwrap();
        });
    }

    pub fn get_temp_outpack_root() -> PathBuf {
        initialize();
        let tmp_dir = tempdir::TempDir::new("outpack").expect("Temp dir created");
        let mut ar = Archive::new(File::open("example.tar").unwrap());
        ar.unpack(&tmp_dir).expect("unwrapped");
        Path::new(&tmp_dir.into_path()).join("example")
    }
}
