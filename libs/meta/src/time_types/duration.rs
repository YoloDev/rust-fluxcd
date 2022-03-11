use schemars::{
  schema::{InstanceType, SchemaObject, StringValidation},
  JsonSchema,
};
use serde::{de, Deserialize, Deserializer, Serialize};
use std::{fmt, num::TryFromIntError, str::FromStr};
use thiserror::Error;

/// A Duration represents the elapsed time between two instants
/// as an i64 nanosecond count. The representation limits the
/// largest representable duration to approximately 290 years.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Duration(i64);

impl Duration {
  pub const ZERO: Duration = Duration(0i64);
  pub const MIN: Duration = Duration(i64::MIN);
  pub const MAX: Duration = Duration(i64::MAX);

  pub const NANOSECOND: Duration = Duration(1);
  pub const MICROSECOND: Duration = Duration(1000 * Duration::NANOSECOND.0);
  pub const MILLISECOND: Duration = Duration(1000 * Duration::MICROSECOND.0);
  pub const SECOND: Duration = Duration(1000 * Duration::MILLISECOND.0);
  pub const MINUTE: Duration = Duration(60 * Duration::SECOND.0);
  pub const HOUR: Duration = Duration(60 * Duration::MINUTE.0);
}

/// Display returns a string representing the duration in the form "72h3m0.5s".
/// Leading zero units are omitted. As a special case, durations less than one
/// second format use a smaller unit (milli-, micro-, or nanoseconds) to ensure
/// that the leading digit is non-zero. The zero duration formats as 0s.
impl fmt::Display for Duration {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if self.0 == 0 {
      return f.write_str("0s");
    }

    // Largest time is 2540400h10m10.000000000s
    let mut buf = Vec::with_capacity(32);
    let neg = self.0.is_negative();
    let u = self.0.unsigned_abs();

    if u < Duration::SECOND.0 as u64 {
      // Special case: if duration is smaller than a second,
      // use smaller units, like 1.2ms
      let prec: i32;
      buf.push(b's');

      if u < Duration::MICROSECOND.0 as u64 {
        // print nanoseconds
        prec = 0;
        buf.push(b'n');
      } else if u < Duration::MILLISECOND.0 as u64 {
        // print microseconds
        prec = 3;
        // U+00B5 'µ' micro sign == 0xC2 0xB5
        buf.push(0xB5);
        buf.push(0xC2);
      } else {
        // print milliseconds
        prec = 6;
        buf.push(b'm');
      }

      let u = formatting::frac(&mut buf, u, prec);
      formatting::int(&mut buf, u);
    } else {
      buf.push(b's');

      let u = formatting::frac(&mut buf, u, 9);

      // u is now integer seconds
      formatting::int(&mut buf, u % 60);
      let u = u / 60;

      // u is now integer minutes
      if u > 0 {
        buf.push(b'm');
        formatting::int(&mut buf, u % 60);
        let u = u / 60;

        // u is now integer hours
        // Stop at hours because days can be different lengths.
        if u > 0 {
          buf.push(b'h');
          formatting::int(&mut buf, u);
        }
      }
    }

    if neg {
      buf.push(b'-');
    }

    // This should never fail, as we only insert valid UTF-8 into the buffer
    buf.reverse();
    f.write_str(std::str::from_utf8(&*buf).unwrap())
  }
}

mod formatting {
  /// frac formats the fraction of v/10**prec (e.g., ".12345") into the
  /// tail of buf, omitting trailing zeros. It omits the decimal
  /// point too when the fraction is 0. It returns the index where the
  /// output bytes begin and the value v/10**prec.
  pub(super) fn frac(buf: &mut Vec<u8>, mut v: u64, prec: i32) -> u64 {
    // Omit trailing zeros up to and including decimal point.
    let mut print = false;
    let mut i = 0;
    while i < prec {
      i += 1;
      let digit = (v % 10) as u8;
      print = print || digit != 0;
      if print {
        buf.push(digit + b'0');
      }
      v /= 10;
    }

    if print {
      buf.push(b'.');
    }

    v
  }

  pub(super) fn int(buf: &mut Vec<u8>, mut v: u64) {
    if v == 0 {
      buf.push(b'0');
    } else {
      while v > 0 {
        let digit = (v % 10) as u8;
        buf.push(digit + b'0');
        v /= 10;
      }
    }
  }
}

impl Duration {
  /// Nanoseconds returns the duration as an integer nanosecond count.
  #[inline]
  pub const fn nanoseconds(&self) -> i64 {
    self.0
  }

  /// Microseconds returns the duration as an integer microsecond count.
  #[inline]
  pub const fn microseconds(&self) -> i64 {
    self.0 / 1_000
  }

  /// Milliseconds returns the duration as an integer millisecond count.
  #[inline]
  pub const fn milliseconds(&self) -> i64 {
    self.0 / 1_000_000
  }

  #[inline]
  pub fn whole_seconds(&self) -> i64 {
    self.0 / 1_000_000_000
  }

  #[inline]
  pub fn seconds(&self) -> f64 {
    let sec = (self.0 / Duration::SECOND.0) as f64;
    let nsec = (self.0 % Duration::SECOND.0) as f64;
    sec + (nsec / 1e9f64)
  }

  #[inline]
  pub fn minutes(&self) -> f64 {
    let min = (self.0 / Duration::MINUTE.0) as f64;
    let nsec = (self.0 % Duration::MINUTE.0) as f64;
    min + (nsec / (1e9f64 * 60f64))
  }

  #[inline]
  pub fn hours(&self) -> f64 {
    let min = (self.0 / Duration::HOUR.0) as f64;
    let nsec = (self.0 % Duration::HOUR.0) as f64;
    min + (nsec / (1e9f64 * 60f64 * 64f64))
  }
}

#[derive(Debug, Error)]
pub enum DurationParseError {
  #[error("invalid duration '{input}'")]
  Invalid { input: String },

  #[error("missing unit in duration '{input}'")]
  MissingUnit { input: String },

  #[error("unknown unit '{unit}' in duration '{input}'")]
  UnknownUnit { input: String, unit: String },
}

impl DurationParseError {
  pub fn input(&self) -> &str {
    match self {
      DurationParseError::Invalid { input } => &**input,
      DurationParseError::MissingUnit { input } => &**input,
      DurationParseError::UnknownUnit { input, .. } => &**input,
    }
  }
}

impl DurationParseError {
  fn invalid(input: &str) -> Self {
    Self::Invalid {
      input: input.into(),
    }
  }

  fn missing_unit(input: &str) -> Self {
    Self::MissingUnit {
      input: input.into(),
    }
  }

  fn unknown_unit(unit: &[u8], input: &str) -> Self {
    let mut unit_str = String::new();
    utf8::LossyDecoder::new(|s| unit_str.push_str(s)).feed(unit);

    Self::UnknownUnit {
      input: input.into(),
      unit: unit_str,
    }
  }
}

impl TryFrom<&str> for Duration {
  type Error = DurationParseError;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    fn unit_base(symbol: &[u8]) -> Option<i64> {
      match symbol {
        b"ns" => Some(Duration::NANOSECOND.0),
        b"us" => Some(Duration::MICROSECOND.0),
        b"\xC2\xB5s" => Some(Duration::MICROSECOND.0), // U+00B5 = micro symbol
        b"\xCE\xBCs" => Some(Duration::MICROSECOND.0), // U+03BC = Greek letter mu
        b"ms" => Some(Duration::MILLISECOND.0),
        b"s" => Some(Duration::SECOND.0),
        b"m" => Some(Duration::MINUTE.0),
        b"h" => Some(Duration::HOUR.0),
        _ => None,
      }
    }

    // leadingInt consumes the leading [0-9]* from s.
    fn leading_ints(s: &mut &[u8]) -> Option<u64> {
      let mut x = 0u64;
      let mut r = s.len();

      for (i, c) in s.iter().copied().enumerate() {
        if !(b'0'..=b'9').contains(&c) {
          r = i;
          break;
        }

        x = x.checked_mul(10u64)?.checked_add((c - b'0') as u64)?;
      }

      *s = &s[r..];
      Some(x)
    }

    // leadingFraction consumes the leading [0-9]* from s.
    // It is used only for fractions, so does not return an error on overflow,
    // it just stops accumulating precision.
    fn leading_fractions(s: &mut &[u8]) -> (u64, f64) {
      let mut x = 0u64;
      let mut r = s.len();
      let mut scale = 1f64;
      let mut overflow = false;

      for (i, c) in s.iter().copied().enumerate() {
        if !(b'0'..=b'9').contains(&c) {
          r = i;
          break;
        }

        if overflow {
          continue;
        }

        match x
          .checked_mul(10u64)
          .and_then(|x| x.checked_add((c - b'0') as u64))
        {
          None => {
            // overflow
            overflow = true;
            continue;
          }
          Some(y) => {
            x = y;
            scale *= 10f64;
          }
        }
      }

      *s = &s[r..];
      (x, scale)
    }

    // [-+]?([0-9]*(\.[0-9]*)?[a-z]+)+
    let mut s = value.as_bytes();
    let mut d = 0u64;

    // Consume [-+]?
    let neg = if !s.is_empty() {
      let c = s[0];
      if c == b'-' || c == b'+' {
        s = &s[1..];
        c == b'-'
      } else {
        false
      }
    } else {
      false
    };

    // Special case: if all that is left is "0", this is zero.
    if s == b"0" {
      return Ok(Duration::ZERO);
    }

    if s.is_empty() {
      return Err(DurationParseError::invalid(value));
    }

    while !s.is_empty() {
      // let mut v: i64; // integers before decimal point
      // let mut f: i64; // integers after decimal point
      // let mut scale = 1f64; // value = v + f/scale

      // The next character must be [0-9.]
      let c = s[0];
      if !(c == b'.' || (b'0'..=b'9').contains(&c)) {
        return Err(DurationParseError::invalid(value));
      }

      // Consume [0-9]*
      let pl = s.len();
      let v = leading_ints(&mut s).ok_or_else(|| DurationParseError::invalid(value))?;
      let pre = s.len() != pl; // whether we consumed anything before a period

      // Consume (\.[0-9]*)?
      let (f, scale, post) = if !s.is_empty() && s[0] == b'.' {
        s = &s[1..];
        let pl = s.len();
        let (f, scale) = leading_fractions(&mut s);
        let post = pl != s.len();
        (f, scale, post)
      } else {
        (0u64, 1f64, false)
      };

      if !pre && !post {
        // no digits (e.g. ".s" or "-.s")
        return Err(DurationParseError::invalid(value));
      }

      // Consume unit.
      let mut r = s.len();
      for (i, c) in s.iter().copied().enumerate() {
        if c == b'.' || (b'0'..=b'9').contains(&c) {
          r = i;
          break;
        }
      }

      if r == 0 {
        return Err(DurationParseError::missing_unit(value));
      }

      let u = &s[..r];
      s = &s[r..];
      let unit = unit_base(u).ok_or_else(|| DurationParseError::unknown_unit(u, value))?;
      let v = v
        .checked_mul(unit as u64)
        .ok_or_else(|| DurationParseError::invalid(value))?;

      let v = if f > 0 {
        // f64 is needed to be nanosecond accurate for fractions of hours.
        // v >= 0 && (f*unit/scale) <= 3.6e+12 (ns/h, h is the largest unit)
        let fraction_part = (f as f64) * ((unit as f64) / scale);
        v.checked_add(fraction_part as u64)
          .ok_or_else(|| DurationParseError::invalid(value))?
      } else {
        v
      };

      d = d
        .checked_add(v)
        .ok_or_else(|| DurationParseError::invalid(value))?;
    }

    match (neg, d) {
      // special case i64::MIN, as it's absolute value is bigger than i64::MAX, so we can't just negate it
      (true, d) if d == i64::unsigned_abs(i64::MIN) => Ok(Duration::MIN),
      (true, d) if d < i64::unsigned_abs(i64::MIN) => Ok(Duration(-(d as i64))),
      (false, d) if d <= i64::MAX as u64 => Ok(Duration(d as i64)),
      (_, _) => Err(DurationParseError::invalid(value)),
    }
  }
}

impl FromStr for Duration {
  type Err = <Duration as TryFrom<&'static str>>::Error;

  #[inline]
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Duration::try_from(s)
  }
}

struct Visitor;

impl<'de> de::Visitor<'de> for Visitor {
  type Value = Duration;

  fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str("Duration")
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Duration::try_from(v).map_err(|_| E::invalid_value(de::Unexpected::Str(v), &self))
  }
}

impl<'de> Deserialize<'de> for Duration {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_str(Visitor)
  }
}

impl Serialize for Duration {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    self.to_string().serialize(serializer)
  }
}

impl JsonSchema for Duration {
  fn is_referenceable() -> bool {
    false
  }

  fn schema_name() -> String {
    "Duration".to_owned()
  }

  fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    SchemaObject {
      instance_type: Some(InstanceType::String.into()),
      string: Some(Box::new(StringValidation {
        pattern: Some("[-+]?([0-9]*(\\.[0-9]*)?[^0-9\\.]+)+".into()),
        ..Default::default()
      })),
      ..Default::default()
    }
    .into()
  }
}

impl From<Duration> for time::Duration {
  fn from(value: Duration) -> Self {
    let seconds = value.whole_seconds();
    let nanos = value.0 - (seconds * Duration::SECOND.0);

    time::Duration::new(seconds, nanos as i32)
  }
}

impl TryFrom<time::Duration> for Duration {
  type Error = TryFromIntError;

  fn try_from(value: time::Duration) -> Result<Self, Self::Error> {
    let millis = value.whole_nanoseconds();
    let millis = i64::try_from(millis)?;

    Ok(Duration(millis))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_case::test_case;

  #[test_case("0s", 0)]
  #[test_case("1ns", Duration::NANOSECOND.0)]
  #[test_case("1.1µs", 1100 * Duration::NANOSECOND.0)]
  #[test_case("2.2ms", 2200 * Duration::MICROSECOND.0)]
  #[test_case("3.3s", 3300 * Duration::MILLISECOND.0)]
  #[test_case("4m5s", 4 * Duration::MINUTE.0 + 5 * Duration::SECOND.0)]
  #[test_case("4m5.001s", 4 * Duration::MINUTE.0 + 5001 * Duration::MILLISECOND.0)]
  #[test_case("5h6m7.001s", 5 * Duration::HOUR.0 + 6 * Duration::MINUTE.0 + 7001 * Duration::MILLISECOND.0)]
  #[test_case("8m0.000000001s", 8 * Duration::MINUTE.0 + Duration::NANOSECOND.0)]
  #[test_case("2562047h47m16.854775807s", Duration::MAX.0)]
  #[test_case("-2562047h47m16.854775808s", Duration::MIN.0)]
  fn duration_string_representation(string: &str, duration: i64) {
    let duration = Duration(duration);

    assert_eq!(string, &*duration.to_string());

    let parsed = Duration::from_str(string).expect("should parse");
    assert_eq!(duration, parsed);

    let round_tripped = Duration::try_from(time::Duration::from(duration)).expect("round trip");
    assert_eq!(round_tripped, duration);
  }
}
