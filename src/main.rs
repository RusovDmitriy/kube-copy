mod config;
mod controller;
mod fs_watcher;
mod k8s_watcher;
mod router;
mod syncer;

use anyhow::Result;
use clap::{ArgAction, Parser};
use config::load_configs;
use controller::Controller;
use fs_watcher::FsWatcher;
use k8s_watcher::K8sWatcher;
use router::SyncRouter;
use std::collections::HashSet;
use std::sync::Arc;
use syncer::KubeSyncer;
use tokio::time::sleep;

#[derive(Parser, Debug)]
#[command(name = "kube-copy", version, about = "🔄 Kubernetes File Watcher & Syncer", long_about = None)]
struct Cli {
    /// Path to the watcher configuration file
    #[arg(short, long, default_value = "watcher.json")]
    config: String,

    /// Sync to all pods at startup
    #[arg(long, default_value_t = false, action = ArgAction::SetTrue)]
    sync_on_start: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let configs = load_configs(&cli.config)?;

    let router = Arc::new(SyncRouter::new(configs.clone()));
    let syncer = Arc::new(KubeSyncer::new());
    let controller = Arc::new(Controller::new(router.clone(), syncer.clone()));

    let mut unique_paths = HashSet::new();
    for config in &configs {
        for path in &config.paths {
            unique_paths.insert(path.src.clone());
        }
    }

    for src_path in unique_paths {
        let ctrl = controller.clone();
        tokio::spawn(async move {
            FsWatcher::watch(src_path, ctrl).await;
        });
    }

    let mut unique_selectors = HashSet::new();
    for config in &configs {
        for selector in &config.label_selectors {
            unique_selectors.insert((
                config.kube_context.clone(),
                config.namespace.clone(),
                selector.clone(),
            ));
        }
    }

    for (ctx, ns, selector) in unique_selectors {
        let ctrl = controller.clone();
        tokio::spawn(async move {
            K8sWatcher::watch(ctx, ns, selector, ctrl, cli.sync_on_start.clone()).await;
        });
    }

    println!("🚀 Watcher started with config: {}", cli.config);

    loop {
        sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}
