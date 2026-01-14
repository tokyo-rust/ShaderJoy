use async_trait::async_trait;

use crate::{
    generation::client::{LlmClient, LlmResult, ShaderGenerationRequest, ShaderGenerationResponse},
    shaders::DEFAULT_FRAGMENT_SHADER,
};

pub struct TestClient {}
impl TestClient {
    pub(crate) fn new() -> Self {
        Self {  }
    }
}

#[async_trait]
impl LlmClient for TestClient {
    async fn generate_shader(
        &self,
        _request: ShaderGenerationRequest,
    ) -> LlmResult<ShaderGenerationResponse> {
        Ok(ShaderGenerationResponse {
            wgsl_code: DEFAULT_FRAGMENT_SHADER.to_string(),
            tokens_used: Some(1337),
        })
    }

    async fn health_check(&self) -> LlmResult<()> {
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "Test Provider"
    }

    fn model_name(&self) -> &str {
        "static-shader-returning-provider"
    }
}
