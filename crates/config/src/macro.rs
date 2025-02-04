macro_rules! config_enum_case_ignore {
    (
        pub enum $name:ident {
            $(
                $variant:ident
            ),*
        }
    ) => {
        #[derive(Copy, Clone, Debug, serde::Serialize)]
        pub enum $name {
            $(
                $variant,
            )*
        }

        impl From<$name> for config::ValueKind {
            fn from(value: $name) -> Self {
                let str = match value {
                    $(
                        $name::$variant => stringify!($variant),
                    )*
                };

                Self::String(str.to_owned())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
                match String::deserialize(deserializer)?.to_uppercase().as_str() {
                    $(
                        stringify!($variant) => Ok($name::$variant),
                    )*
                    val => Err(
                        D::Error::custom(
                            EnumError::new(
                                stringify!($name).to_owned(),
                                vec![$(stringify!($variant).to_owned(),)*],
                                val.to_owned()
                            )
                        )
                    ),
                }
            }
        }
    };
}

pub(crate) use config_enum_case_ignore;
