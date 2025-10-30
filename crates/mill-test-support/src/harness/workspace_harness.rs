//! Cross-language workspace test harness
//!
//! Provides language-equivalent fixtures for workspace operations across multiple languages.
//! Follows the same pattern as import_harness.rs and refactoring_harness.rs.

pub use super::refactoring_harness::Language;

impl Language {
    pub fn all_with_workspace_support() -> Vec<Language> {
        // Currently testing: TypeScript, Rust, Python (3/9 languages)
        //
        // These languages have WorkspaceSupport trait implementations,
        // but plugin registration blockers prevent testing the others:
        //
        // BLOCKERS:
        // - Java, Go: Plugin not found (registration issue in mill-plugin-bundle)
        // - CSharp, Swift, C: No workspace support implementation
        // - Cpp: Has stub implementation that doesn't work (always returns false/empty)
        //
        // TODO: Resolve plugin registration to enable Java
        // TODO: Add workspace support for Go (go.work files)
        vec![
            Language::TypeScript,
            Language::Rust,
            Language::Python,
        ]
    }
}

/// Workspace operations that can be tested
#[derive(Debug, Clone)]
pub enum WorkspaceOperation {
    IsWorkspaceManifest,
    ListWorkspaceMembers,
    AddWorkspaceMember { member: String },
    RemoveWorkspaceMember { member: String },
    UpdatePackageName { new_name: String },
}

/// Expected behavior for a workspace test
#[derive(Debug, Clone)]
pub enum WorkspaceExpectedBehavior {
    IsWorkspace(bool),
    MembersList(Vec<String>),
    Added,  // Verify member was added by checking list contains it
    Removed,  // Verify member was removed by checking list doesn't contain it
    NameUpdated(String),  // Verify name matches expected
    NotSupported,
}

/// Language-specific manifest fixture for workspace tests
#[derive(Debug, Clone)]
pub struct WorkspaceFixture {
    pub language: Language,
    pub manifest_content: &'static str,
    pub operation: WorkspaceOperation,
    pub expected: WorkspaceExpectedBehavior,
}

/// Complete test case for cross-language workspace testing
pub struct WorkspaceTestCase {
    pub scenario_name: &'static str,
    pub fixtures: Vec<WorkspaceFixture>,
}

impl WorkspaceTestCase {
    pub fn new(scenario_name: &'static str) -> Self {
        Self {
            scenario_name,
            fixtures: Vec::new(),
        }
    }

    pub fn with_all_languages<F>(mut self, generator: F) -> Self
    where
        F: Fn(Language) -> WorkspaceFixture,
    {
        for lang in Language::all_with_workspace_support() {
            let fixture = generator(lang);
            self.fixtures.push(fixture);
        }
        self
    }
}

/// Predefined workspace test scenarios
pub struct WorkspaceScenarios;

impl WorkspaceScenarios {
    /// Check if manifest is a workspace (positive case)
    pub fn is_workspace_manifest_positive() -> WorkspaceTestCase {
        WorkspaceTestCase::new("is_workspace_manifest_positive").with_all_languages(|lang| {
            let manifest = match lang {
                Language::TypeScript => r#"{"name":"root","workspaces":["packages/*"]}"#,
                Language::Rust => "[workspace]\nmembers = [\"crates/*\"]\n",
                Language::Python => "[tool.pdm.workspace]\nmembers = [\"packages/*\"]\n",
                Language::Java => "<?xml version=\"1.0\"?>\n<project>\n<modules>\n<module>module-a</module>\n</modules>\n</project>",
                Language::Cpp => "cmake_minimum_required(VERSION 3.10)\nproject(MyWorkspace)\nadd_subdirectory(module-a)\n",
                _ => unreachable!(),
            };

            WorkspaceFixture {
                language: lang,
                manifest_content: manifest,
                operation: WorkspaceOperation::IsWorkspaceManifest,
                expected: WorkspaceExpectedBehavior::IsWorkspace(true),
            }
        })
    }

    /// Check if manifest is NOT a workspace (negative case)
    pub fn is_workspace_manifest_negative() -> WorkspaceTestCase {
        WorkspaceTestCase::new("is_workspace_manifest_negative").with_all_languages(|lang| {
            let manifest = match lang {
                Language::TypeScript => r#"{"name":"single-package","version":"1.0.0"}"#,
                Language::Rust => "[package]\nname = \"single-crate\"\nversion = \"1.0.0\"\n",
                Language::Python => "[project]\nname = \"single-package\"\n",
                Language::Java => "<?xml version=\"1.0\"?>\n<project>\n<artifactId>single</artifactId>\n</project>",
                Language::Cpp => "cmake_minimum_required(VERSION 3.10)\nproject(SingleProject)\n",
                _ => unreachable!(),
            };

            WorkspaceFixture {
                language: lang,
                manifest_content: manifest,
                operation: WorkspaceOperation::IsWorkspaceManifest,
                expected: WorkspaceExpectedBehavior::IsWorkspace(false),
            }
        })
    }

    /// List existing workspace members
    pub fn list_workspace_members() -> WorkspaceTestCase {
        WorkspaceTestCase::new("list_workspace_members").with_all_languages(|lang| {
            let (manifest, expected) = match lang {
                Language::TypeScript => (
                    r#"{"name":"root","workspaces":["packages/a","packages/b"]}"#,
                    vec!["packages/a".to_string(), "packages/b".to_string()],
                ),
                Language::Rust => (
                    "[workspace]\nmembers = [\"crates/a\", \"crates/b\"]\n",
                    vec!["crates/a".to_string(), "crates/b".to_string()],
                ),
                Language::Python => (
                    "[tool.pdm.workspace]\nmembers = [\"packages/a\", \"packages/b\"]\n",
                    vec!["packages/a".to_string(), "packages/b".to_string()],
                ),
                Language::Java => (
                    "<?xml version=\"1.0\"?>\n<project>\n<modules>\n<module>module-a</module>\n<module>module-b</module>\n</modules>\n</project>",
                    vec!["module-a".to_string(), "module-b".to_string()],
                ),
                Language::Cpp => (
                    "cmake_minimum_required(VERSION 3.10)\nproject(MyWorkspace)\nadd_subdirectory(module-a)\nadd_subdirectory(module-b)\n",
                    vec!["module-a".to_string(), "module-b".to_string()],
                ),
                _ => unreachable!(),
            };

            WorkspaceFixture {
                language: lang,
                manifest_content: manifest,
                operation: WorkspaceOperation::ListWorkspaceMembers,
                expected: WorkspaceExpectedBehavior::MembersList(expected),
            }
        })
    }

    /// Add a new workspace member
    pub fn add_workspace_member() -> WorkspaceTestCase {
        WorkspaceTestCase::new("add_workspace_member").with_all_languages(|lang| {
            let (manifest, member) = match lang {
                Language::TypeScript => (r#"{"name":"root","workspaces":["packages/a"]}"#, "packages/b"),
                Language::Rust => ("[workspace]\nmembers = [\"crates/a\"]\n", "crates/b"),
                Language::Python => ("[tool.pdm.workspace]\nmembers = [\"packages/a\"]\n", "packages/b"),
                Language::Java => ("<?xml version=\"1.0\"?>\n<project>\n<modules>\n<module>module-a</module>\n</modules>\n</project>", "module-b"),
                Language::Cpp => ("cmake_minimum_required(VERSION 3.10)\nproject(MyWorkspace)\nadd_subdirectory(module-a)\n", "module-b"),
                _ => unreachable!(),
            };

            WorkspaceFixture {
                language: lang,
                manifest_content: manifest,
                operation: WorkspaceOperation::AddWorkspaceMember {
                    member: member.to_string(),
                },
                expected: WorkspaceExpectedBehavior::Added,
            }
        })
    }

    /// Add duplicate member (should be idempotent)
    pub fn add_workspace_member_duplicate() -> WorkspaceTestCase {
        WorkspaceTestCase::new("add_workspace_member_duplicate").with_all_languages(|lang| {
            let (manifest, member) = match lang {
                Language::TypeScript => (r#"{"name":"root","workspaces":["packages/a"]}"#, "packages/a"),
                Language::Rust => ("[workspace]\nmembers = [\"crates/a\"]\n", "crates/a"),
                Language::Python => ("[tool.pdm.workspace]\nmembers = [\"packages/a\"]\n", "packages/a"),
                Language::Java => ("<?xml version=\"1.0\"?>\n<project>\n<modules>\n<module>module-a</module>\n</modules>\n</project>", "module-a"),
                Language::Cpp => ("cmake_minimum_required(VERSION 3.10)\nproject(MyWorkspace)\nadd_subdirectory(module-a)\n", "module-a"),
                _ => unreachable!(),
            };

            WorkspaceFixture {
                language: lang,
                manifest_content: manifest,
                operation: WorkspaceOperation::AddWorkspaceMember {
                    member: member.to_string(),
                },
                expected: WorkspaceExpectedBehavior::Added,  // Should still work (idempotent)
            }
        })
    }

    /// Remove an existing workspace member
    pub fn remove_workspace_member() -> WorkspaceTestCase {
        WorkspaceTestCase::new("remove_workspace_member").with_all_languages(|lang| {
            let (manifest, member) = match lang {
                Language::TypeScript => (r#"{"name":"root","workspaces":["packages/a","packages/b"]}"#, "packages/a"),
                Language::Rust => ("[workspace]\nmembers = [\"crates/a\", \"crates/b\"]\n", "crates/a"),
                Language::Python => ("[tool.pdm.workspace]\nmembers = [\"packages/a\", \"packages/b\"]\n", "packages/a"),
                Language::Java => ("<?xml version=\"1.0\"?>\n<project>\n<modules>\n<module>module-a</module>\n<module>module-b</module>\n</modules>\n</project>", "module-a"),
                Language::Cpp => ("cmake_minimum_required(VERSION 3.10)\nproject(MyWorkspace)\nadd_subdirectory(module-a)\nadd_subdirectory(module-b)\n", "module-a"),
                _ => unreachable!(),
            };

            WorkspaceFixture {
                language: lang,
                manifest_content: manifest,
                operation: WorkspaceOperation::RemoveWorkspaceMember {
                    member: member.to_string(),
                },
                expected: WorkspaceExpectedBehavior::Removed,
            }
        })
    }

    /// Update package name
    pub fn update_package_name() -> WorkspaceTestCase {
        WorkspaceTestCase::new("update_package_name").with_all_languages(|lang| {
            let (manifest, new_name) = match lang {
                Language::TypeScript => (r#"{"name":"old-name","version":"1.0.0"}"#, "new-name"),
                Language::Rust => ("[package]\nname = \"old-name\"\nversion = \"1.0.0\"\n", "new-name"),
                Language::Python => ("[project]\nname = \"old-name\"\n", "new-name"),
                Language::Java => ("<?xml version=\"1.0\"?>\n<project>\n<artifactId>old-name</artifactId>\n</project>", "new-name"),
                Language::Cpp => ("cmake_minimum_required(VERSION 3.10)\nproject(old-name)\n", "new-name"),
                _ => unreachable!(),
            };

            WorkspaceFixture {
                language: lang,
                manifest_content: manifest,
                operation: WorkspaceOperation::UpdatePackageName {
                    new_name: new_name.to_string(),
                },
                expected: WorkspaceExpectedBehavior::NameUpdated(new_name.to_string()),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_scenarios_defined() {
        let scenarios = vec![
            WorkspaceScenarios::is_workspace_manifest_positive(),
            WorkspaceScenarios::is_workspace_manifest_negative(),
            WorkspaceScenarios::list_workspace_members(),
            WorkspaceScenarios::add_workspace_member(),
            WorkspaceScenarios::add_workspace_member_duplicate(),
            WorkspaceScenarios::remove_workspace_member(),
            WorkspaceScenarios::update_package_name(),
        ];

        assert_eq!(scenarios.len(), 7, "Should have 7 core workspace scenarios");
    }

    #[test]
    fn test_scenario_has_all_languages() {
        let scenario = WorkspaceScenarios::list_workspace_members();
        let languages = Language::all_with_workspace_support();

        assert_eq!(
            scenario.fixtures.len(),
            languages.len(),
            "Each scenario should have fixtures for all workspace-supporting languages"
        );
    }
}
