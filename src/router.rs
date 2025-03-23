use crate::config::{PathConfig, WatcherConfig};
use std::collections::HashMap;
use std::sync::Arc;

pub struct SyncRouter {
    by_src: HashMap<String, Vec<Arc<WatcherConfig>>>,
    by_selector: HashMap<String, Vec<Arc<WatcherConfig>>>,
}

impl SyncRouter {
    pub fn new(configs: Vec<WatcherConfig>) -> Self {
        let mut by_src: HashMap<String, Vec<Arc<WatcherConfig>>> = HashMap::new();
        let mut by_selector: HashMap<String, Vec<Arc<WatcherConfig>>> = HashMap::new();

        for config in &configs {
            let arc_config = Arc::new(config.clone());
            for path in &config.paths {
                by_src
                    .entry(path.src.clone())
                    .or_default()
                    .push(arc_config.clone());
            }
            for selector in &config.label_selectors {
                by_selector
                    .entry(selector.clone())
                    .or_default()
                    .push(arc_config.clone());
            }
        }

        Self {
            by_src,
            by_selector,
        }
    }

    pub fn match_configs(
        &self,
        src: &str,
        selector: &str,
    ) -> Vec<(Arc<WatcherConfig>, PathConfig)> {
        let src_matches = self.by_src.get(src);
        let sel_matches = self.by_selector.get(selector);

        let mut result = vec![];

        if let (Some(src_list), Some(sel_list)) = (src_matches, sel_matches) {
            for config in src_list {
                if sel_list.contains(config) {
                    for path in &config.paths {
                        if path.src == src {
                            result.push((config.clone(), path.clone()));
                        }
                    }
                }
            }
        }

        result
    }

    pub fn configs_by_src(&self, src: &str) -> Vec<Arc<WatcherConfig>> {
        self.by_src.get(src).cloned().unwrap_or_default()
    }

    pub fn configs_by_selector(&self, selector: &str) -> Vec<Arc<WatcherConfig>> {
        self.by_selector.get(selector).cloned().unwrap_or_default()
    }
}
