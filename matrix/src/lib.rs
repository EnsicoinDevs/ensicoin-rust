mod protocol;
mod types;

pub use protocol::Http;

#[macro_export]
macro_rules! hash_map {
    ( $( $key:expr => $val:expr ),* ) => {
        {
            use std::collections::HashMap;
            let mut hash_map: HashMap<_, _> = HashMap::new();

            $(
                hash_map.insert($key, $val);
            )*

            hash_map
        }
    };
}
