use crate::domain::telemetry::AnalyticsConfig;
use chrono::{DateTime, Utc};
use std::path::PathBuf;

/// Parsed skill invocation from a single log line.
#[derive(Debug, Clone, PartialEq)]
pub struct SkillInvocation {
    pub skill_name: String,
    pub provider_id: String,
    pub timestamp: DateTime<Utc>,
}

/// Trait for provider-specific log parsers.
pub trait LogParser: Send + Sync {
    fn provider_id(&self) -> &str;
    fn log_directories(&self) -> Vec<PathBuf>;
    fn parse_line(&self, line: &str) -> Option<SkillInvocation>;
}

// ---------------------------------------------------------------------------
// Claude Code
// ---------------------------------------------------------------------------

pub struct ClaudeCodeLogParser;

impl LogParser for ClaudeCodeLogParser {
    fn provider_id(&self) -> &str {
        "claude-code"
    }

    fn log_directories(&self) -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        if let Some(home) = dirs_next::home_dir() {
            dirs.push(home.join("Library/Logs/Claude")); // macOS
            dirs.push(home.join(".local/share/Claude/logs")); // Linux
        }
        dirs
    }

    fn parse_line(&self, line: &str) -> Option<SkillInvocation> {
        let name = extract_quoted_after(line, "executed tool `")
            .or_else(|| extract_quoted_after(line, "skill `"))
            .or_else(|| extract_after_prefix(line, "running skill: "))?;
        Some(SkillInvocation {
            skill_name: name.to_string(),
            provider_id: self.provider_id().to_string(),
            timestamp: Utc::now(),
        })
    }
}

// ---------------------------------------------------------------------------
// GitHub Copilot
// ---------------------------------------------------------------------------

pub struct CopilotLogParser;

impl LogParser for CopilotLogParser {
    fn provider_id(&self) -> &str {
        "copilot"
    }

    fn log_directories(&self) -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        if let Some(home) = dirs_next::home_dir() {
            dirs.push(home.join("Library/Logs/GitHub Copilot")); // macOS
        }
        dirs
    }

    fn parse_line(&self, line: &str) -> Option<SkillInvocation> {
        let name = extract_quoted_after(line, "invoked tool `")
            .or_else(|| extract_after_prefix(line, "tool call: "))?;
        Some(SkillInvocation {
            skill_name: name.to_string(),
            provider_id: self.provider_id().to_string(),
            timestamp: Utc::now(),
        })
    }
}

// ---------------------------------------------------------------------------
// OpenCode
// ---------------------------------------------------------------------------

pub struct OpenCodeLogParser;

impl LogParser for OpenCodeLogParser {
    fn provider_id(&self) -> &str {
        "opencode"
    }

    fn log_directories(&self) -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        if let Some(home) = dirs_next::home_dir() {
            dirs.push(home.join(".config/opencode/logs"));
        }
        dirs
    }

    fn parse_line(&self, line: &str) -> Option<SkillInvocation> {
        let name = extract_quoted_after(line, "executing skill `")
            .or_else(|| extract_after_prefix(line, "skill execution: "))?;
        Some(SkillInvocation {
            skill_name: name.to_string(),
            provider_id: self.provider_id().to_string(),
            timestamp: Utc::now(),
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn extract_quoted_after<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    let start = line.find(prefix)? + prefix.len();
    let end = line[start..].find('\'')?;
    Some(&line[start..start + end])
}

fn extract_after_prefix<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    let start = line.find(prefix)? + prefix.len();
    let rest = &line[start..];
    let end = rest
        .find(|c: char| c.is_whitespace() || c == '\'' || c == '`')
        .unwrap_or(rest.len());
    let name = rest[..end].trim();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

// ---------------------------------------------------------------------------
// Built-in parser registry
// ---------------------------------------------------------------------------

pub fn default_parsers() -> Vec<Box<dyn LogParser>> {
    vec![
        Box::new(ClaudeCodeLogParser),
        Box::new(CopilotLogParser),
        Box::new(OpenCodeLogParser),
    ]
}

// ---------------------------------------------------------------------------
// Scan a single directory tree for new invocations
// ---------------------------------------------------------------------------

pub fn scan_directory(parser: &dyn LogParser, config: &mut AnalyticsConfig) {
    for dir in parser.log_directories() {
        if !dir.exists() {
            continue;
        }
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for line in content.lines() {
                        if let Some(inv) = parser.parse_line(line) {
                            config.increment_invocation(&inv.skill_name, &inv.provider_id);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claude_parses_executed_tool() {
        let p = ClaudeCodeLogParser;
        let line = "2026-05-01 14:32:00 INFO executed tool `web-browsing-tool' with args ...";
        let inv = p.parse_line(line).unwrap();
        assert_eq!(inv.skill_name, "web-browsing-tool");
        assert_eq!(inv.provider_id, "claude-code");
    }

    #[test]
    fn claude_parses_skill_invoked() {
        let p = ClaudeCodeLogParser;
        let line = "skill `react-parser' invoked by user";
        let inv = p.parse_line(line).unwrap();
        assert_eq!(inv.skill_name, "react-parser");
    }

    #[test]
    fn claude_no_match() {
        let p = ClaudeCodeLogParser;
        assert!(p.parse_line("some random log line").is_none());
    }

    #[test]
    fn copilot_parses_tool_call() {
        let p = CopilotLogParser;
        let line = "tool call: web-browsing-tool";
        let inv = p.parse_line(line).unwrap();
        assert_eq!(inv.skill_name, "web-browsing-tool");
        assert_eq!(inv.provider_id, "copilot");
    }

    #[test]
    fn opencode_parses_executing_skill() {
        let p = OpenCodeLogParser;
        let line = "executing skill `git-helper' ...";
        let inv = p.parse_line(line).unwrap();
        assert_eq!(inv.skill_name, "git-helper");
        assert_eq!(inv.provider_id, "opencode");
    }

    #[test]
    fn scan_directory_updates_config() {
        let dir = tempfile::tempdir().unwrap();
        let log_dir = dir.path().join("logs");
        std::fs::create_dir_all(&log_dir).unwrap();
        std::fs::write(log_dir.join("claude.log"), "executed tool `test-skill'\n").unwrap();

        let log_dir_path = log_dir.clone();
        struct TestParser {
            dir: std::path::PathBuf,
        }
        impl LogParser for TestParser {
            fn provider_id(&self) -> &str {
                "test"
            }
            fn log_directories(&self) -> Vec<PathBuf> {
                vec![self.dir.clone()]
            }
            fn parse_line(&self, line: &str) -> Option<SkillInvocation> {
                extract_after_prefix(line, "executed tool `").map(|name| SkillInvocation {
                    skill_name: name.to_string(),
                    provider_id: "test".to_string(),
                    timestamp: Utc::now(),
                })
            }
        }

        let mut config = AnalyticsConfig::default();
        scan_directory(&TestParser { dir: log_dir_path }, &mut config);
        let skill = config.skills.get("test-skill").unwrap();
        assert_eq!(skill.total_invocations, 1);
    }
}
