use clap::{Parser, Subcommand};

use crate::DynController;

#[derive(Parser)]
struct Cli {
  #[clap(subcommand)]
  command: Command,
}

impl Cli {
  async fn run(self, controllers: Vec<DynController<'_>>) -> eyre::Result<()> {
    self.command.run(controllers).await
  }
}

#[derive(Subcommand, Debug)]
#[clap(arg_required_else_help = true)]
pub enum Command {
  Crd {
    /// Print all crds to stdout
    #[clap(short, long, exclusive = true)]
    all: bool,

    /// Name or full path of CRD
    #[clap(exclusive = true)]
    name: Option<String>,

    #[clap(subcommand)]
    command: Option<CrdCommand>,
  },
}

impl Command {
  async fn run(self, controllers: Vec<DynController<'_>>) -> eyre::Result<()> {
    match self {
      Command::Crd { all: true, .. } => todo!(),
      Command::Crd {
        name: Some(crd), ..
      } => {
        let crd = controllers.into_iter().find(|c| {
          let info = &c.info;
          let group = &*info.group;
          let kind = &*info.kind;
          kind == crd || format!("{group}/{kind}") == crd
        });

        match crd {
          None => todo!("not found error message"),
          Some(c) => {
            let crd = c.crd();
            let yaml = serde_yaml::to_string(&crd).unwrap();

            println!("{yaml}");
            Ok(())
          }
        }
      }
      Command::Crd {
        command: Some(cmd), ..
      } => cmd.run(controllers).await,
      _ => todo!("{:?}", self),
    }
  }
}

#[derive(Subcommand, Debug)]
pub enum CrdCommand {
  /// List all CRDs
  List,
}

impl CrdCommand {
  async fn run(self, controllers: Vec<DynController<'_>>) -> eyre::Result<()> {
    match self {
      CrdCommand::List => {
        for ctrl in controllers {
          let info = ctrl.info;
          let group = info.group;
          let kind = info.kind;
          println!("{group}/{kind}");
        }
        Ok(())
      }
    }
  }
}

pub(crate) async fn run<'a>(
  name: &str,
  version: &str,
  controllers: Vec<DynController<'a>>,
) -> eyre::Result<()> {
  let cmd = clap::Command::new(name).version(version);
  let cmd = <Cli as clap::Args>::augment_args(cmd);

  let args = cmd.clone().get_matches();
  let parsed = <Cli as clap::FromArgMatches>::from_arg_matches(&args)?;

  parsed.run(controllers).await
}
