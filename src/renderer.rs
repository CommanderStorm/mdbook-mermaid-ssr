use std::sync::Arc;
use std::time::Duration;

use anyhow::{Result, bail};
use escape_string::escape;
use headless_chrome::{Browser, LaunchOptions, Tab};
use serde_json::Value;
use unescape::unescape;

/// Default timeout for Chrome operations (30 seconds)
pub const DEFAULT_TIMEOUT_SECS: Duration = Duration::from_secs(30);

/// The Mermaid struct holds the embedded Chromium instance that is used to render Mermaid
/// diagrams
#[derive(Clone)]
pub struct Mermaid {
    _browser: Browser,
    tab: Arc<Tab>,
}

impl Mermaid {
    /// Initializes Mermaid with default timeout settings
    pub fn try_init() -> Result<Self> {
        Self::try_init_with_timeout(DEFAULT_TIMEOUT_SECS)
    }

    /// Initializes Mermaid with custom timeout
    ///
    /// # Arguments
    /// * `timeout` - Maximum duration for Chrome operations (navigation, evaluation, etc.)
    ///
    /// # Example:
    /// ```no_run
    /// # use mdbook_mermaid_ssr::renderer::Mermaid;
    /// # use std::time::Duration;
    /// let mermaid = Mermaid::try_init_with_timeout(Duration::from_secs(60))
    ///     .expect("Failed to initialize");
    /// ```
    pub fn try_init_with_timeout(timeout: Duration) -> Result<Self> {
        // Configure browser with timeout settings
        let launch_options = LaunchOptions::default_builder()
            .idle_browser_timeout(timeout)
            .build()?;

        let browser = Browser::new(launch_options)?;
        let mermaid_js = include_str!("../payload/mermaid.js");
        let html_payload = include_str!("../payload/index.html");

        let tab = browser.new_tab()?;

        // Set default timeout for tab operations
        tab.set_default_timeout(timeout);

        tab.navigate_to(&format!("data:text/html;charset=utf-8,{}", html_payload))?;

        // Load mermaid library
        tab.evaluate(mermaid_js, false)?;

        // Initialize mermaid and set up render function in global scope
        let init_script = r#"
                mermaid.initialize({
                    startOnLoad: false,
                    theme: 'default',
                    securityLevel: 'loose'
                });

                window.render = async function(code) {
                    try {
                        const { svg } = await mermaid.render('mermaid-diagram-' + Date.now(), code);
                        return svg;
                    } catch (error) {
                        console.error('Mermaid rendering error:', error);
                        return null;
                    }
                };
            "#;
        tab.evaluate(init_script, false)?;

        Ok(Self {
            _browser: browser,
            tab,
        })
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
        // Call the async render function and await its result
        let script = format!(
            "(async () => {{ return await window.render('{}'); }})()",
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_mermaid_instance_without_crashing() {
        let mermaid = Mermaid::try_init();
        assert!(mermaid.is_ok());
    }

    #[test]
    fn render_mermaid() {
        let mermaid = Mermaid::try_init().unwrap();
        let rendered = mermaid.render("graph TB\na-->b");
        if let Err(ref e) = rendered {
            log::error!("Render error: {}", e);
        }
        assert!(
            rendered.is_ok(),
            "Failed to render mermaid diagram: {:?}",
            rendered.err()
        );
        // TODO: Perform visual image comparison
        assert!(rendered.unwrap().starts_with("<svg"));
    }

    #[test]
    fn syntax_error() {
        let mermaid = Mermaid::try_init().unwrap();
        let rendered = mermaid.render("grph TB\na-->b");
        assert!(rendered.is_err());
    }

    #[test]
    fn test_custom_timeout() {
        use std::time::Duration;
        let mermaid = Mermaid::try_init_with_timeout(Duration::from_secs(60));
        assert!(mermaid.is_ok());
    }
}
