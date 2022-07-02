use std::{
    path::PathBuf,
    sync::{self, Arc},
    time::Duration,
};

use notify::{watcher, RecursiveMode, Watcher};
use tokio::sync::broadcast::Sender;

pub fn start_notify(shared_tx: Arc<Sender<()>>, current_dir: PathBuf) {
    std::thread::spawn(move || {
        let (tx, rx) = sync::mpsc::channel();
        //TODO: replace unwrap with proper error handling
        let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
        watcher
            .watch(&current_dir, RecursiveMode::Recursive)
            .unwrap();
        loop {
            if rx.recv().is_ok() {
                shared_tx.send(());
            }
        }
    });
}
