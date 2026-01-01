use mdbook_preprocessor::PreprocessorContext;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};

pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Configuration for the mermaid-ssr preprocessor
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "kebab-case"))]
pub struct Config {
    /// Timeout for rendering operations in milliseconds
    #[serde(default = "default_timeout", with = "humantime_serde")]
    pub timeout: Duration,

    /// How to handle rendering errors
    #[serde(default)]
    pub on_error: ErrorHandling,

    /// Custom path to Chrome/Chromium executable
    pub chrome_path: Option<PathBuf>,

    /// Mermaid configuration options (will be passed to mermaid.initialize())
    #[serde(flatten)]
    pub mermaid: MermaidConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_TIMEOUT,
            on_error: ErrorHandling::default(),
            chrome_path: None,
            mermaid: MermaidConfig::default(),
        }
    }
}

fn default_timeout() -> Duration {
    DEFAULT_TIMEOUT
}

/// Mermaid initialization options
/// See: https://mermaid.js.org/config/setup/modules/mermaidAPI.html#mermaidapi-configuration-defaults
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "kebab-case"))]
pub struct MermaidConfig {
    #[serde(default)]
    pub security_level: SecurityLevel,

    /// Always false for SSR - we control rendering
    #[serde(default)]
    pub start_on_load: bool,

    /// Additional mermaid configuration options
    #[serde(flatten)]
    pub additional: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "kebab-case"))]
pub enum SecurityLevel {
    /// HTML tags in the text are encoded and click functionality is disabled.
    #[default]
    Strict,
    /// HTML tags in text are allowed and click functionality is enabled.
    Loose,
    /// HTML tags in text are allowed (only script elements are removed), and click functionality is enabled.
    Antiscript,
    /// With this security level, all rendering takes place in a sandboxed iframe. This prevent any JavaScript from running in the context. This may hinder interactive functionality of the diagram, like scripts, popups in the sequence diagram, or links to other tabs or targets, etc.
    Sandbox,
}

impl Config {
    /// Parse configuration from PreprocessorContext
    pub fn from_context(ctx: &PreprocessorContext) -> Self {
        const NAME: &str = "mermaid-ssr";

        ctx.config
            .preprocessors::<Config>()
            .ok()
            .and_then(|mut map| map.remove(NAME))
            .unwrap_or_else(|| {
                log::debug!("No configuration found for {}. Using defaults.", NAME);
                Config::default()
            })
    }

    /// Build the mermaid initialization script with all configured options
    pub fn build_mermaid_init_script(&self) -> String {
        // Clone and convert additional kebab-case keys to camelCase
        let mut mermaid_config = self.mermaid.clone();
        let additional: serde_json::Map<String, serde_json::Value> = mermaid_config
            .additional
            .into_iter()
            .map(|(key, value)| (kebab_to_camel(&key), value))
            .collect();
        mermaid_config.additional = additional;

        let config_json =
            serde_json::to_string(&mermaid_config).expect("Failed to serialize mermaid config");

        format!(
            r#"mermaid.initialize({});

window.render = async function(id, code) {{
    try {{
        const {{ svg }} = await mermaid.render(id, code);
        return svg;
    }} catch (error) {{
        console.error('Mermaid rendering error:', error);
        return null;
    }}
}};"#,
            config_json
        )
    }
}

/// How to handle rendering errors
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "kebab-case"))]
pub enum ErrorHandling {
    /// Fail the build on rendering errors (default)
    #[default]
    Fail,
    /// Emit HTML comments on rendering errors
    Comment,
}

fn kebab_to_camel(s: &str) -> String {
    let mut iter = s.split('-');
    let mut result = iter.next().unwrap_or("").to_string();

    for part in iter {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            result.push(first.to_ascii_uppercase());
            result.extend(chars);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_init_script_defaults() {
        let config = Config::default();
        let script = config.build_mermaid_init_script();
        insta::assert_snapshot!(script, @r#"
        mermaid.initialize({"securityLevel":"strict","startOnLoad":false});

        window.render = async function(id, code) {
            try {
                const { svg } = await mermaid.render(id, code);
                return svg;
            } catch (error) {
                console.error('Mermaid rendering error:', error);
                return null;
            }
        };
        "#);
    }

    #[test]
    fn test_build_init_script_with_additional_options() {
        let mut config = Config::default();
        config
            .mermaid
            .additional
            .insert("look".to_string(), "hand-drawn".to_string().into());
        let script = config.build_mermaid_init_script();
        insta::assert_snapshot!(script, @r#"
        mermaid.initialize({"securityLevel":"strict","startOnLoad":false,"look":"hand-drawn"});

        window.render = async function(id, code) {
            try {
                const { svg } = await mermaid.render(id, code);
                return svg;
            } catch (error) {
                console.error('Mermaid rendering error:', error);
                return null;
            }
        };
        "#);
    }

    #[test]
    fn test_build_init_script_with_security_level() {
        let mut config = Config::default();
        config.mermaid.security_level = SecurityLevel::Antiscript;
        let script = config.build_mermaid_init_script();
        insta::assert_snapshot!(script, @r#"
        mermaid.initialize({"securityLevel":"antiscript","startOnLoad":false});

        window.render = async function(id, code) {
            try {
                const { svg } = await mermaid.render(id, code);
                return svg;
            } catch (error) {
                console.error('Mermaid rendering error:', error);
                return null;
            }
        };
        "#);
    }

    #[test]
    fn test_kebab_to_camel_does_convert_kebab_case() {
        assert_eq!(kebab_to_camel("security-level"), "securityLevel");
        assert_eq!(kebab_to_camel("font-family"), "fontFamily");
        assert_eq!(kebab_to_camel("theme"), "theme");
        assert_eq!(kebab_to_camel("flowchart-curve"), "flowchartCurve");
    }

    #[test]
    fn test_kebab_to_camel_does_not_convert_camel_case() {
        assert_eq!(kebab_to_camel("securityLevel"), "securityLevel");
        assert_eq!(kebab_to_camel("fontFamily"), "fontFamily");
        assert_eq!(kebab_to_camel("theme"), "theme");
        assert_eq!(kebab_to_camel("flowchartCurve"), "flowchartCurve");
    }
}
