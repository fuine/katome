//! Miscallenous utility macros and functions.

// extern crate rustc_serialize;
// Following two macros have been copied from rustfmt sources
// See more in https://github.com/rust-lang-nursery/rustfmt

/// Create easily decodable enums for config.
macro_rules! impl_enum_decodable {
    ( $e:ident, $( $x:ident ),* ) => {
        impl ::rustc_serialize::Decodable for $e {
            fn decode<D: ::rustc_serialize::Decoder>(d: &mut D) -> Result<Self, D::Error> {
                use std::ascii::AsciiExt;
                let s = try!(d.read_str());
                $(
                    if stringify!($x).eq_ignore_ascii_case(&s) {
                      return Ok($e::$x);
                    }
                )*
                Err(d.error("Bad variant"))
            }
        }

        impl ::std::str::FromStr for $e {
            type Err = &'static str;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                use std::ascii::AsciiExt;
                $(
                    if stringify!($x).eq_ignore_ascii_case(s) {
                        return Ok($e::$x);
                    }
                )*
                Err("Bad variant")
            }
        }
    }
}

/// Create config enum
macro_rules! config_option_enum {
    ($(#[$str_attr:meta])+ $e:ident: $( $(#[$attr:meta])+  $x:ident ),+ $(,)*) => {
        #[derive(Copy, Clone, Eq, PartialEq, Debug)]
        $(#[$str_attr]),+
        pub enum $e {
            $( $(#[$attr]),+ $x ),+
        }

        impl_enum_decodable!($e, $( $x ),+);
    }
}
