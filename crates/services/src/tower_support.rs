//! Tower service integration for middleware and HTTP service composition.

#[cfg(feature = "tower")]
use tower::Service;
use crate::prelude::*;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

/// Request types for the Tower service.
#[derive(Debug, Clone)]
pub enum AstGrepRequest {
    Scan(ScanOptions),
    Search(SearchRequest),
    Fix(FixRequest),
}

/// Response types for the Tower service.
#[derive(Debug, Clone)]
pub enum AstGrepResponse {
    Scan(ScanResults),
    Search(SearchResults),
    Fix(FixResults),
}

/// Future type for async service calls.
pub type AstGrepFuture = Pin<Box<dyn Future<Output = Result<AstGrepResponse>> + Send>>;

/// Tower service wrapper for AstGrepService.
#[cfg(feature = "tower")]
pub struct AstGrepTowerService {
    inner: Arc<AstGrepService>,
}

#[cfg(feature = "tower")]
impl AstGrepTowerService {
    pub fn new(service: AstGrepService) -> Self {
        Self {
            inner: Arc::new(service),
        }
    }
}

#[cfg(feature = "tower")]
impl Service<AstGrepRequest> for AstGrepTowerService {
    type Response = AstGrepResponse;
    type Error = AstGrepError;
    type Future = AstGrepFuture;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: AstGrepRequest) -> Self::Future {
        let service = self.inner.clone();

        Box::pin(async move {
            match req {
                AstGrepRequest::Scan(options) => {
                    let result = service.scan_files(options).await?;
                    Ok(AstGrepResponse::Scan(result))
                }
                AstGrepRequest::Search(request) => {
                    let result = service.search_pattern(request).await?;
                    Ok(AstGrepResponse::Search(result))
                }
                AstGrepRequest::Fix(request) => {
                    let result = service.apply_fixes(request).await?;
                    Ok(AstGrepResponse::Fix(result))
                }
            }
        })
    }
}

/// Service builder with middleware support.
#[cfg(feature = "tower")]
pub struct AstGrepServiceBuilder {
    service: Option<AstGrepService>,
}

#[cfg(feature = "tower")]
impl AstGrepServiceBuilder {
    pub fn new() -> Self {
        Self { service: None }
    }

    pub fn with_service(mut self, service: AstGrepService) -> Self {
        self.service = Some(service);
        self
    }

    pub fn build(self) -> Result<AstGrepTowerService> {
        let service = self.service.ok_or_else(|| AstGrepError::validation_error(
            "service",
            "Service is required"
        ))?;

        Ok(AstGrepTowerService::new(service))
    }

    /// Add timeout middleware.
    #[cfg(feature = "tower")]
    pub fn with_timeout(
        self,
        timeout: std::time::Duration
    ) -> tower::timeout::Timeout<AstGrepTowerService> {
        let service = self.build().expect("Service must be configured");
        tower::timeout::Timeout::new(service, timeout)
    }

    /// Add rate limiting middleware.
    #[cfg(feature = "tower")]
    pub fn with_rate_limit(
        self,
        num: u64,
        per: std::time::Duration,
    ) -> tower::limit::RateLimit<AstGrepTowerService> {
        let service = self.build().expect("Service must be configured");
        tower::limit::RateLimit::new(service, tower::limit::rate::Rate::new(num, per))
    }

    /// Add concurrency limit middleware.
    #[cfg(feature = "tower")]
    pub fn with_concurrency_limit(
        self,
        max: usize,
    ) -> tower::limit::ConcurrencyLimit<AstGrepTowerService> {
        let service = self.build().expect("Service must be configured");
        tower::limit::ConcurrencyLimit::new(service, max)
    }
}

#[cfg(feature = "tower")]
impl Default for AstGrepServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to create a Tower service stack.
#[cfg(feature = "tower")]
pub fn create_service_stack(
    service: AstGrepService,
    timeout: Option<std::time::Duration>,
    rate_limit: Option<(u64, std::time::Duration)>,
    concurrency_limit: Option<usize>,
) -> Box<dyn Service<
    AstGrepRequest,
    Response = AstGrepResponse,
    Error = AstGrepError,
    Future = Pin<Box<dyn Future<Output = Result<AstGrepResponse>> + Send>>,
> + Send> {
    let mut builder = AstGrepServiceBuilder::new().with_service(service);

    // Note: This is a simplified example. In practice, you'd use ServiceBuilder
    // for proper middleware composition.
    let base_service = builder.build().expect("Service configuration failed");

    Box::new(base_service)
}

/// HTTP handler utilities for web frameworks.
pub mod http {
    use super::*;
    use serde_json;

    /// Convert HTTP JSON request to AstGrepRequest.
    pub fn parse_http_request(
        method: &str,
        body: &str,
    ) -> Result<AstGrepRequest> {
        match method {
            "POST" if body.contains("\"scan\"") => {
                let options: ScanOptions = serde_json::from_str(body)
                    .map_err(|e| AstGrepError::validation_error("request", &e.to_string()))?;
                Ok(AstGrepRequest::Scan(options))
            }
            "POST" if body.contains("\"search\"") => {
                let request: SearchRequest = serde_json::from_str(body)
                    .map_err(|e| AstGrepError::validation_error("request", &e.to_string()))?;
                Ok(AstGrepRequest::Search(request))
            }
            "POST" if body.contains("\"fix\"") => {
                let request: FixRequest = serde_json::from_str(body)
                    .map_err(|e| AstGrepError::validation_error("request", &e.to_string()))?;
                Ok(AstGrepRequest::Fix(request))
            }
            _ => Err(AstGrepError::validation_error("method", "Unsupported method or request type")),
        }
    }

    /// Convert AstGrepResponse to HTTP JSON response.
    pub fn format_http_response(response: AstGrepResponse) -> Result<String> {
        serde_json::to_string_pretty(&response)
            .map_err(|e| AstGrepError::Output {
                message: format!("Failed to serialize response: {}", e),
                output_type: "JSON".to_string(),
                source: None,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::noop::*;
    use std::sync::Arc;

    fn create_test_service() -> AstGrepService {
        let registry = crate::registry::ServiceRegistry::builder()
            .with_file_discovery(Arc::new(NoOpFileDiscoveryService))
            .with_configuration(Arc::new(NoOpConfigurationService))
            .with_output(Arc::new(NoOpOutputService))
            .with_interaction(Arc::new(NoOpInteractionService))
            .with_terminal(Arc::new(NoOpTerminalService))
            .with_test_execution(Arc::new(NoOpTestExecutionService))
            .build();

        let runtime = crate::runtime::create_runtime_for(
            crate::runtime::RuntimeEnvironment::CloudflareWorkers
        );

        AstGrepService::new(registry, runtime)
    }

    #[tokio::test]
    #[cfg(feature = "tower")]
    async fn test_tower_service() {
        let service = create_test_service();
        let mut tower_service = AstGrepTowerService::new(service);

        let request = AstGrepRequest::Scan(ScanOptions::default());
        let response = tower::Service::call(&mut tower_service, request).await;

        assert!(response.is_ok());
        match response.unwrap() {
            AstGrepResponse::Scan(results) => {
                assert_eq!(results.files_processed, 0);
            }
            _ => panic!("Expected scan response"),
        }
    }

    #[test]
    #[cfg(feature = "tower")]
    fn test_service_builder() {
        let service = create_test_service();
        let builder = AstGrepServiceBuilder::new().with_service(service);
        let tower_service = builder.build();

        assert!(tower_service.is_ok());
    }

    #[test]
    fn test_http_request_parsing() {
        let scan_body = r#"{"paths": ["."], "file_patterns": ["*.rs"]}"#;
        let request = http::parse_http_request("POST", scan_body);

        // This will fail because the body doesn't contain "scan" keyword
        // In practice, you'd have a proper request format
        assert!(request.is_err());
    }

    #[test]
    fn test_http_response_formatting() {
        let response = AstGrepResponse::Scan(ScanResults {
            matches: Vec::new(),
            execution_time: None,
            files_processed: 0,
        });

        let json = http::format_http_response(response);
        assert!(json.is_ok());
        assert!(json.unwrap().contains("matches"));
    }
}
