use async_trait::async_trait;
use eyre::Result;
use fluxcd_api_source_github_keys::GitHubUserSshKeys;
use fluxcd_utils_cap::{metrics, Controller, ControllerApp};
use kube::runtime::controller::ReconcilerAction;

const CRATE_NAME: &str = env!("CARGO_PKG_NAME");
// const CRATE_DESC: &str = env!("CARGO_PKG_DESCRIPTION");
const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

struct GitHubUserSshKeysController {
  metrics: metrics::Recorder,
}

impl GitHubUserSshKeysController {
  pub fn new() -> Result<Self> {
    let metrics = metrics::Recorder::new()?;

    Ok(Self { metrics })
  }
}

#[async_trait]
impl Controller<GitHubUserSshKeys> for GitHubUserSshKeysController {
  async fn reconcile(
    self: std::sync::Arc<Self>,
    _resource: std::sync::Arc<GitHubUserSshKeys>,
  ) -> eyre::Result<ReconcilerAction> {
    todo!()
  }

  fn error_policy(self: std::sync::Arc<Self>, _error: &eyre::Report) -> ReconcilerAction {
    todo!()
  }

  fn metrics(&self) -> &metrics::Recorder {
    &self.metrics
  }
}

fn main() -> eyre::Result<()> {
  ControllerApp::main(CRATE_NAME, CRATE_VERSION, |app| {
    Ok(app.controller(GitHubUserSshKeysController::new()?))
  })
}
