use mill_plugin_api::WorkspaceSupport;
use regex::Regex;

#[derive(Default, Clone)]
pub struct SwiftWorkspaceSupport;

impl SwiftWorkspaceSupport {
    fn find_dependencies_array<'a>(&self, content: &'a str) -> Option<regex::Match<'a>> {
        let re = Regex::new(r"(?s)dependencies:\s*\[(.*?)\]").unwrap();
        re.find(content)
    }
}

impl WorkspaceSupport for SwiftWorkspaceSupport {
    fn is_workspace_manifest(&self, content: &str) -> bool {
        // A Package.swift is a "workspace" if it contains local package dependencies.
        // We can check for the presence of `.package(path: ...)`
        let re = Regex::new(r#"\.package\s*\(\s*path:"#).unwrap();
        re.is_match(content)
    }

    fn update_package_name(&self, content: &str, new_name: &str) -> String {
        let re = Regex::new(r#"(name:\s*")([^"]+)""#).unwrap();
        re.replace(content, format!(r#"$1{}"#, new_name)).to_string()
    }

    fn add_workspace_member(&self, content: &str, member_path: &str) -> String {
        let mut new_content = content.to_string();
        let new_package_line = format!("\n        .package(path: \"{}\"),", member_path);

        if let Some(deps_match) = self.find_dependencies_array(&new_content) {
            let end_pos = deps_match.end() - 1; // Before the closing ']'
            new_content.insert_str(end_pos, &new_package_line);
        } else {
            // If no dependencies array, we can't add the member.
            // Returning original content as we can't signal an error here.
        }
        new_content
    }

    fn remove_workspace_member(&self, content: &str, member_path: &str) -> String {
        let pattern = format!(r#"(?m)^\s*\.package\s*\(\s*path:\s*"{}"\s*\),?\s*[\r\n]?"#, regex::escape(member_path));
        let re = Regex::new(&pattern).unwrap();
        re.replace_all(content, "").to_string()
    }

    fn list_workspace_members(&self, content: &str) -> Vec<String> {
        let re = Regex::new(r#"\.package\s*\(\s*path:\s*"([^"]+)"\s*\)"#).unwrap();
        re.captures_iter(content)
            .map(|cap| cap[1].to_string())
            .collect()
    }
}