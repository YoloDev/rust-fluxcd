#[macro_export]
macro_rules! one_of {
  ($lit:literal $(,)?) => {
    concat!("'", $lit, "'")
  };
  ($first:literal, $second:literal $(,)?) => {
    concat!("either '", $first, "', or '", $second, "'")
  };
  (
    $first:literal, $($lit:literal),+$(,)?
  ) => {
    $crate::one_of!(@acc [$($lit)+] ["one of '" $first "'"])
  };
  (@acc [$last:literal] [$($acc:literal)+]) => {
    concat!($($acc,)+ ", or '", $last, "'")
  };
  (@acc [$next:literal $($lit:literal)+] [$($acc:literal)+]) => {
    $crate::one_of!(@acc [$($lit)+] [$($acc)+ ", '" $next "'"])
  };
}

#[macro_export]
macro_rules! str_enum {
  (
    $(#[$m:meta])*
    $vis:vis enum $name:ident {
      $(
        $(#[$var_m:meta])*
        $var_name:ident = $var_val:literal
      ),+$(,)?
    }
  ) => {
    $(#[$m])*
    $vis enum $name {
      $(
        $(#[$var_m])*
        $var_name,
      )+
    }

    impl ::core::fmt::Display for $name {
      fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        match self {
          $(
            Self::$var_name => f.write_str($var_val),
          )+
        }
      }
    }

    impl<'de> ::serde::Deserialize<'de> for $name {
      fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
      where
        D: ::serde::Deserializer<'de>,
      {
        struct Visitor;
        impl<'de> ::serde::de::Visitor<'de> for Visitor {
          type Value = $name;

          fn expecting(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.write_str($crate::one_of!($($var_val,)*))
          }

          fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
          where
            E: ::serde::de::Error,
          {
            match v {
              $(
                $var_val => Ok($name::$var_name),
              )*
              _ => Err(E::invalid_value(::serde::de::Unexpected::Str(v), &self)),
            }
          }
        }

        deserializer.deserialize_str(Visitor)
      }
    }

    impl ::serde::Serialize for $name {
      fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
      where
        S: ::serde::Serializer,
      {
        match self {
          $(
            Self::$var_name => $var_val.serialize(serializer),
          )+
        }
      }
    }

    impl TryFrom<&str> for $name {
      type Error = ();

      fn try_from(value: &str) -> Result<Self, ()> {
        match value {
          $(
            $var_val => Ok(Self::$var_name),
          )*
          _ => Err(()),
        }
      }
    }
  };
}

#[macro_export]
macro_rules! api_object {
  (
    $(#[$m:meta])*
    $vis:vis struct $name:ident {
      $(
        $(#[$fld_m:meta])*
        $fld_name:ident : $fld_ty:ty = $fld_api_name:literal
      ),*$(,)?
    }
  ) => {
    $(#[$m])*
    $vis struct $name {
      $(
        $(#[$fld_m])*
        $fld_name: Option<$fld_ty>,
      )+
    }

    impl<'de> ::serde::Deserialize<'de> for $name {
      fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
      where
        D: ::serde::Deserializer<'de>,
      {
        ::paste::paste! {
          #[allow(non_camel_case_types)]
          enum Field {
            $([<Key_ $fld_name>],)*
            Other,
          }

          impl<'de> ::serde::Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
              D: ::serde::Deserializer<'de>,
            {
              struct Visitor;

              impl<'de> ::serde::de::Visitor<'de> for Visitor {
                type Value = Field;

                fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                  f.write_str("field identifier")
                }

                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                  E: serde::de::Error,
                {
                  Ok(match v {
                    $($fld_api_name => Field::[<Key_ $fld_name>],)*
                    _ => Field::Other,
                  })
                }
              }

              ::serde::Deserializer::deserialize_identifier(deserializer, Visitor)
            }
          }

          struct Visitor;

          impl<'de> ::serde::de::Visitor<'de> for Visitor {
            type Value = $name;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
              f.write_str(stringify!($name))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
              A: serde::de::MapAccess<'de>,
            {
              $(
                let mut [<value_ $fld_name>]: Option<$fld_ty> = None;
              )*

              while let Some(key) = ::serde::de::MapAccess::next_key::<Field>(&mut map)? {
                match key {
                  $(
                    Field::[<Key_ $fld_name>] => [<value_ $fld_name>] = ::serde::de::MapAccess::next_value(&mut map)?,
                  )*
                  Field::Other => { let _: ::serde::de::IgnoredAny = ::serde::de::MapAccess::next_value(&mut map)?; },
                }
              }

              Ok($name {
                $(
                  $fld_name: [<value_ $fld_name>],
                )*
              })
            }
          }
        }

        <D as ::serde::Deserializer>::deserialize_struct(
          deserializer,
          stringify!($name),
          &[$($fld_api_name,)*],
          Visitor
        )
      }
    }

    impl ::serde::Serialize for $name {
      fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
      where
        S: ::serde::Serializer,
      {
        let mut state = <S as ::serde::Serializer>::serialize_struct(
          serializer,
          stringify!($name),
          0 $(
            + self.$fld_name.as_ref().map_or(0, |_| 1)
          )*
        )?;

        $(
          if let Some(value) = &self.$fld_name {
            ::serde::ser::SerializeStruct::serialize_field(&mut state, $fld_api_name, value)?;
          }
        )*

        ::serde::ser::SerializeStruct::end(state)
      }
    }
  };
}
