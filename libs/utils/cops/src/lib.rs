pub mod metrics;

use async_trait::async_trait;
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::{
  api::ListParams,
  core::Resource as KubeResource,
  runtime::{controller::ReconcilerAction, Controller as KubeController},
  Api, Client, CustomResourceExt,
};
use metrics::Recorder;
use serde::Deserialize;
use std::{fmt, hash, sync::Arc};

#[async_trait]
pub trait Controller<Resource>
where
  Resource: CustomResourceExt
    + Clone
    + KubeResource
    + fmt::Debug
    + Send
    + Sync
    + for<'de> Deserialize<'de>
    + 'static,
  <Resource as KubeResource>::DynamicType: Eq + hash::Hash + Default + Clone,
{
  fn metrics(&self) -> &Recorder;

  async fn reconcile(self: Arc<Self>, resource: Arc<Resource>) -> eyre::Result<ReconcilerAction>;
  fn error_policy(self: Arc<Self>, error: &eyre::Report) -> ReconcilerAction;

  fn crd() -> CustomResourceDefinition {
    Resource::crd()
  }

  fn configure(self: Arc<Self>, controller: KubeController<Resource>) -> KubeController<Resource> {
    controller
  }

  fn create(client: Client) -> KubeController<Resource> {
    let api = Api::<Resource>::all(client);
    KubeController::new(api, ListParams::default())
  }
}
