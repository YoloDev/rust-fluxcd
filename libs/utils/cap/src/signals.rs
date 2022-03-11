use crate::ShutdownSignalFuture;
use futures::{
  future::{ready, Shared},
  FutureExt, Stream, StreamExt,
};
use signal_hook_tokio::Signals;
use std::{convert::TryFrom, fmt, future::Future, io, pin::Pin};
use thiserror::Error;
use tracing::{event, Level};

macro_rules! define_signals {
  (
    pub enum $name:ident {
      $($case:ident = $val:ident),+
      $(,)?
    }
  ) => {
    #[repr(i32)]
    #[allow(clippy::enum_variant_names)]
    pub enum $name {
      $($case = ::signal_hook::consts::$val,)+
    }

    impl TryFrom<i32> for $name {
      type Error = ();

      fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
          $(::signal_hook::consts::$val => Ok(Self::$case),)+
          _ => Err(()),
        }
      }
    }

    impl fmt::Debug for $name {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
          $(Self::$case => f.write_str(stringify!($val)),)+
        }
      }
    }

    impl fmt::Display for $name {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
          $(Self::$case => f.write_str(stringify!($val)),)+
        }
      }
    }

    impl $name {
      const ALL: &'static [i32] = &[$(::signal_hook::consts::$val,)+];
    }
  };
}

define_signals! {
  pub enum Signal {
    SigTerm = SIGTERM,
    SigInt = SIGINT,
    SigQuit = SIGQUIT,
  }
}

#[derive(Clone)]
pub struct SharedSignal(Shared<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>);

impl SharedSignal {
  fn new(signal: Pin<Box<dyn Future<Output = ()> + Send + 'static>>) -> Self {
    Self(signal.shared())
  }
}

impl Future for SharedSignal {
  type Output = ();

  fn poll(
    mut self: Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Self::Output> {
    self.0.poll_unpin(cx)
  }
}

impl From<SharedSignal> for ShutdownSignalFuture {
  fn from(value: SharedSignal) -> Self {
    Box::pin(value)
  }
}

#[derive(Debug, Error)]
pub enum SignalWatchError {
  #[error(transparent)]
  Io(#[from] io::Error),
}

impl Signal {
  pub fn watch() -> Result<impl Stream<Item = Signal>, SignalWatchError> {
    let signals = Signals::new(Self::ALL)?;
    event!(target: "udev-device-manager", Level::DEBUG, "Started listening for termination signals");

    Ok(signals.filter_map(|s| ready(Signal::try_from(s).ok())))
  }

  pub fn shared() -> Result<SharedSignal, SignalWatchError> {
    let mut stream = Self::watch()?;

    Ok(SharedSignal::new(Box::pin(async move {
      let _ = stream.next().await;
    })))
  }
}
