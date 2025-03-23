use crate::controller::Controller;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

pub struct FsWatcher;

impl FsWatcher {
    pub async fn watch(src_path: String, controller: Arc<Controller>) {
        let (tx, mut rx) = mpsc::channel(1);
        let debounce_tasks: Arc<Mutex<HashMap<String, JoinHandle<()>>>> =
            Arc::new(Mutex::new(HashMap::new()));

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
                let key = src_path.clone();
                let ctrl = controller.clone();
                let debounce_tasks = debounce_tasks.clone();

                let key_for_task = key.clone();

                if let Some(handle) = debounce_tasks.lock().unwrap().remove(&key) {
                    handle.abort();
                }

                let handle = tokio::spawn(async move {
                    sleep(Duration::from_millis(1000)).await;
                    println!("✅ Debounced sync for {}", key_for_task);
                    ctrl.on_fs_change(key_for_task).await;
                });

                debounce_tasks.lock().unwrap().insert(key, handle);
            }
        }
    }
}
