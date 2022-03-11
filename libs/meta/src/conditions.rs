use fluxcd_utils_macros::str_enum;

str_enum! {
  /// These constants define generic Condition types to be used by GitOps Toolkit components.
  ///
  /// The ReadyCondition SHOULD be implemented by all components' Kubernetes resources to indicate they have been fully
  /// reconciled by their respective reconciler. This MAY suffice for simple resources, e.g. a resource that just declares
  /// state once and is not expected to receive any updates afterwards.
  ///
  /// For Kubernetes resources that are expected to receive spec updates over time, take a longer time to reconcile, or
  /// deal with more complex logic in which for example a finite error state can be observed, it is RECOMMENDED to
  /// implement the StalledCondition and ReconcilingCondition.
  ///
  /// By doing this, observers making use of kstatus to determine the current state of the resource will have a better
  /// experience while they are e.g. waiting for a change to be reconciled, and will be able to stop waiting for a change
  /// if a StalledCondition is observed, without having to rely on a timeout.
  ///
  /// For more information on kstatus, see:
  /// https://github.com/kubernetes-sigs/cli-utils/blob/v0.25.0/pkg/kstatus/README.md
  #[non_exhaustive]
  #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
  pub enum Condition {
    /// ReadyCondition indicates the resource is ready and fully reconciled.
    /// If the Condition is False, the resource SHOULD be considered to be in the process of reconciling and not a
    /// representation of actual state.
    Ready = "Ready",

    /// StalledCondition indicates the reconciliation of the resource has stalled, e.g. because the controller has
    /// encountered an error during the reconcile process or it has made insufficient progress (timeout).
    /// The Condition adheres to an "abnormal-true" polarity pattern, and MUST only be present on the resource if the
    /// Condition is True.
    /// For more information about polarity patterns, see:
    /// https://github.com/kubernetes/community/blob/master/contributors/devel/sig-architecture/api-conventions.md#typical-status-properties
    Stalled = "Stalled",

    /// ReconcilingCondition indicates the controller is currently working on reconciling the latest changes. This MAY be
    /// True for multiple reconciliation attempts, e.g. when an transient error occurred.
    /// The Condition adheres to an "abnormal-true" polarity pattern, and MUST only be present on the resource if the
    /// Condition is True.
    /// For more information about polarity patterns, see:
    /// https://github.com/kubernetes/community/blob/master/contributors/devel/sig-architecture/api-conventions.md#typical-status-properties
    Reconciling = "Reconciling",
  }
}

str_enum! {
  /// These constants define generic Condition reasons to be used by GitOps Toolkit components.
  ///
  /// Making use of a generic Reason is RECOMMENDED whenever it can be applied to a Condition in which it provides
  /// sufficient context together with the type to summarize the meaning of the Condition cause.
  ///
  /// Where any of the generic Condition reasons does not suffice, GitOps Toolkit components can introduce new reasons to
  /// their API specification, or use an arbitrary PascalCase string when setting the Condition.
  /// Declaration of domain common Condition reasons in the API specification is RECOMMENDED, as it eases observations
  /// for user and computer.
  ///
  /// For more information on Condition reason conventions, see:
  /// https://github.com/kubernetes/community/blob/master/contributors/devel/sig-architecture/api-conventions.md#typical-status-properties
  #[non_exhaustive]
  #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
  pub enum Reason {
    /// SucceededReason indicates a condition or event observed a success, for example when declared desired state
    /// matches actual state, or a performed action succeeded.
    ///
    /// More information about the reason of success MAY be available as additional metadata in an attached message.
    Succeeded = "Succeeded",

    /// FailedReason indicates a condition or event observed a failure, for example when declared state does not match
    /// actual state, or a performed action failed.
    ///
    /// More information about the reason of failure MAY be available as additional metadata in an attached message.
    Failed = "Failed",

    /// ProgressingReason indicates a condition or event observed progression, for example when the reconciliation of a
    /// resource or an action has started.
    ///
    /// When this reason is given, other conditions and types MAY no longer be considered as an up-to-date observation.
    /// Producers of the specific condition type or event SHOULD provide more information about the expectations and
    /// precise meaning in their API specification.
    ///
    /// More information about the reason or the current state of the progression MAY be available as additional metadata
    /// in an attached message.
    Progressing = "Progressing",

    /// SuspendedReason indicates a condition or event has observed a suspension, for
    /// example because a resource has been suspended, or a dependency is.
    Suspended = "Suspended",
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_test::*;

  #[test]
  fn condition_serde() {
    assert_tokens(&Condition::Ready, &[Token::Str("Ready")]);
    assert_tokens(&Condition::Stalled, &[Token::Str("Stalled")]);
    assert_tokens(&Condition::Reconciling, &[Token::Str("Reconciling")]);
  }
}
