// use color_eyre::eyre::Context;
// use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Result, Watcher as _};
// use tracing::{error, info};

use crate::types::KastenHandle;

#[derive(Debug)]
#[expect(dead_code)]
pub struct Deimos {
    kh: KastenHandle,
    // fh: FilamentsHandle,
}

// impl Deimos {
//     pub const fn new(kh: KastenHandle, fh: FilamentsHandle) -> Self {
//         Self { kh, fh }
//     }

//     /// Watches the `Filaments Directory` for file changes and updates
//     /// the internal state of the kasten and the `Filaments` displayed in the
//     /// gui.
//     ///
//     /// NOTE: This function must be spawned as a top level await
//     pub async fn watch(&self) -> color_eyre::Result<()> {
//         info!("deimos spawned!");

//         let (tx, mut rx) = tokio::sync::mpsc::channel::<Result<Event>>(10);
//         let mut watcher = RecommendedWatcher::new(
//             move |res| tx.blocking_send(res).expect("failed to send event"),
//             Config::default(),
//         )?;

//         watcher
//             .watch(&self.kh.read().await.root, RecursiveMode::Recursive)
//             .with_context(|| "failed to start the FS watcher")?;

//         while let Some(res) = rx.recv().await {
//             let kt = &mut *self.kh.write().await;
//             let fh = &mut *self.fh.lock().expect("Lock must not be poisoned");

//             match res {
//                 Ok(event) => info!("event: {event:?}"),
//                 Err(e) => error!("watch error: {e:?}"),
//             }

//             *fh = (&kt.index).into();
//         }

//         Ok(())
//     }
// }
