use std::marker::PhantomData;

/// A serializer that uses `serde` and `serde_json` to serialize the test
/// inputs (of arbitrary type `T: Serializable + for<'e> Deserializable<'e>`)
/// to a json file.
#[doc(cfg(feature = "serde_json_serializer"))]
pub struct SerdeSerializer<S> {
    phantom: PhantomData<S>,
}

impl<S> Default for SerdeSerializer<S> {
    #[no_coverage]
    fn default() -> Self {
        Self { phantom: PhantomData }
    }
}

impl<S> crate::traits::Serializer for SerdeSerializer<S>
where
    S: serde::Serialize + for<'e> serde::Deserialize<'e>,
{
    type Value = S;

    #[no_coverage]
    fn extension(&self) -> &str {
        "json"
    }
    #[no_coverage]
    fn from_data(&self, data: &[u8]) -> Option<S> {
        serde_json::from_slice(data).ok()
    }
    #[no_coverage]
    fn to_data(&self, value: &Self::Value) -> Vec<u8> {
        match serde_json::to_vec(value) {
            Ok(ret) => ret,
            Err(_) => {
                let ret = format!("{{\"Err\":{:#04X?}}}", bincode::serialize(value).unwrap());
                ret.into_bytes()
            }
        }
    }
}
