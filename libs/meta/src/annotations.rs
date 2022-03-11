use fluxcd_utils_macros::api_object;
use schemars::JsonSchema;
use std::collections::BTreeMap;

/// ReconcileRequestAnnotation is the annotation used for triggering a reconciliation
/// outside of a defined schedule. The value is interpreted as a token, and any change
/// in value SHOULD trigger a reconciliation.
pub const RECONCILE_REQUEST_ANNOTATION: &str = "reconcile.fluxcd.io/requestedAt";

/// ReconcileAnnotationValue returns a value for the reconciliation request annotation, which can be used to detect
/// changes; and, a boolean indicating whether the annotation was set.
pub fn get_reconcile_annotation_value(annotations: &BTreeMap<String, String>) -> Option<&String> {
  annotations.get(RECONCILE_REQUEST_ANNOTATION)
}

api_object! {
  /// ReconcileRequestStatus is a struct to embed in a status type, so that all types using the mechanism have the same
  /// field.
  #[derive(Default, PartialEq, Hash, Debug, Clone, JsonSchema)]
  pub struct ReconcileRequestStatus {
    /// LastHandledReconcileAt holds the value of the most recent
    /// reconcile request value, so a change of the annotation value
    /// can be detected.
    last_handled_reconcile_at: String = "lastHandledReconcileAt",
  }
}

impl ReconcileRequestStatus {
  /// GetLastHandledReconcileRequest returns the most recent reconcile request value from the ReconcileRequestStatus.
  pub fn get_last_handled_reconcile_request(&self) -> Option<&str> {
    self.last_handled_reconcile_at.as_deref()
  }

  /// SetLastHandledReconcileRequest sets the most recent reconcile request value in the ReconcileRequestStatus.
  pub fn set_last_handled_reconcile_request(&mut self, token: Option<impl Into<String>>) {
    self.last_handled_reconcile_at = token.map(Into::into);
  }
}

/// StatusWithHandledReconcileRequest describes a status type which holds the value of the most recent
/// ReconcileAnnotationValue.
pub trait StatusWithHandledReconcileRequest {
  fn get_last_handled_reconcile_request(&self) -> Option<&str>;
}

/// StatusWithHandledReconcileRequestSetter describes a status with a setter for the most ReconcileAnnotationValue.
pub trait StatusWithHandledReconcileRequestSetter: StatusWithHandledReconcileRequest {
  fn set_last_handled_reconcile_request(&mut self, token: Option<impl Into<String>>);
}

impl StatusWithHandledReconcileRequest for ReconcileRequestStatus {
  #[inline]
  fn get_last_handled_reconcile_request(&self) -> Option<&str> {
    self.get_last_handled_reconcile_request()
  }
}

impl StatusWithHandledReconcileRequestSetter for ReconcileRequestStatus {
  #[inline]
  fn set_last_handled_reconcile_request(&mut self, token: Option<impl Into<String>>) {
    self.set_last_handled_reconcile_request(token)
  }
}

#[cfg(test)]
mod tests {
  use std::time::SystemTime;

  use super::*;
  use time::{self, format_description::well_known::Rfc3339, OffsetDateTime};

  struct WhateverStatus {
    reconsiler_request_status: ReconcileRequestStatus,
  }

  struct Whatever {
    annotations: BTreeMap<String, String>,
    status: WhateverStatus,
  }

  impl StatusWithHandledReconcileRequest for WhateverStatus {
    fn get_last_handled_reconcile_request(&self) -> Option<&str> {
      self
        .reconsiler_request_status
        .get_last_handled_reconcile_request()
    }
  }

  impl StatusWithHandledReconcileRequestSetter for WhateverStatus {
    fn set_last_handled_reconcile_request(&mut self, token: Option<impl Into<String>>) {
      self.reconsiler_request_status = ReconcileRequestStatus {
        last_handled_reconcile_at: token.map(Into::into),
      }
    }
  }

  fn now_string() -> String {
    let now = SystemTime::now();
    let odt = OffsetDateTime::from(now);
    odt.format(&Rfc3339).expect("valid date")
  }

  #[test]
  fn test_get_annotation_value() {
    let mut obj = Whatever {
      annotations: (BTreeMap::new()),
      status: (WhateverStatus {
        reconsiler_request_status: ReconcileRequestStatus::default(),
      }),
    };

    let val = get_reconcile_annotation_value(&obj.annotations);
    assert_eq!(
      val, None,
      "expected ReconcileAnnotationValue to return zero value and false when no annotations"
    );

    obj.status.set_last_handled_reconcile_request(val);

    obj
      .annotations
      .insert(RECONCILE_REQUEST_ANNOTATION.into(), now_string());

    let val = get_reconcile_annotation_value(&obj.annotations);
    assert_ne!(
      val, None,
      "expected ReconcileAnnotationValue to return true when an annotation is set"
    );

    assert_ne!(
      val.map(|s| &**s),
      obj.status.get_last_handled_reconcile_request(),
      "expected to detect change in annotation value"
    );

    obj.status.set_last_handled_reconcile_request(val);
    obj
      .annotations
      .insert(RECONCILE_REQUEST_ANNOTATION.into(), now_string());

    let val = get_reconcile_annotation_value(&obj.annotations);
    assert_ne!(
      val, None,
      "expected ReconcileAnnotationValue to return true when an annotation is set"
    );

    assert_ne!(
      val.map(|s| &**s),
      obj.status.get_last_handled_reconcile_request(),
      "expected to detect change in annotation value"
    );
  }
}
