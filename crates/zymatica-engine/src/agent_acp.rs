use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpSyncState {
    pub workspace_uri: String,
    pub open_files: Vec<String>,
    pub active_file: Option<String>,
    pub cursor_line: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpAgentMessage {
    pub message_id: String,
    pub conversation_id: String,
    pub payload: Value,
}

pub struct AcpServerEngine {
    state: AcpSyncState,
}

impl AcpServerEngine {
    pub fn new(workspace_uri: &str) -> Self {
        Self {
            state: AcpSyncState {
                workspace_uri: workspace_uri.to_string(),
                open_files: vec![],
                active_file: None,
                cursor_line: None,
            },
        }
    }

    pub fn update_active_file(&mut self, path: String, line: Option<u32>) {
        if !self.state.open_files.contains(&path) {
            self.state.open_files.push(path.clone());
        }
        self.state.active_file = Some(path);
        self.state.cursor_line = line;
    }

    pub fn get_state(&self) -> &AcpSyncState {
        &self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acp_state_update() {
        let mut server = AcpServerEngine::new("file:///workspace");
        server.update_active_file("/workspace/src/lib.rs".to_string(), Some(42));
        let st = server.get_state();
        assert_eq!(st.active_file.as_deref(), Some("/workspace/src/lib.rs"));
        assert_eq!(st.cursor_line, Some(42));
    }
}
