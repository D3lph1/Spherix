pub mod serverbound;
pub mod clientbound;

/// The macro is also generates simple synthetic tests for each packet.
macro_rules! packet {
    (
        $(
            $packet:ident {
                $($name:ident: $t:ty),*
            }
        )*
    ) => {
        $(
            crate::packet::packet_impl!{
                $packet {
                    $($name: $t),*
                }
            }

            crate::packet::packet_debug_impl!{
                $packet {
                    $($name: $t),*
                }
            }

            impl crate::io::Readable for $packet {
                #[allow(unused_variables)]
                fn read<R: std::io::Read>(buf: &mut R) -> Result<$packet, crate::io::Error> {
                    Ok($packet {
                        $($name: <$t>::read(buf)?,)*
                    })
                }
            }

            impl crate::io::Writable for $packet {
                #[allow(unused_variables, unused_mut)]
                fn write<W: std::io::Write>(&self, buf: &mut W) -> Result<usize, crate::io::Error> {
                    let mut written: usize = 0;

                    $(written += self.$name.write(buf)?;)*

                    Ok(written)
                }
            }
        )*

        paste::paste!{
            $(
                #[cfg(test)]
                mod [< tests_ $packet:snake >] {
                    use std::io::Cursor;

                    use super::$packet;
                    use crate::io::Writable;
                    use crate::io::Readable;

                    #[test]
                    fn [< $packet:snake >]() {
                        crate::packet::tests::synthetic_test::<$packet>();
                    }
                }
            )*
        }
    };
}

macro_rules! packet_wo_io {
    (
        $(
            $packet:ident {
                $($name:ident: $t:ty),*
            }
        )*
    ) => {
        $(
            crate::packet::packet_impl!{
                $packet {
                    $($name: $t),*
                }
            }

            crate::packet::packet_debug_impl!{
                $packet {
                    $($name: $t),*
                }
            }
        )*
    };
}

macro_rules! packet_impl {
    (
        $packet:ident {
            $($name:ident: $t:ty),*
        }
    ) => {
        #[derive(Clone)]
        #[cfg_attr(test, derive(PartialEq, Default))]
        pub struct $packet {
            $(pub $name: $t),*
        }
    };
}

macro_rules! packet_debug_impl {
    (
        $packet:ident {
            $($name:ident: $t:ty),*
        }
    ) => {
        impl core::fmt::Debug for $packet {
                #[allow(unused_mut)]
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
                    let mut v: Vec<String> = Vec::new();

                    $(
                        v.push(format!(
                            "{}: {}",
                            stringify!($name),
                            owo_colors::OwoColorize::bright_black(
                                &format!("{}", spherix_macro::type_debug!(self.$name, $t))
                            )
                        ));
                    )*

                    if v.len() > 0 {
                        write!(f, "{} {{ {} }}", owo_colors::OwoColorize::italic(&stringify!($packet)), v.join(", "))
                    } else {
                        write!(f, "{}", owo_colors::OwoColorize::italic(&stringify!($packet)))
                    }
                }
            }
    };
}

macro_rules! packet_clientbound {
    (
        $enum_ident:ident {
            $($packet_id:literal = $packet:ident),*
        }
    ) => {
        pub enum $enum_ident {
            $($packet($packet),)*
        }

        impl $enum_ident {
            pub fn name(&self) -> String {
                match self {
                    $(
                        $enum_ident::$packet(p) => stringify!($packet).to_owned(),
                    )*
                }
            }
        }

        impl core::fmt::Debug for $enum_ident {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
                let s = match self {
                   $(
                        $enum_ident::$packet(p) => format!(
                            "{} {:?}",
                            owo_colors::OwoColorize::bright_black(&format!("{:#04x}", $packet_id)),
                            p
                        ),
                    )*
                };

                write!(f, "{}", s)
            }
        }

        impl crate::io::Readable for $enum_ident {
            fn read<R: std::io::Read>(buf: &mut R) -> Result<Self, crate::io::Error> {
                let packet_id = crate::io::VarInt::read(buf)?;

                match packet_id.0 {
                    $(
                        $packet_id => Ok($enum_ident::$packet($packet::read(buf)?)),
                    )*
                    x => Err(crate::io::Error::InvalidPacketId(crate::io::VarInt(x)))
                }
            }
        }

        impl crate::io::Writable for $enum_ident {
            fn write<W: std::io::Write>(&self, buf: &mut W) -> Result<usize, crate::io::Error> {
                match self {
                    $(
                        $enum_ident::$packet(p) => {
                            let written = crate::io::VarInt($packet_id).write(buf)?;
                            Ok(written + p.write(buf)?)
                        }
                    )*
                }
            }
        }
    }
}

macro_rules! packet_serverbound {
    (
        $enum_ident:ident {
            $($packet_id:literal = $packet:ident),*
        }
    ) => {
        crate::packet::packet_clientbound!(
            $enum_ident {
                $($packet_id = $packet),*
            }
        );
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::io::Cursor;

    use crate::io::{Readable, Writable};

    pub(crate) fn synthetic_test<P>() where P: Default + Readable + Writable + PartialEq + Debug {
        let mut buf = Vec::new();

        let written_packet = P::default();
        let written = written_packet.write(&mut buf).unwrap();
        let buf_len = buf.len();

        let mut cursor = Cursor::new(buf);

        let read_packet = P::read(&mut cursor).unwrap();

        assert_eq!(written_packet, read_packet);
        assert_eq!(written, buf_len);
    }
}

pub(crate) use packet;
pub(crate) use packet_impl;
pub(crate) use packet_wo_io;
pub(crate) use packet_debug_impl;
pub(crate) use packet_clientbound;
pub(crate) use packet_serverbound;
