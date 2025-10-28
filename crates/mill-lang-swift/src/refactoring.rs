use crate::SwiftPlugin;
use async_trait::async_trait;
use mill_foundation::protocol::{EditPlan, EditPlanMetadata};
use mill_plugin_api::{PluginError, RefactoringProvider};

#[async_trait]
impl RefactoringProvider for SwiftPlugin {
    async fn plan_extract_function(
        &self,
        _source: &str,
        _start_line: u32,
        _end_line: u32,
        _function_name: &str,
        _file_path: &str,
    ) -> Result<EditPlan, PluginError> {
        Ok(EditPlan {
            source_file: _file_path.to_string(),
            edits: vec![],
            dependency_updates: vec![],
            validations: vec![],
            metadata: EditPlanMetadata {
                intent_name: "extract_function".to_string(),
                intent_arguments: serde_json::Value::Null,
                created_at: chrono::Utc::now(),
                complexity: 0,
                impact_areas: vec![],
                consolidation: None,
            },
        })
    }

    async fn plan_extract_variable(
        &self,
        _source: &str,
        _start_line: u32,
        _start_col: u32,
        _end_line: u32,
        _end_col: u32,
        _variable_name: Option<String>,
        _file_path: &str,
    ) -> Result<EditPlan, PluginError> {
        Ok(EditPlan {
            source_file: _file_path.to_string(),
            edits: vec![],
            dependency_updates: vec![],
            validations: vec![],
            metadata: EditPlanMetadata {
                intent_name: "extract_variable".to_string(),
                intent_arguments: serde_json::Value::Null,
                created_at: chrono::Utc::now(),
                complexity: 0,
                impact_areas: vec![],
                consolidation: None,
            },
        })
    }

    async fn plan_inline_variable(
        &self,
        _source: &str,
        _variable_line: u32,
        _variable_col: u32,
        _file_path: &str,
    ) -> Result<EditPlan, PluginError> {
        Ok(EditPlan {
            source_file: _file_path.to_string(),
            edits: vec![],
            dependency_updates: vec![],
            validations: vec![],
            metadata: EditPlanMetadata {
                intent_name: "inline_variable".to_string(),
                intent_arguments: serde_json::Value::Null,
                created_at: chrono::Utc::now(),
                complexity: 0,
                impact_areas: vec![],
                consolidation: None,
            },
        })
    }
}