use fluxcd_utils_macros::api_object;

api_object! {
  /// LocalObjectReference contains enough information to locate the referenced Kubernetes resource object.
  #[derive(Default, PartialEq, Hash, Debug, Clone)]
  pub struct LocalObjectReference {
    /// Name of the referent.
    name: String = "name",
  }
}

api_object! {
  /// NamespacedObjectReference contains enough information to locate the referenced Kubernetes resource object in any
  /// namespace.
  #[derive(Default, PartialEq, Hash, Debug, Clone)]
  pub struct NamespacedObjectReference {
    /// Name of the referent.
    name: String = "name",

    /// Namespace of the referent, when not specified it acts as LocalObjectReference.
    namespace: String = "namespace",
  }
}

api_object! {
  /// NamespacedObjectReference contains enough information to locate the referenced Kubernetes resource object in any
  /// namespace.
  #[derive(Default, PartialEq, Hash, Debug, Clone)]
  pub struct NamespacedObjectKindReference {
    /// API version of the referent, if not specified the Kubernetes preferred version will be used.
    api_version: String = "apiVersion",

    /// Kind of the referent.
    kind: String = "kind",

    /// Name of the referent.
    name: String = "name",

    /// Namespace of the referent, when not specified it acts as LocalObjectReference.
    namespace: String = "namespace",
  }
}
