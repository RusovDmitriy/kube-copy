use crate::controller::Controller;
use anyhow::Context;
use futures::StreamExt;
use k8s_openapi::api::core::v1::Pod;
use kube::{api::ListParams, Api, Client, Config};
use kube_runtime::watcher::{self, Config as WatcherConfig, Event};
use std::sync::Arc;

pub struct K8sWatcher;

impl K8sWatcher {
    pub async fn watch(
        ctx: String,
        ns: String,
        selector: String,
        controller: Arc<Controller>,
        sync_on_start: bool,
    ) {
        let config = match Self::load_config(&ctx).await {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("❌ Failed to load kubeconfig for context '{}': {}", ctx, e);
                return;
            }
        };

        let client = match Client::try_from(config.clone()) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("❌ Failed to create client: {}", e);
                return;
            }
        };

        let pods: Api<Pod> = Api::namespaced(client.clone(), &ns);

        if sync_on_start {
            Self::trigger_existing_ready_pods(&ctx, &ns, &selector, &controller, &pods).await;
        }

        let watcher_config = WatcherConfig::default().labels(&selector);
        let mut stream = watcher::watcher(pods, watcher_config).boxed();

        println!(
            "🔍 Watching pods (selector: {}, namespace: {}, ctx: {})",
            selector, ns, ctx
        );

        while let Some(event) = stream.next().await {
            match event {
                Ok(Event::Apply(pod)) => {
                    if let Some(name) = pod.metadata.name.clone() {
                        if Self::is_pod_ready(&pod) {
                            println!(
                                "🚀 Pod {} is ready (selector: {}, ctx: {})",
                                name, selector, ctx
                            );
                            controller
                                .on_pod_ready(ctx.clone(), ns.clone(), selector.clone(), name)
                                .await;
                        }
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    eprintln!("❌ Watch error: {}", e);
                }
            }
        }
    }

    async fn load_config(ctx: &str) -> anyhow::Result<Config> {
        Config::from_kubeconfig(&kube::config::KubeConfigOptions {
            context: Some(ctx.to_string()),
            ..Default::default()
        })
        .await
        .context("Failed to load kubeconfig")
    }

    fn is_pod_ready(pod: &Pod) -> bool {
        pod.status
            .as_ref()
            .and_then(|s| s.container_statuses.as_ref())
            .map(|statuses| statuses.iter().all(|cs| cs.ready))
            .unwrap_or(false)
    }

    async fn trigger_existing_ready_pods(
        ctx: &str,
        ns: &str,
        selector: &str,
        controller: &Arc<Controller>,
        pods_api: &Api<Pod>,
    ) {
        let lp = ListParams::default().labels(selector);
        match pods_api.list(&lp).await {
            Ok(existing_pods) => {
                for pod in existing_pods {
                    if let Some(name) = pod.metadata.name.clone() {
                        if Self::is_pod_ready(&pod) {
                            println!(
                                "🚀 Initial pod {} is ready (selector: {}, ctx: {})",
                                name, selector, ctx
                            );
                            controller
                                .on_pod_ready(
                                    ctx.to_string(),
                                    ns.to_string(),
                                    selector.to_string(),
                                    name,
                                )
                                .await;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to list initial pods: {}", e);
            }
        }
    }
}
