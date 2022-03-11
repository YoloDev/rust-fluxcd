use fluxcd_acl::AccessFrom;
use fluxcd_meta::{Duration, ReconcileRequestStatus};
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[kube(
  group = "source.fluxcd.yolodev.io",
  version = "v1beta1",
  kind = "GitHubUserSshKeys",
  status = "GitHubUserSshKeysStatus",
  namespaced
)]
pub struct GitHubUserSshKeysSpec {
  /// GitHub user name.
  user: String,

  /// The interval at which to check for repository updates.
  interval: Duration,

  /// The timeout for fetching values, defaults to 60s.
  #[serde(skip_serializing_if = "Option::is_none", default)]
  timeout: Option<Duration>,

  /// Suspend tells the controller to suspend the reconciliation of this source.
  /// This flag tells the controller to suspend the reconciliation of this source.
  #[serde(skip_serializing_if = "std::ops::Not::not", default = "const_false")]
  suspend: bool,

  #[serde(skip_serializing_if = "Option::is_none", default)]
  access_from: Option<AccessFrom>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct GitHubUserSshKeysStatus {
  #[serde(flatten)]
  reconcile_request_status: ReconcileRequestStatus,
}

#[inline]
const fn const_false() -> bool {
  false
}
