macro_rules! embed_compressed {
    ($($file:tt,)*) => {
        /// This function will decompress the selected resource on every call, so it shouldn't be
        /// called more than once per resource if possible
        pub fn get_decompress(res: &str) -> Vec<u8> {
            let dat = match res {
                $(
                    $file => {
                        include_bytes!(concat!(env!("OUT_DIR"), "/compressed/", $file)) as &[u8]
                    }
                )*
                _ => panic!("Resource not found '{}'", res),
            };

            zstd::decode_all(dat).unwrap()
        }
    };
}

include!(concat!(env!("OUT_DIR"), "/resource_constants.rs"));
