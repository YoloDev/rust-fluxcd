use k8s_openapi::{api::core::v1::ObjectReference, apimachinery::pkg::apis::meta::v1::Condition};
use prometheus::{
  core::Collector, exponential_buckets, GaugeVec, HistogramOpts, HistogramTimer, HistogramVec, Opts,
};

pub struct Recorder {
  condition: GaugeVec,
  suspend: GaugeVec,
  duration: HistogramVec,
}

macro_rules! reconcile_metric {
  (gauge, $name:literal, $help:literal, [$($label:literal),*$(,)?]$(,)?) => {{
    let opts = Opts::new($name, $help)
      .subsystem("reconcile")
      .namespace("gotk");

    <GaugeVec>::new(opts, &[$($label,)*])
  }};

  (histogram, $name:literal, $help:literal, $buckets:expr, [$($label:literal),*$(,)?]$(,)?) => {{
    let opts = HistogramOpts::new($name, $help)
      .subsystem("reconcile")
      .namespace("gotk")
      .buckets($buckets);

    <HistogramVec>::new(opts, &[$($label,)*])
  }};
}

impl Recorder {
  pub fn new() -> eyre::Result<Self> {
    Ok(Self {
      condition: reconcile_metric!(
        gauge,
        "condition",
        "The current condition status of a GitOps Toolkit resource reconciliation.",
        ["kind", "name", "namespace", "type", "status"],
      )?,

      suspend: reconcile_metric!(
        gauge,
        "status",
        "The current suspend status of a GitOps Toolkit resource.",
        ["kind", "name", "namespace"],
      )?,

      duration: reconcile_metric!(
        histogram,
        "duration_seconds",
        "The duration in seconds of a GitOps Toolkit resource reconciliation.",
        exponential_buckets(10e-9, 10f64, 10)?,
        ["kind", "name", "namespace"],
      )?,
    })
  }
}

impl Collector for Recorder {
  fn desc(&self) -> Vec<&prometheus::core::Desc> {
    let mut result = Vec::new();
    result.extend(self.condition.desc());
    result.extend(self.suspend.desc());
    result.extend(self.duration.desc());

    result
  }

  fn collect(&self) -> Vec<prometheus::proto::MetricFamily> {
    let mut result = Vec::new();
    result.extend(self.condition.collect());
    result.extend(self.suspend.collect());
    result.extend(self.duration.collect());

    result
  }
}

impl Recorder {
  pub fn record_condition(&self, obj: &ObjectReference, condition: &Condition, deleted: bool) {
    let kind = obj.kind.as_deref().unwrap_or_default();
    let name = obj.name.as_deref().unwrap_or_default();
    let namespace = obj.namespace.as_deref().unwrap_or_default();
    let ty = &*condition.type_;

    let record = |status: &str, value: bool| {
      self
        .condition
        .with_label_values(&[kind, name, namespace, ty, status])
        .set(if value { 1f64 } else { 0f64 })
    };

    record("True", !deleted && condition.status == "True");
    record("False", !deleted && condition.status == "False");
    record("Unknown", !deleted && condition.status == "Unknown");
    record("Deleted", deleted);
  }

  pub fn record_suspend(&self, obj: &ObjectReference, suspend: bool) {
    let kind = obj.kind.as_deref().unwrap_or_default();
    let name = obj.name.as_deref().unwrap_or_default();
    let namespace = obj.namespace.as_deref().unwrap_or_default();
    let value = if suspend { 1f64 } else { 0f64 };

    self
      .suspend
      .with_label_values(&[kind, name, namespace])
      .set(value);
  }

  pub fn record_duration(&self, obj: &ObjectReference) -> HistogramTimer {
    let kind = obj.kind.as_deref().unwrap_or_default();
    let name = obj.name.as_deref().unwrap_or_default();
    let namespace = obj.namespace.as_deref().unwrap_or_default();

    self
      .duration
      .with_label_values(&[kind, name, namespace])
      .start_timer()
  }
}
