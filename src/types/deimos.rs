use color_eyre::eyre::Context;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Result, Watcher as _};
use tracing::{error, info};

use tokio::sync::mpsc::UnboundedSender;

use crate::{
    tui::Signal,
    types::{KastenHandle, ZettelId},
};

#[derive(Debug)]
pub struct Deimos {
    kh: KastenHandle,
    signal_tx: UnboundedSender<Signal>,
}

impl Deimos {
    pub const fn new(kh: KastenHandle, signal_tx: UnboundedSender<Signal>) -> Self {
        Self { kh, signal_tx }
    }

    /// Watches the `Filaments Directory` for file changes and updates
    /// the internal state of the kasten and the `Filaments` displayed in the
    /// gui.
    ///
    /// NOTE: This function must be spawned as a top level await
    pub async fn watch(&self) -> color_eyre::Result<()> {
        info!("deimos spawned!");

        let (tx, mut rx) = tokio::sync::mpsc::channel::<Result<Event>>(10);
        let mut watcher = RecommendedWatcher::new(
            move |res| tx.blocking_send(res).expect("failed to send event"),
            Config::default(),
        )?;

        watcher
            .watch(&self.kh.read().await.root, RecursiveMode::Recursive)
            .with_context(|| "failed to start the FS watcher")?;

        while let Some(res) = rx.recv().await {
            let Ok(event) = res.inspect_err(|e| error!("watcher error: {e:?}")) else {
                continue;
            };

            if let EventKind::Modify(notify::event::ModifyKind::Data(_)) = event.kind {
                for path in event.paths {
                    let Ok(zid) = ZettelId::try_from(path.clone())
                        .inspect_err(|e| error!("Failed to convert path into zettel id! : {e}"))
                    else {
                        continue;
                    };

                    let kt = &mut *self.kh.write().await;
                    kt.process_path(path).await?;

                    let links = kt.index.get_links(&zid).clone();

                    self.signal_tx.send(Signal::SetLinks { zid, links })?;
                }
            }
        }

        Ok(())
    }
}
