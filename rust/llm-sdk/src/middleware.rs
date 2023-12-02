use reqwest::{header, Request, Response};
use reqwest_middleware::{Middleware, Next, Result};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use task_local_extensions::Extensions;

pub(crate) struct RetryMiddleware {
    inner: RetryTransientMiddleware<ExponentialBackoff>,
}

impl From<RetryTransientMiddleware<ExponentialBackoff>> for RetryMiddleware {
    fn from(inner: RetryTransientMiddleware<ExponentialBackoff>) -> Self {
        Self { inner }
    }
}

#[async_trait::async_trait]
impl Middleware for RetryMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        // check if req is cloneable without using try_clone
        // check request header - if content-type is multipart/form-data, then don't retry
        match req.headers().get(header::CONTENT_TYPE).map(|v| v.to_str()) {
            Some(Ok(content_type)) => {
                if content_type.contains("multipart/form-data")
                    || content_type.contains("application/octet-stream")
                {
                    // Don't need to retry.
                    next.run(req, extensions).await
                } else {
                    self.inner.handle(req, extensions, next).await
                }
            }
            _ => {
                // Retry.
                self.inner.handle(req, extensions, next).await
            }
        }
    }
}
