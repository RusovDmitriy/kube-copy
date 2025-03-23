use crate::router::SyncRouter;
use crate::syncer::KubeSyncer;
use std::sync::Arc;

#[derive(Clone)]
pub struct Controller {
    router: Arc<SyncRouter>,
    syncer: Arc<KubeSyncer>,
}

impl Controller {
    pub fn new(router: Arc<SyncRouter>, syncer: Arc<KubeSyncer>) -> Self {
        Self { router, syncer }
    }

    pub async fn on_fs_change(&self, src: String) {
        let configs = self.router.configs_by_src(&src);
        for config in configs {
            for selector in &config.label_selectors {
                if let Ok(pods) = self
                    .syncer
                    .get_ready_pods(&config.kube_context, &config.namespace, selector)
                    .await
                {
                    for pod_name in pods {
                        let matched = self.router.match_configs(&src, selector);
                        for (conf, path) in matched {
                            self.syncer
                                .sync(
                                    &conf.kube_context,
                                    &conf.namespace,
                                    &pod_name,
                                    &path.src,
                                    &path.dest,
                                )
                                .await;
                        }
                    }
                }
            }
        }
    }

    pub async fn on_pod_ready(&self, ctx: String, ns: String, selector: String, pod_name: String) {
        let configs = self.router.configs_by_selector(&selector);
        for config in configs {
            for path in &config.paths {
                self.syncer
                    .sync(&ctx, &ns, &pod_name, &path.src, &path.dest)
                    .await;
            }
        }
    }
}
