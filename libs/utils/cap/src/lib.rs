mod cli;
mod signals;

use eyre::Report;
use futures::{Future, Stream, StreamExt, TryFutureExt};
use k8s_openapi::apiextensions_apiserver::pkg::apis::apiextensions::v1::CustomResourceDefinition;
use kube::{
  core::DynamicObject,
  runtime::{
    controller::{self, Context, ReconcilerAction},
    reflector::ObjectRef,
    watcher,
  },
  Client, CustomResourceExt, Resource,
};
use serde::Deserialize;
use std::{fmt, hash, pin::Pin, sync::Arc};
use tokio::runtime::Runtime;
use tracing::{info, warn};

pub use fluxcd_utils_cops::metrics;
pub use fluxcd_utils_cops::Controller;

pub struct ReportWrapper(Report);

impl std::fmt::Display for ReportWrapper {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    <Report as std::fmt::Display>::fmt(&self.0, f)
  }
}

impl std::fmt::Debug for ReportWrapper {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    <Report as std::fmt::Debug>::fmt(&self.0, f)
  }
}

impl std::error::Error for ReportWrapper {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    self.0.source()
  }
}

impl ReportWrapper {
  pub fn into_inner(self) -> Report {
    self.0
  }
}

type ShutdownSignalFuture = Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>;
type ReconcilerSuccessResult = (ObjectRef<DynamicObject>, ReconcilerAction);
type ReconcilerErrorResult = controller::Error<ReportWrapper, watcher::Error>;
type ReconcilerResult = Result<ReconcilerSuccessResult, ReconcilerErrorResult>;
type ReconcilerStream<'a> = Pin<Box<dyn Stream<Item = ReconcilerResult> + 'a>>;

type DynControllerCrd<'a> = Box<dyn FnOnce() -> CustomResourceDefinition + 'a>;
type DynControllerFactory<'a> =
  Box<dyn FnOnce(Client, ShutdownSignalFuture) -> ReconcilerStream<'a> + 'a>;

struct ControllerResourceInfo {
  group: Arc<str>,
  kind: Arc<str>,
  // version: String,
  // api_version: String,
}

struct DynController<'a> {
  info: ControllerResourceInfo,
  crd: DynControllerCrd<'a>,
  factory: DynControllerFactory<'a>,
}

impl<'a> DynController<'a> {
  fn crd(self) -> CustomResourceDefinition {
    let crd = self.crd;
    crd()
  }

  fn new<C, R>(controller: C) -> Self
  where
    C: Controller<R> + 'static,
    R: CustomResourceExt
      + Clone
      + Resource
      + fmt::Debug
      + Send
      + Sync
      + for<'de> Deserialize<'de>
      + 'static,
    <R as Resource>::DynamicType: Eq + hash::Hash + Default + Clone + fmt::Debug + Unpin,
  {
    let info = {
      let dt = <R as Resource>::DynamicType::default();

      ControllerResourceInfo {
        group: <R as Resource>::group(&dt).into(),
        kind: <R as Resource>::kind(&dt).into(),
        // version: <R as Resource>::version(&dt).into(),
        // api_version: <R as Resource>::api_version(&dt).into(),
      }
    };

    let crd: DynControllerCrd<'a> = Box::new(|| C::crd());
    let kind = info.kind.clone();
    let factory: DynControllerFactory<'a> = Box::new(move |client, signal| {
      let ctxt = Context::new(controller);
      let ctrl = C::create(client);
      let ctrl = C::configure(ctxt.clone().into_inner(), ctrl);

      let reconciler = {
        let kind = kind.clone();
        move |resource: Arc<R>, ctx: Context<C>| {
          let meta = resource.meta();
          let name = meta.name.as_deref().unwrap_or("<NULL>");
          let namespace = meta.namespace.as_deref().unwrap_or("<NULL>");
          let _span = tracing::info_span!("reconcile", controller.kind = %kind, resource.namespace = %namespace, resource.name = %name);
          info!("reconcile...");
          C::reconcile(ctx.into_inner(), resource).map_err(ReportWrapper)
        }
      };
      let error_policy = {
        // let kind = kind.clone();
        move |error: &ReportWrapper, ctx: Context<C>| {
          let _span = tracing::info_span!("error_policy", controller.kind = %kind);
          C::error_policy(ctx.into_inner(), &error.0)
        }
      };

      let stream = ctrl
        .graceful_shutdown_on(signal)
        .run(reconciler, error_policy, ctxt)
        .map(|result| match result {
          Ok((obj, action)) => Ok((obj.erase(), action)),
          Err(e) => Err(e),
        });

      Box::pin(stream)
    });

    DynController { info, crd, factory }
  }
}

pub struct ControllerApp<'a> {
  controllers: Vec<DynController<'a>>,
}

impl<'a> ControllerApp<'a> {
  fn new() -> Self {
    Self {
      controllers: Vec::new(),
    }
  }

  pub fn controller<C, R>(mut self, controller: C) -> Self
  where
    C: fluxcd_utils_cops::Controller<R> + 'static,
    R: CustomResourceExt
      + Clone
      + Resource
      + fmt::Debug
      + Send
      + Sync
      + for<'de> Deserialize<'de>
      + 'static,
    <R as Resource>::DynamicType: Eq + hash::Hash + Default + Clone + fmt::Debug + Unpin,
  {
    self.controllers.push(DynController::new(controller));
    self
  }

  async fn run(self, name: &str, version: &str) -> eyre::Result<()> {
    cli::run(name, version, self.controllers).await
  }

  pub fn main(
    name: &str,
    version: &str,
    setup: impl for<'b> FnOnce(ControllerApp<'b>) -> eyre::Result<ControllerApp<'b>>,
  ) -> eyre::Result<()> {
    let app = Self::new();
    let app = setup(app)?;

    let rt = Runtime::new()?;
    rt.block_on(async {
      fluxcd_utils_telemetry::setup()?;
      let result = app.run(name, version).await;
      fluxcd_utils_telemetry::teardown();

      result
    })?;

    Ok(())
  }
}
