use crate::controller::Controller;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct FsWatcher;

impl FsWatcher {
    pub async fn watch(src_path: String, controller: Arc<Controller>) {
        let (tx, mut rx) = mpsc::channel(1);
        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.blocking_send(res);
            },
            notify::Config::default(),
        )
        .expect("Failed to initialize watcher");

        watcher
            .watch(Path::new(&src_path), RecursiveMode::Recursive)
            .expect("Failed to watch directory");

        println!("👀 Watching directory: {}", src_path);

        while let Some(Ok(event)) = rx.recv().await {
            if matches!(
                event.kind,
                EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
            ) {
                println!("📦 File system change detected at: {:?}", event.paths);
                controller.on_fs_change(src_path.clone()).await;
            }
        }
    }
}
