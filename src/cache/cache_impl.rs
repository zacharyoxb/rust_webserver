// Standard library imports
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::env;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
// External crate imports
use hyper::Uri;
use tokio::sync::RwLock;
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Config};

pub struct Cache {
    content: RwLock<HashMap<Uri, (String, SystemTime)>>,
}

impl Cache {
    pub(crate) fn new() -> Arc<Self> {
          Arc::new(Self {
             content: RwLock::new(HashMap::new()),
        })
    }

    pub(crate) async fn read_cache(cache: Arc<Self>, uri: &Uri) -> Option<(String, SystemTime)> {
        let guard = cache.content.read().await;
        return match guard.get(uri) {
            Some((http_content, last_modified)) => {
                // start
                Some((http_content.clone(), last_modified.clone()))
            }
            None => None
        }
    }

    pub(crate) async fn write_cache(cache: Arc<Self>, uri: &Uri, http_content: &String, last_modified: &SystemTime) {
        let mut guard = cache.content.write().await;
        guard.insert(uri.clone(), (http_content.clone(), last_modified.clone()));
    }

    // Monitors html file for changes so cache doesn't go stale
    pub(crate) async fn start_watching(cache: &Arc<Self>) {
        let (tx, rx) = channel();

        let cache_clone = Arc::clone(&cache);

        // Spawn a tokio task to handle directory changes
        tokio::task::spawn(async move {
            let mut watcher: RecommendedWatcher = Watcher::new(tx.clone(), Config::default()).unwrap();
            let mut path_buf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            path_buf.push("html");
            let path = path_buf.as_path();
            watcher.watch(path, RecursiveMode::Recursive).unwrap();

            let mut debounce_timer = tokio::time::Instant::now();

            loop {
                match rx.recv() {
                    Ok(event) => match event {
                        Ok(event) => match event.kind {
                            notify::EventKind::Modify(_) |
                            notify::EventKind::Create(_) |
                            notify::EventKind::Remove(_) => {
                                if tokio::time::Instant::now().duration_since(debounce_timer) > Duration::from_secs(1) {
                                    debounce_timer = tokio::time::Instant::now();
                                    let mut content = cache_clone.content.write().await;
                                    content.clear();
                                }
                            },
                            _ => (),
                        },
                        Err(e) => eprintln!("watch error: {:?}", e),
                    },
                    Err(e) => eprintln!("recv error: {:?}", e),
                }
            }
        });
    }
}