use std::sync::Arc;

use anyhow::{Result, bail};
use escape_string::escape;
use headless_chrome::{Browser, LaunchOptions, Tab};
use serde_json::Value;
use unescape::unescape;

use crate::config::Config;

/// The Mermaid struct holds the embedded Chromium instance that is used to render Mermaid
/// diagrams
#[derive(Clone)]
pub struct Mermaid {
    browser: Browser,
    tab: Arc<Tab>,
}

impl Mermaid {
    /// Initializes Mermaid with default timeout settings
    pub fn try_init() -> Result<Self> {
        Self::try_init_with_config(&Config::default())
    }

    /// Initializes Mermaid with a configuration object
    ///
    /// # Arguments
    /// * `config` - Configuration for the renderer
    ///
    /// # Example:
    /// ```no_run
    /// # use mdbook_mermaid_ssr::renderer::Mermaid;
    /// # use mdbook_mermaid_ssr::config::Config;
    /// let config = Config::default();
    /// let mermaid = Mermaid::try_init_with_config(&config)
    ///     .expect("Failed to initialize");
    /// ```
    pub fn try_init_with_config(config: &Config) -> Result<Self> {
        // Configure browser with timeout settings and optional custom chrome path
        let mut launch_options_builder = LaunchOptions::default_builder();
        launch_options_builder.idle_browser_timeout(config.timeout);

        if let Some(ref chrome_path) = config.chrome_path {
            launch_options_builder.path(Some(chrome_path.into()));
        }

        let launch_options = launch_options_builder.build()?;

        let browser = Browser::new(launch_options)?;
        let mermaid_js = include_str!("../payload/mermaid.js");
        let html_payload = include_str!("../payload/index.html");

        let tab = browser.new_tab()?;
        tab.set_default_timeout(config.timeout);
        tab.navigate_to(&format!("data:text/html;charset=utf-8,{}", html_payload))?;
        // Load mermaid library
        tab.evaluate(mermaid_js, false)?;
        // Initialize mermaid with configured options and set up render function
        let init_script = config.build_mermaid_init_script();
        tab.evaluate(&init_script, false)?;

        Ok(Self { browser, tab })
    }

    /// Renders a diagram
    ///
    /// # Example:
    /// ```no_run
    /// # use mdbook_mermaid_ssr::renderer::Mermaid;
    /// let mermaid = Mermaid::try_init().expect("Failed to initialize");
    /// let svg = mermaid.render("graph TB\na-->b").expect("Unable to render!");
    /// ```
    pub fn render(&self, input: &str) -> Result<String> {
        let id = fxhash::hash(input);
        // Call the async render function and await its result
        let script = format!(
            "(async () => {{ return await window.render('mermaid-diagram-{id}', '{}'); }})()",
            escape(input)
        );
        let data = self.tab.evaluate(&script, true)?;

        // Use proper JSON parsing instead of fragile string operations
        let svg = match data.value {
            Some(Value::String(s)) => {
                // Unescape the string value if needed
                unescape(&s).unwrap_or(s)
            }
            Some(Value::Null) | None => {
                bail!("Failed to compile Mermaid diagram: render returned null");
            }
            Some(other) => {
                bail!("Unexpected return type from render: {:?}", other);
            }
        };

        if svg.is_empty() {
            bail!("Failed to compile Mermaid diagram: empty result");
        }

        Ok(svg)
    }
    /// Gives access to the underlying browser instance
    pub fn browser(&self) -> &Browser {
        &self.browser
    }
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use crate::config::SecurityLevel;

    use super::*;

    #[test]
    fn create_mermaid_instance_without_crashing() {
        let mermaid = Mermaid::try_init();
        assert!(mermaid.is_ok());
    }

    #[test]
    fn syntax_error() {
        let mermaid = Mermaid::try_init().unwrap();
        let rendered = mermaid.render(
            r#"
grph TB
    a-->b"#,
        );
        assert!(rendered.is_err());
    }

    #[test]
    fn test_with_config() {
        let mut config = Config::default();
        config.mermaid.security_level = SecurityLevel::Strict;
        let mermaid = Mermaid::try_init_with_config(&config);
        assert!(mermaid.is_ok());
    }

    #[test]
    fn test_security_level_strict() {
        let mut config = Config::default();
        config.mermaid.security_level = SecurityLevel::Strict;
        let mermaid = Mermaid::try_init_with_config(&config).unwrap();

        // Test with HTML tags - in strict mode, HTML should be encoded
        let diagram = r#"graph TD
    A["<b>Bold</b> <i>Italic</i>"]
    B["<span>Span text</span>"]
    A -->|"<em>edge label</em>"| B"#;

        let svg = mermaid.render(diagram).unwrap();
        insta::assert_snapshot!("security_level_strict", format_html(svg));
    }

    #[test]
    fn test_security_level_loose() {
        let mut config = Config::default();
        config.mermaid.security_level = SecurityLevel::Loose;
        let mermaid = Mermaid::try_init_with_config(&config).unwrap();

        // Test with HTML tags - in loose mode, HTML is allowed
        let diagram = r#"graph TD
    A["<b>Bold</b> <i>Italic</i>"]
    B["<span>Styled</span> text"]
    A -->|"<em>edge</em>"| B"#;

        let svg = mermaid.render(diagram).unwrap();
        insta::assert_snapshot!("security_level_loose", format_html(svg));
    }

    #[test]
    fn test_security_level_antiscript() {
        let mut config = Config::default();
        config.mermaid.security_level = SecurityLevel::Antiscript;
        let mermaid = Mermaid::try_init_with_config(&config).unwrap();

        // Test with HTML tags - antiscript allows HTML but removes script elements
        let diagram = r#"graph TD
    A["<b>Bold</b> <strong>Strong</strong>"]
    B["<i>Italic</i> <em>Emphasis</em>"]
    A -->|"<span>edge</span>"| B"#;

        let svg = mermaid.render(diagram).unwrap();
        insta::assert_snapshot!("security_level_antiscript", format_html(svg));
    }

    #[test]
    fn test_security_level_sandbox() {
        let mut config = Config::default();
        config.mermaid.security_level = SecurityLevel::Sandbox;
        let mermaid = Mermaid::try_init_with_config(&config).unwrap();

        // Test sandbox mode - all rendering in sandboxed context
        let diagram = r#"graph TD
    A["<b>Text</b> in sandbox"]
    B["Isolated rendering"]
    A --> B"#;

        let svg = mermaid.render(diagram).unwrap();
        insta::assert_snapshot!("security_level_sandbox", format_html(svg));
    }

    /// Format HTML with proper indentation for better readability in snapshots
    fn format_html(html: impl AsRef<str>) -> String {
        // Try to format with oxfmt if available
        let result = Command::new("npx")
            .arg("oxfmt@latest")
            .arg("--stdin-filepath")
            .arg("snapshot.html")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();

        match result {
            Ok(mut child) => {
                use std::io::Write;
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(html.as_ref().as_bytes());
                }

                match child.wait_with_output() {
                    Ok(output) if output.status.success() => {
                        String::from_utf8_lossy(&output.stdout).to_string()
                    }
                    _ => {
                        eprintln!("Warning: oxfmt formatting failed, using original HTML");
                        html.as_ref().to_string()
                    }
                }
            }
            Err(_) => {
                eprintln!("Warning: oxfmt not available, using original HTML");
                html.as_ref().to_string()
            }
        }
    }
}
