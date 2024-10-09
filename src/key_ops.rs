use serde::{
    de::{self, Deserialize, Deserializer},
    ser::{Serialize, SerializeSeq, Serializer},
};
use std::fmt::Debug;

macro_rules! impl_key_ops {
    ($(($key_op:ident, $const_name:ident, $i:literal)),+,) => {
        bitflags::bitflags! {
            #[derive(Default)]
            pub struct KeyOps: u16 {
                $(const $const_name = $i;)*
            }
        }

        impl Debug for KeyOps {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut debug = f.debug_set();
                $(
                    if self.contains(KeyOps::$const_name) {
                        debug.entry(&stringify!($key_op));
                    }
                )+
                debug.finish()
            }
        }

        impl Clone for KeyOps {
            fn clone(&self) -> Self {
                KeyOps::from_bits_truncate(self.bits())
            }
        }

        impl std::cmp::Eq for KeyOps {}

        impl std::cmp::PartialEq for KeyOps {
            fn eq(&self, other: &Self) -> bool {
                self.bits() == other.bits()
            }
        }

        impl Serialize for KeyOps {
            fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                let mut seq = s.serialize_seq(Some(self.bits().count_ones() as usize))?;
                $(
                    if self.contains(KeyOps::$const_name) {
                        seq.serialize_element(stringify!($key_op))?;
                    }
                )+
                seq.end()
            }
        }

        impl<'de> Deserialize<'de> for KeyOps {
            fn deserialize<D: Deserializer<'de>>(d: D) -> Result<KeyOps, D::Error> {
                let op_strs: Vec<String> = Deserialize::deserialize(d)?;
                let mut ops = KeyOps::default();
                for op_str in op_strs {
                    $(
                        if op_str == stringify!($key_op) {
                            ops |= KeyOps::$const_name;
                            continue;
                        }
                    )+
                        return Err(de::Error::custom(&format!("invalid key op: `{}`", op_str)));
                }
                Ok(ops)
            }
        }
    };
}

#[rustfmt::skip]
impl_key_ops!(
    (sign,       SIGN,        0b00000001),
    (verify,     VERIFY,      0b00000010),
    (encrypt,    ENCRYPT,     0b00000100),
    (decrypt,    DECRYPT,     0b00001000),
    (wrapKey,    WRAP_KEY,    0b00010000),
    (unwrapKey,  UNWRAP_KEY,  0b00100000),
    (deriveKey,  DERIVE_KEY,  0b01000000),
    (deriveBits, DERIVE_BITS, 0b10000000),
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_invalid() {
        let result: Result<KeyOps, _> = serde_json::from_str(r#"["unknown"]"#);
        assert!(result.is_err());
    }

    #[test]
    fn serialize() {
        let ops = KeyOps::SIGN | KeyOps::DERIVE_BITS;
        let json = serde_json::to_string(&ops).unwrap();
        assert_eq!(json, r#"["sign","deriveBits"]"#)
    }

    #[test]
    fn debug() {
        let ops = KeyOps::SIGN | KeyOps::ENCRYPT | KeyOps::DERIVE_BITS;
        let debug_str = format!("{:?}", ops);
        assert_eq!(debug_str, r#"{"sign", "encrypt", "deriveBits"}"#);
    }

    #[test]
    fn clone_eq() {
        let ops = KeyOps::SIGN | KeyOps::VERIFY;
        let cloned_ops = ops.clone();
        assert_eq!(ops, cloned_ops);
    }
}
