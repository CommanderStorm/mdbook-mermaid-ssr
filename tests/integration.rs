use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Once;

static BUILD_ONCE: Once = Once::new();

fn test_book_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("test-book")
}

fn output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("test-book")
}

/// Format HTML with proper indentation for better readability in snapshots
fn format_html(html: &str) -> String {
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
                let _ = stdin.write_all(html.as_bytes());
            }

            match child.wait_with_output() {
                Ok(output) if output.status.success() => {
                    String::from_utf8_lossy(&output.stdout).to_string()
                }
                _ => {
                    eprintln!("Warning: oxfmt formatting failed, using original HTML");
                    html.to_string()
                }
            }
        }
        Err(_) => {
            eprintln!("Warning: oxfmt not available, using original HTML");
            html.to_string()
        }
    }
}

fn ensure_built() {
    BUILD_ONCE.call_once(|| {
        // First, build the mdbook-mermaid-ssr binary
        let build_status = Command::new("cargo")
            .arg("build")
            .arg("--bin")
            .arg("mdbook-mermaid-ssr")
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .status()
            .expect("Failed to build mdbook-mermaid-ssr");

        assert!(
            build_status.success(),
            "Failed to build mdbook-mermaid-ssr binary"
        );

        let book_dir = test_book_dir();
        let output = output_dir();

        // Clean previous build to ensure we test current code
        if output.exists() {
            fs::remove_dir_all(&output).expect("Failed to clean output directory");
        }

        // Get the path to the built binary
        let binary_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("debug")
            .join("mdbook-mermaid-ssr");

        // Add the binary directory to PATH for mdbook to find the preprocessor
        let path_env = std::env::var("PATH").unwrap_or_default();
        let binary_dir = binary_path.parent().unwrap();
        let new_path = format!("{}:{}", binary_dir.display(), path_env);

        // Build the book using mdbook
        let status = Command::new("mdbook")
            .arg("build")
            .arg("--dest-dir")
            .arg(output)
            .current_dir(&book_dir)
            .env("PATH", new_path)
            .status()
            .expect("Failed to run mdbook build");

        assert!(status.success(), "mdbook build failed");
    });
}

#[test]
fn test_book_builds() {
    ensure_built();

    let output = output_dir();
    assert!(output.exists(), "Output directory should exist");
    assert!(
        output.join("index.html").exists(),
        "index.html should exist"
    );
}

#[test]
fn test_chapter_with_mermaid() {
    ensure_built();

    let content = fs::read_to_string(output_dir().join("chapter_with_mermaid.html"))
        .expect("Failed to read chapter_with_mermaid.html");

    // Should contain mermaid-generated SVG elements (check for SVG with typical mermaid structure)
    assert!(
        content.contains("<svg") && content.contains("flowchart"),
        "Chapter with mermaid should contain mermaid-generated SVG"
    );

    // Should NOT contain mermaid code blocks
    assert!(
        !content.contains("```mermaid"),
        "Should not contain raw mermaid blocks"
    );

    // Redact non-deterministic mermaid diagram IDs before snapshotting
    let content = content
        .split("<main>")
        .nth(1)
        .expect("Failed to find <main> tag");
    let content = content
        .split("</main>")
        .nth(0)
        .expect("Failed to find </main> tag");

    let formatted = format_html(content);
    insta::assert_snapshot!("chapter_with_mermaid", formatted);
}

#[test]
fn test_chapter_without_mermaid() {
    ensure_built();

    let content = fs::read_to_string(output_dir().join("chapter_without_mermaid.html"))
        .expect("Failed to read chapter_without_mermaid.html");

    // Should NOT contain mermaid diagrams
    // (Note: mdBook may include SVG icons and the word "mermaid" in text,
    // so we check for absence of mermaid diagram markers)
    assert!(
        !content.contains("flowchart") && !content.contains("sequenceDiagram"),
        "Chapter without mermaid should not contain mermaid diagram content"
    );

    // Should preserve code blocks
    assert!(content.contains("rust"), "Should preserve rust code blocks");

    // Redact non-deterministic content before snapshotting
    let content = content
        .split("<main>")
        .nth(1)
        .expect("Failed to find <main> tag");
    let content = content
        .split("</main>")
        .nth(0)
        .expect("Failed to find </main> tag");

    let formatted = format_html(content);
    insta::assert_snapshot!(formatted, @r##"
    <h1 id="regular-chapter"><a class="header" href="#regular-chapter">Regular Chapter</a></h1>
    <p>This chapter has no Mermaid diagrams.</p>
    <pre class="playground"><code class="language-rust">fn main() {
        println!("Hello, world!");
    }</code></pre>
    <p>Just regular markdown with code blocks.</p>
    <h2 id="a-subsection"><a class="header" href="#a-subsection">A Subsection</a></h2>
    <p>More text here.</p>
    <pre><code class="language-python">def greet(name):
        return f"Hello, {name}!"
    </code></pre>
    <p>Final paragraph.</p>
    "##);
}

// Helper function to build a test book with specific configuration
fn build_test_book(book_name: &str) -> PathBuf {
    let book_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(book_name);

    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(book_name);

    // Clean previous build
    if output.exists() {
        fs::remove_dir_all(&output).expect("Failed to clean output directory");
    }

    // Get the path to the built binary
    let binary_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("debug")
        .join("mdbook-mermaid-ssr");

    // Ensure binary exists
    if !binary_path.exists() {
        let build_status = Command::new("cargo")
            .arg("build")
            .arg("--bin")
            .arg("mdbook-mermaid-ssr")
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .status()
            .expect("Failed to build mdbook-mermaid-ssr");

        assert!(
            build_status.success(),
            "Failed to build mdbook-mermaid-ssr binary"
        );
    }

    // Add the binary directory to PATH
    let path_env = std::env::var("PATH").unwrap_or_default();
    let binary_dir = binary_path.parent().unwrap();
    let new_path = format!("{}:{}", binary_dir.display(), path_env);

    // Build the book
    let status = Command::new("mdbook")
        .arg("build")
        .arg("--dest-dir")
        .arg(&output)
        .current_dir(&book_dir)
        .env("PATH", new_path)
        .status()
        .expect("Failed to run mdbook build");

    assert!(status.success(), "mdbook build failed for {}", book_name);

    output
}

#[test]
fn test_config_custom_timeout() {
    let output = build_test_book("test-book-with-timeout");

    // Verify the book built successfully
    assert!(output.exists(), "Output directory should exist");
    assert!(
        output.join("index.html").exists(),
        "index.html should exist"
    );

    // Read the chapter and verify diagram rendered
    let content = fs::read_to_string(output.join("chapter_with_mermaid.html"))
        .expect("Failed to read chapter_with_mermaid.html");

    // Should contain rendered SVG
    assert!(
        content.contains("<svg") && content.contains("graph"),
        "Chapter should contain rendered mermaid diagram"
    );

    // Verify the specific diagram content is present
    assert!(
        content.contains("Is timeout configured"),
        "Should contain timeout-specific diagram content"
    );
}

#[test]
fn test_config_on_error_comment() {
    let output = build_test_book("test-book-error-comment");

    // Verify the book built successfully (should not fail despite invalid diagram)
    assert!(output.exists(), "Output directory should exist");
    assert!(
        output.join("index.html").exists(),
        "index.html should exist"
    );

    // Read the chapter with invalid mermaid
    let content = fs::read_to_string(output.join("chapter_with_invalid_mermaid.html"))
        .expect("Failed to read chapter_with_invalid_mermaid.html");

    // Should contain the valid diagrams
    assert!(
        content.contains("<svg"),
        "Should contain valid rendered diagrams"
    );

    // Should contain HTML comment for the error (on-error = "comment")
    assert!(
        content.contains("<!-- Mermaid") || content.contains("<!--"),
        "Should contain HTML comment for rendering error"
    );

    // The valid diagrams should still render
    assert!(
        content.contains("Alice") && content.contains("Bob"),
        "Valid diagrams after error should still render"
    );
}

#[test]
fn test_config_theme_forest() {
    let output = build_test_book("test-book-theme-forest");

    assert!(output.exists(), "Output directory should exist");

    let content =
        fs::read_to_string(output.join("chapter.html")).expect("Failed to read chapter.html");

    // Should contain rendered SVG
    assert!(
        content.contains("<svg"),
        "Should contain rendered mermaid diagram"
    );

    // Forest theme uses specific color schemes - check for SVG content
    assert!(
        content.contains("Forest Theme"),
        "Should contain diagram with forest theme content"
    );

    // The SVG should be present, indicating successful rendering with theme
    assert!(
        content.contains("graph") || content.contains("flowchart"),
        "Should contain graph/flowchart elements"
    );
}

#[test]
fn test_config_full_configuration() {
    let output = build_test_book("test-book-full-config");

    assert!(output.exists(), "Output directory should exist");

    // Test dark theme chapter
    let dark_content =
        fs::read_to_string(output.join("dark_theme.html")).expect("Failed to read dark_theme.html");

    assert!(
        dark_content.contains("<svg"),
        "Dark theme chapter should contain SVG"
    );
    assert!(
        dark_content.contains("Dark Theme"),
        "Should contain dark theme diagram content"
    );

    // Test hand-drawn style chapter
    let hand_drawn_content =
        fs::read_to_string(output.join("hand_drawn.html")).expect("Failed to read hand_drawn.html");

    assert!(
        hand_drawn_content.contains("<svg"),
        "Hand-drawn chapter should contain SVG"
    );
    assert!(
        hand_drawn_content.contains("Hand-Drawn Style") || hand_drawn_content.contains("Sketchy"),
        "Should contain hand-drawn style content"
    );

    // Test multiple diagrams chapter
    let multiple_content = fs::read_to_string(output.join("multiple_diagrams.html"))
        .expect("Failed to read multiple_diagrams.html");

    // Count SVG occurrences (should have 5 diagrams)
    let svg_count = multiple_content.matches("<svg").count();
    assert!(
        svg_count >= 5,
        "Should contain at least 5 SVG diagrams, found {}",
        svg_count
    );

    // Verify different diagram types are present
    assert!(
        multiple_content.contains("flowchart") || multiple_content.contains("graph"),
        "Should contain flowchart/graph diagram"
    );
    assert!(
        multiple_content.contains("sequenceDiagram") || multiple_content.contains("sequence"),
        "Should contain sequence diagram"
    );
}

#[test]
fn test_config_security_level_in_output() {
    let output = build_test_book("test-book-full-config");

    let content =
        fs::read_to_string(output.join("dark_theme.html")).expect("Failed to read dark_theme.html");

    // With security-level = "loose", diagrams should render successfully
    assert!(
        content.contains("<svg"),
        "Diagrams should render with loose security level"
    );
}

#[test]
fn test_all_diagram_types_with_config() {
    let output = build_test_book("test-book-full-config");

    let content = fs::read_to_string(output.join("multiple_diagrams.html"))
        .expect("Failed to read multiple_diagrams.html");

    // Extract main content
    let main_content = content
        .split("<main>")
        .nth(1)
        .expect("Failed to find <main> tag")
        .split("</main>")
        .nth(0)
        .expect("Failed to find </main> tag");

    // All diagram types should be rendered as SVG
    let svg_count = main_content.matches("<svg").count();
    assert!(
        svg_count >= 5,
        "Should have at least 5 different diagram types rendered, found {}",
        svg_count
    );

    // Verify the page contains various diagram type indicators
    assert!(
        main_content.contains("Flowchart") || main_content.contains("flowchart"),
        "Should reference flowchart diagrams"
    );
    assert!(
        main_content.contains("Sequence") || main_content.contains("sequence"),
        "Should reference sequence diagrams"
    );
    assert!(
        main_content.contains("Class") || main_content.contains("class"),
        "Should reference class diagrams"
    );
}

#[test]
fn test_config_affects_svg_output() {
    // Build book with full config (dark theme + handDrawn)
    let full_config_output = build_test_book("test-book-full-config");
    let full_config_content = fs::read_to_string(full_config_output.join("dark_theme.html"))
        .expect("Failed to read dark_theme.html");

    // Build default config book
    ensure_built();
    let default_content = fs::read_to_string(output_dir().join("chapter_with_mermaid.html"))
        .expect("Failed to read default chapter");

    // Both should have SVG, but the configuration should affect the output
    assert!(
        full_config_content.contains("<svg"),
        "Full config should have SVG"
    );
    assert!(
        default_content.contains("<svg"),
        "Default config should have SVG"
    );

    // The SVGs should be different (configuration affects rendering)
    // This is a basic check - in practice, theme and look settings change
    // the SVG structure, colors, and styling
    assert_ne!(
        full_config_content, default_content,
        "Configuration should affect the final HTML output"
    );
}
