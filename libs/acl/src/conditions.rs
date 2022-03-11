use fluxcd_utils_macros::str_enum;

str_enum! {
  /// These constants define the Condition types for when the GitOps Toolkit components perform ACL assertions.
  #[non_exhaustive]
  #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
  pub enum Condition {
    /// ReadyCondition indicates the resource is ready and fully reconciled.
    /// If the Condition is False, the resource SHOULD be considered to be in the process of reconciling and not a
    /// representation of actual state.
    AccessDenied = "AccessDenied",
  }
}

str_enum! {
  /// These constants define the Condition reasons for when the GitOps Toolkit components perform ACL assertions.
  #[non_exhaustive]
  #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
  pub enum Reason {
    /// AccessDeniedReason indicates that access to a resource has been denied by an ACL assertion.
    AccessDenied = "AccessDenied",
  }
}
