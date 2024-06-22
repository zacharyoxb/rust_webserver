use std::sync::Arc;
use std::time::SystemTime;

use hyper::body::Bytes;
use hyper::Request;

use crate::cache::Cache;
use crate::method_handlers::handler_utils;
use crate::resource_getters::dir_accessor;

enum WebContentState {
    Content {
        data: Bytes,
        content_type: String,
        last_modified: SystemTime,
        etag: String,
    },
    NotFound {
        data: Bytes,
    },
}

pub struct WebContent {
    state: WebContentState,
}

impl WebContent {
    fn new_content(
        data: Bytes,
        content_type: String,
        last_modified: SystemTime,
        etag: String,
    ) -> Self {
        Self {
            state: WebContentState::Content {
                data,
                content_type,
                last_modified,
                etag,
            },
        }
    }

    fn new_not_found(data: Bytes) -> Self {
        Self {
            state: WebContentState::NotFound { data },
        }
    }

    pub(crate) fn get_data(&self) -> &Bytes {
        match &self.state {
            WebContentState::Content { data, .. } | WebContentState::NotFound { data } => data,
        }
    }

    pub(crate) fn get_content_type(&self) -> Option<&String> {
        match &self.state {
            WebContentState::Content { content_type, .. } => Some(content_type),
            WebContentState::NotFound { .. } => None,
        }
    }

    pub(crate) fn get_last_modified(&self) -> Option<&SystemTime> {
        match &self.state {
            WebContentState::Content { last_modified, .. } => Some(last_modified),
            WebContentState::NotFound { .. } => None,
        }
    }

    pub(crate) fn get_etag(&self) -> Option<&String> {
        match &self.state {
            WebContentState::Content { etag, .. } => Some(etag),
            WebContentState::NotFound { .. } => None,
        }
    }

    pub(crate) fn is_not_found(&self) -> bool {
        matches!(self.state, WebContentState::NotFound { .. })
    }
}

pub(crate) async fn get_web_content(
    req: &Request<hyper::body::Incoming>,
    cache: Arc<Cache>,
) -> Option<WebContent> {
    // Holds cache results
    let cache_result = Cache::read_cache(Arc::clone(&cache), req.uri()).await;

    // Variable indicating whether cache can be checked
    let can_check_cache = handler_utils::header_evals::can_check_cache(req.headers());

    // Variable holding the etag of the cache value (if found) to check for staleness
    let cache_etag = cache_result
        .clone()
        .map(|(_, _, _, etag)| etag)
        .unwrap_or("".to_string());

    // Content temporarily wrapped in an option
    let mut wrapped_content: Option<WebContent> = None;

    // Check the cache for the requested resource
    if can_check_cache {
        if let Some((data, content_type, last_modified, etag)) = cache_result {
            wrapped_content = Some(WebContent::new_content(
                data,
                content_type,
                last_modified,
                etag,
            ));
        }
    }

    // If wasn't in cache or couldn't check cache, do a direct read
    if wrapped_content.is_none() {
        match dir_accessor::retrieve_resource(req.uri()).await? {
            (data, Some((content_type, last_modified))) => {
                let etag = Cache::generate_etag(&data);
                // If wasn't in cache, or etags don't match
                if cache_etag.is_empty() || cache_etag != etag {
                    Cache::write_cache(
                        Arc::clone(&cache),
                        req.uri(),
                        &data,
                        &content_type,
                        &last_modified,
                        &etag,
                    )
                    .await;
                }
                // Store read values in struct
                wrapped_content = Some(WebContent::new_content(
                    data,
                    content_type,
                    last_modified,
                    etag,
                ));
            }
            (data, None) => {
                // This represents a 404 page
                wrapped_content = Some(WebContent::new_not_found(data));
            }
        }
    }

    Some(wrapped_content.unwrap())
}
