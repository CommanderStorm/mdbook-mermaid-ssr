// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub mod config;
pub mod renderer;

use anyhow::Context;
use config::{Config, ErrorHandling};
use mdbook_preprocessor::book::{Book, BookItem};
use mdbook_preprocessor::errors::Result;
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};
use pulldown_cmark::{CodeBlockKind::*, Event, Options, Parser, Tag, TagEnd};

pub struct Mermaid {
    renderer: renderer::Mermaid,
    config: Config,
}

impl Mermaid {
    pub fn new(config: Config) -> Result<Self> {
        let renderer = renderer::Mermaid::try_init_with_config(&config)
            .context("Failed to initialize SSR renderer. Chrome/Chromium must be installed.")?;
        Ok(Self { renderer, config })
    }
}

impl Preprocessor for Mermaid {
    fn name(&self) -> &str {
        "mermaid-ssr"
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        log::info!("Rendering mermaid diagrams with SSR");

        let mut res = None;
        book.for_each_mut(|item: &mut BookItem| {
            if let Some(Err(_)) = res {
                return;
            }

            if let BookItem::Chapter(ref mut chapter) = *item {
                res = Some(
                    add_mermaid(&chapter.content, &self.renderer, &self.config).map(|md| {
                        chapter.content = md;
                    }),
                );
            }
        });

        res.unwrap_or(Ok(())).map(|_| book)
    }

    fn supports_renderer(&self, renderer: &str) -> Result<bool> {
        Ok(renderer == "html")
    }
}

fn add_mermaid(content: &str, renderer: &renderer::Mermaid, config: &Config) -> Result<String> {
    let mut mermaid_content = String::new();
    let mut in_mermaid_block = false;

    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);

    let mut code_span = 0..0;
    let mut start_new_code_span = true;

    let mut mermaid_blocks = vec![];

    let events = Parser::new_ext(content, opts);
    for (e, span) in events.into_offset_iter() {
        log::trace!("e={e:?}, span={span:?}");
        if let Event::Start(Tag::CodeBlock(Fenced(code))) = e {
            if code.as_ref() == "mermaid" {
                in_mermaid_block = true;
                mermaid_content.clear();
            }
            continue;
        }

        if !in_mermaid_block {
            continue;
        }

        // We're in the code block. The text is what we want.
        // Code blocks can come in multiple text events.
        if let Event::Text(_) = e {
            if start_new_code_span {
                code_span = span;
                start_new_code_span = false;
            } else {
                code_span = code_span.start..span.end;
            }

            continue;
        }

        if let Event::End(TagEnd::CodeBlock) = e {
            in_mermaid_block = false;

            let mermaid_content = &content[code_span.clone()];

            // Render to SVG directly using SSR
            let mermaid_code = match renderer.render(mermaid_content) {
                Ok(svg) => {
                    log::info!("Successfully rendered mermaid diagram to SVG");
                    format!("{svg}\n\n")
                }
                Err(e) => {
                    log::error!(
                        "Failed to render mermaid diagram: {e}. Content: {mermaid_content}"
                    );

                    // Handle error based on configuration
                    match config.on_error {
                        ErrorHandling::Fail => {
                            return Err(e);
                        }
                        ErrorHandling::Comment => {
                            let mermaid_code = mermaid_content
                                .replace("```", "``\\`")
                                .lines()
                                .collect::<Vec<_>>()
                                .join("\n> ");
                            format!(
                                r#"> [!IMPORTANT]
> **Mermaid diagram rendering failed during SSR because:**
> ```raw
> {e}
> ```
>
> This is the diagram code that caused the error:
> ```raw
> {mermaid_code}
> ```
>
> To fix this issue, please follow these steps:
> - Check your Mermaid code for any syntax errors by pasting it into the [Mermaid Playground](https://mermaid.live/).
> - Look at the stdout log produced during mdbook build for more details
>
> <sub><sub>You are seeing this message because the setting `on-error` is `comment` and not `fail`.</sub></sub>"#,
                            )
                        }
                    }
                }
            };

            mermaid_blocks.push((span, mermaid_code));
            start_new_code_span = true;
        }
    }

    let mut content = content.to_string();
    for (span, block) in mermaid_blocks.iter().rev() {
        let pre_content = &content[0..span.start];
        let post_content = &content[span.end..];
        content = format!("{pre_content}\n{block}{post_content}");
    }
    Ok(content)
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::{add_mermaid, renderer};
    use crate::config::Config;

    #[test]
    fn adds_mermaid() {
        let mermaid = renderer::Mermaid::try_init().unwrap();
        let config = Config::default();
        let content = r#"# Chapter

```mermaid
graph TD
A --> B
```

Text
"#;

        let result = add_mermaid(content, &mermaid, &config).unwrap();

        // Check that SVG was generated
        assert!(result.contains("<svg"));
        assert!(result.contains("</svg>"));
        assert!(result.contains("# Chapter"));
        assert!(result.contains("Text"));
    }

    #[test]
    fn leaves_tables_untouched() {
        // Regression test.
        // Previously we forgot to enable the same markdwon extensions as mdbook itself.
        let mermaid = renderer::Mermaid::try_init().unwrap();
        let config = Config::default();

        let content = r#"# Heading

| Head 1 | Head 2 |
|--------|--------|
| Row 1  | Row 2  |
"#;

        let expected = r#"# Heading

| Head 1 | Head 2 |
|--------|--------|
| Row 1  | Row 2  |
"#;

        assert_eq!(expected, add_mermaid(content, &mermaid, &config).unwrap());
    }

    #[test]
    fn leaves_html_untouched() {
        // Regression test.
        // Don't remove important newlines for syntax nested inside HTML
        let mermaid = renderer::Mermaid::try_init().unwrap();
        let config = Config::default();

        let content = r#"# Heading

<del>

*foo*

</del>
"#;

        let expected = r#"# Heading

<del>

*foo*

</del>
"#;

        assert_eq!(expected, add_mermaid(content, &mermaid, &config).unwrap());
    }

    #[test]
    fn html_in_list() {
        // Regression test.
        // Don't remove important newlines for syntax nested inside HTML
        let mermaid = renderer::Mermaid::try_init().unwrap();
        let config = Config::default();

        let content = r#"# Heading

1. paragraph 1
   ```
   code 1
   ```
2. paragraph 2
"#;

        let expected = r#"# Heading

1. paragraph 1
   ```
   code 1
   ```
2. paragraph 2
"#;

        assert_eq!(expected, add_mermaid(content, &mermaid, &config).unwrap());
    }

    #[test]
    fn escape_in_mermaid_block() {
        let _ = env_logger::try_init();
        let mermaid = renderer::Mermaid::try_init().unwrap();
        let config = Config::default();
        let content = r#"
```mermaid
classDiagram
    class PingUploader {
        <<interface>>
        +Upload() UploadResult
    }
```

hello
"#;

        let result = add_mermaid(content, &mermaid, &config).unwrap();

        // Check that SVG was generated and contains the interface markers
        assert!(result.contains("<svg"));
        assert!(result.contains("</svg>"));
        assert!(result.contains("hello"));
    }

    #[test]
    fn more_backticks() {
        let _ = env_logger::try_init();
        let mermaid = renderer::Mermaid::try_init().unwrap();
        let config = Config::default();
        let content = r#"# Chapter

````mermaid
graph TD
A --> B
````

Text
"#;

        let result = add_mermaid(content, &mermaid, &config).unwrap();

        // Check that SVG was generated
        assert!(result.contains("<svg"));
        assert!(result.contains("</svg>"));
        assert!(result.contains("# Chapter"));
        assert!(result.contains("Text"));
    }

    #[test]
    fn crlf_line_endings() {
        let _ = env_logger::try_init();
        let mermaid = renderer::Mermaid::try_init().unwrap();
        let config = Config::default();
        let content = "# Chapter\r\n\r\n````mermaid\r\n\r\ngraph TD\r\nA --> B\r\n````";

        let result = add_mermaid(content, &mermaid, &config).unwrap();

        // Check that SVG was generated
        assert!(result.contains("<svg"));
        assert!(result.contains("</svg>"));
    }

    #[test]
    fn test_on_error_fail() {
        let _ = env_logger::try_init();
        let mermaid = renderer::Mermaid::try_init().unwrap();
        let mut config = Config::default();
        config.on_error = crate::config::ErrorHandling::Fail;

        let content = r#"
```mermaid
grph TD
A --> B
```
"#;

        let result = add_mermaid(content, &mermaid, &config);
        assert!(
            result.is_err(),
            "Expected error when on_error is set to fail"
        );
    }

    #[test]
    fn test_on_error_comment() {
        let _ = env_logger::try_init();
        let mermaid = renderer::Mermaid::try_init().unwrap();
        let mut config = Config::default();
        config.on_error = crate::config::ErrorHandling::Comment;

        let content = r#"
```mermaid
grph TD
A --> B
```
"#;

        let result = add_mermaid(content, &mermaid, &config);
        assert!(
            result.is_ok(),
            "Expected success when on_error is set to comment"
        );
        let output = result.unwrap();
        assert!(output.contains("[!IMPORTANT]"));
        assert!(output.contains("Mermaid diagram rendering failed during SSR"));
    }
}
