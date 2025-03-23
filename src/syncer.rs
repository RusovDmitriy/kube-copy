use anyhow::{Context, Result};
use k8s_openapi::api::core::v1::Pod;
use kube::api::ListParams;
use kube::{Api, Client, Config};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Mutex;
use tokio::process::Command;

pub struct KubeSyncer {
    clients: Mutex<HashMap<String, Client>>, // context -> client
}

impl KubeSyncer {
    pub fn new() -> Self {
        Self {
            clients: Mutex::new(HashMap::new()),
        }
    }

    async fn get_or_create_client(&self, kube_context: &str) -> Result<Client> {
        if let Some(client) = self.clients.lock().unwrap().get(kube_context) {
            return Ok(client.clone());
        }

        let config = Config::from_kubeconfig(&kube::config::KubeConfigOptions {
            context: Some(kube_context.to_string()),
            ..Default::default()
        })
        .await
        .context("Failed to load kube config")?;

        let client = Client::try_from(config).context("Failed to create kube client")?;

        // 3. Повторно взять lock и вставить
        let mut clients = self.clients.lock().unwrap();
        clients.insert(kube_context.to_string(), client.clone());

        Ok(client)
    }

    pub async fn get_ready_pods(
        &self,
        kube_context: &str,
        namespace: &str,
        selector: &str,
    ) -> Result<Vec<String>> {
        let client = self.get_or_create_client(kube_context).await?;
        let pods: Api<Pod> = Api::namespaced(client, namespace);

        let lp = ListParams::default().labels(selector);
        let pod_list = pods.list(&lp).await.context("Failed to list pods")?;

        let ready_pods = pod_list
            .into_iter()
            .filter(Self::is_pod_ready)
            .filter_map(|pod| pod.metadata.name)
            .collect();

        Ok(ready_pods)
    }

    pub async fn sync(
        &self,
        kube_context: &str,
        namespace: &str,
        pod: &str,
        src: &str,
        dest: &str,
    ) {
        let target = format!("{}/{}:{}", namespace, pod, dest);
        println!("📤 Syncing {} -> {} (ctx: {})", src, target, kube_context);

        let output = Command::new("kubectl")
            .arg("cp")
            .arg(src)
            .arg(&target)
            .arg("--context")
            .arg(kube_context)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        match output {
            Ok(out) if out.status.success() => {
                println!("✅ Synced {} -> {} (ctx: {})", src, target, kube_context);
            }
            Ok(out) => {
                let err = String::from_utf8_lossy(&out.stderr);
                eprintln!("❌ Sync failed: {}", err);
            }
            Err(e) => {
                eprintln!("❌ Failed to run kubectl cp: {}", e);
            }
        }
    }

    fn is_pod_ready(pod: &Pod) -> bool {
        pod.status
            .as_ref()
            .and_then(|s| s.container_statuses.as_ref())
            .map(|statuses| statuses.iter().all(|cs| cs.ready))
            .unwrap_or(false)
    }
}
