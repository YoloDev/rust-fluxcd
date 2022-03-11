use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// AccessFrom defines an ACL for allowing cross-namespace references to a source object
/// based on the caller's namespace labels.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct AccessFrom {
  /// NamespaceSelectors is the list of namespace selectors to which this ACL applies.
  /// Items in this list are evaluated using a logical OR operation.
  #[serde(rename = "namespaceSelectors")]
  pub namespace_selectors: Vec<NamespaceSelector>,
}

/// NamespaceSelector selects the namespaces to which this ACL applies.
/// An empty map of MatchLabels matches all namespaces in a cluster.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct NamespaceSelector {
  /// MatchLabels is a map of {key,value} pairs. A single {key,value} in the matchLabels
  /// map is equivalent to an element of matchExpressions, whose key field is "key", the
  /// operator is "In", and the values array contains only "value". The requirements are ANDed.
  #[serde(
    rename = "matchLabels",
    skip_serializing_if = "BTreeMap::is_empty",
    default
  )]
  pub match_labels: BTreeMap<String, String>,
}
