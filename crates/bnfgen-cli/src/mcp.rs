mod generate;

use crate::mcp::generate::GenerationRequest;
use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ServerCapabilities, ServerInfo};
use rmcp::{tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler};

#[derive(Clone)]
pub struct BnfgenMCP {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl BnfgenMCP {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        description = "Generates random strings based on a provided BNF grammar and a starting symbol."
    )]
    async fn generate(
        &self,
        _params: Parameters<GenerationRequest>,
    ) -> Result<CallToolResult, McpError> {
        todo!()
    }
}

#[tool_handler]
impl ServerHandler for BnfgenMCP {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("This server provides tools for generating random strings based on BNF grammars. Use the 'generate' tool to create strings from a specified grammar and starting symbol.".to_string()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
