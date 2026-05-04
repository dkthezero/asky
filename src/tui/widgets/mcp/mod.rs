use crate::domain::mcp::{McpRegistry, McpServer};

/// MCP registry state for TUI rendering.
#[derive(Debug, Clone)]
pub struct McpState {
    pub registry: McpRegistry,
}

impl Default for McpState {
    fn default() -> Self {
        let path = crate::domain::paths::mcp_path();
        let registry = McpRegistry::load(&path).unwrap_or_default();
        Self { registry }
    }
}

impl McpState {
    pub fn refresh(&mut self) {
        let path = crate::domain::paths::mcp_path();
        if let Ok(registry) = McpRegistry::load(&path) {
            self.registry = registry;
        }
    }

    pub fn servers_list(&self) -> Vec<(&String, &McpServer)> {
        let mut items: Vec<_> = self.registry.servers.iter().collect();
        items.sort_by(|a, b| a.0.cmp(b.0));
        items
    }
}

pub mod render;
