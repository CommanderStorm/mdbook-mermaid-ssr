use log::info;
use pretty_assertions::assert_ne;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Once;
use mdbook_mermaid_ssr::renderer::Oxfmt;

static BUILD_BINARY_ONCE: Once = Once::new();
static BUILD_TEST_BOOK: Once = Once::new();
static BUILD_ERROR_COMMENT: Once = Once::new();
static BUILD_THEME_FOREST: Once = Once::new();
static BUILD_FULL_CONFIG: Once = Once::new();

fn test_book_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

fn output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("books")
}

fn ensure_binary_built() {
    BUILD_BINARY_ONCE.call_once(|| {
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
    });
}

fn build_book(book_name: &str) -> PathBuf {
    ensure_binary_built();

    info!("Building book {book_name}");
    let output_dir = output_dir().join(book_name);
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir).expect("Failed to clean output directory");
    }

    let binary_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("debug")
        .join("mdbook-mermaid-ssr");

    let path_env = std::env::var("PATH").unwrap_or_default();
    let binary_dir = binary_path.parent().unwrap();
    let new_path = format!("{path_env}:{}", binary_dir.display());

    let book_dir = test_book_dir().join(book_name);
    let status = Command::new("mdbook")
        .arg("build")
        .arg("--dest-dir")
        .arg(&output_dir)
        .current_dir(&book_dir)
        .env("PATH", new_path)
        .status()
        .expect("Failed to run mdbook build");

    assert!(status.success(), "mdbook {book_name} build failed");
    output_dir
}

fn extract_main_content(html: &str) -> &str {
    html.split("<main>")
        .nth(1)
        .expect("Failed to find <main> tag")
        .split("</main>")
        .next()
        .expect("Failed to find </main> tag")
}

#[test]
fn test_book_builds() {
    BUILD_TEST_BOOK.call_once(|| {
        build_book("test-book");
    });

    let output = output_dir().join("test-book");
    assert!(output.exists(), "Output directory should exist");
    assert!(
        output.join("index.html").exists(),
        "index.html should exist"
    );
}

#[test]
fn test_chapter_with_mermaid() {
    BUILD_TEST_BOOK.call_once(|| {
        build_book("test-book");
    });

    let output = output_dir().join("test-book");
    let content = fs::read_to_string(output.join("chapter_with_mermaid.html"))
        .expect("Failed to read chapter_with_mermaid.html");

    assert!(
        content.contains("<svg") && content.contains("flowchart"),
        "Chapter with mermaid should contain mermaid-generated SVG"
    );
    assert!(
        !content.contains("```mermaid"),
        "Should not contain raw mermaid blocks"
    );

    let main_content = extract_main_content(&content);
    let formatted = Oxfmt::format(main_content).expect("Failed to format SVG");
    insta::assert_snapshot!("chapter_with_mermaid", formatted);
}

#[test]
fn test_chapter_without_mermaid() {
    BUILD_TEST_BOOK.call_once(|| {
        build_book("test-book");
    });

    let output = output_dir().join("test-book");
    let content = fs::read_to_string(output.join("chapter_without_mermaid.html"))
        .expect("Failed to read chapter_without_mermaid.html");

    assert!(
        !content.contains("flowchart") && !content.contains("sequenceDiagram"),
        "Chapter without mermaid should not contain mermaid diagram content"
    );
    assert!(content.contains("rust"), "Should preserve rust code blocks");

    let main_content = extract_main_content(&content);
    let formatted = Oxfmt::format(main_content).expect("Failed to format SVG");
    insta::assert_snapshot!("chapter_without_mermaid", formatted);
}

#[test]
fn test_config_on_error_comment() {
    BUILD_ERROR_COMMENT.call_once(|| {
        build_book("test-book-error-comment");
    });

    let output = output_dir().join("test-book-error-comment");

    assert!(output.exists(), "Output directory should exist");
    assert!(
        output.join("index.html").exists(),
        "index.html should exist"
    );

    let content = fs::read_to_string(output.join("chapter_with_invalid_mermaid.html"))
        .expect("Failed to read chapter_with_invalid_mermaid.html");

    assert!(
        content.contains("<svg"),
        "Should contain valid rendered diagrams"
    );
    assert!(
        content.contains("<!-- Mermaid") || content.contains("<!--"),
        "Should contain HTML comment for rendering error"
    );
    assert!(
        content.contains("Alice") && content.contains("Bob"),
        "Valid diagrams after error should still render"
    );

    let main_content = extract_main_content(&content);
    let formatted = Oxfmt::format(main_content).expect("Failed to format SVG");
    insta::assert_snapshot!("error_comment_handling", formatted);
}

#[test]
fn test_config_theme_forest() {
    BUILD_THEME_FOREST.call_once(|| {
        build_book("test-book-theme-forest");
    });

    let output = output_dir().join("test-book-theme-forest");
    assert!(output.exists(), "Output directory should exist");

    let content =
        fs::read_to_string(output.join("chapter.html")).expect("Failed to read chapter.html");

    assert!(
        content.contains("<svg"),
        "Should contain rendered mermaid diagram"
    );
    assert!(
        content.contains("Forest Theme"),
        "Should contain diagram with forest theme content"
    );
    assert!(
        content.contains("graph") || content.contains("flowchart"),
        "Should contain graph/flowchart elements"
    );

    let main_content = extract_main_content(&content);
    let formatted = Oxfmt::format(main_content).expect("Failed to format SVG");
    insta::assert_snapshot!("theme_forest", formatted);
}

#[test]
fn test_config_full_configuration() {
    BUILD_FULL_CONFIG.call_once(|| {
        build_book("test-book-full-config");
    });

    let output = output_dir().join("test-book-full-config");
    assert!(output.exists(), "Output directory should exist");

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

    let dark_main = extract_main_content(&dark_content);
    let dark_formatted = Oxfmt::format(dark_main).expect("Failed to format SVG");
    insta::assert_snapshot!("full_config_dark_theme", dark_formatted);

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

    let hand_drawn_main = extract_main_content(&hand_drawn_content);
    let hand_drawn_formatted = Oxfmt::format(hand_drawn_main).expect("Failed to format svg");
    insta::assert_snapshot!("full_config_hand_drawn", hand_drawn_formatted);

    let multiple_content = fs::read_to_string(output.join("multiple_diagrams.html"))
        .expect("Failed to read multiple_diagrams.html");

    let svg_count = multiple_content.matches("<svg").count();
    assert!(
        svg_count >= 5,
        "Should contain at least 5 SVG diagrams, found {svg_count}"
    );
    assert!(
        multiple_content.contains("flowchart") || multiple_content.contains("graph"),
        "Should contain flowchart/graph diagram"
    );
    assert!(
        multiple_content.contains("sequenceDiagram") || multiple_content.contains("sequence"),
        "Should contain sequence diagram"
    );

    let multiple_main = extract_main_content(&multiple_content);
    let multiple_formatted = Oxfmt::format(multiple_main).expect("Failed to format svg");
    insta::assert_snapshot!("full_config_multiple_diagrams", multiple_formatted);
}

#[test]
fn test_config_security_level_in_output() {
    BUILD_FULL_CONFIG.call_once(|| {
        build_book("test-book-full-config");
    });

    let output = output_dir().join("test-book-full-config");

    let content =
        fs::read_to_string(output.join("dark_theme.html")).expect("Failed to read dark_theme.html");

    assert!(
        content.contains("<svg"),
        "Diagrams should render with loose security level"
    );
}

#[test]
fn test_all_diagram_types_with_config() {
    BUILD_FULL_CONFIG.call_once(|| {
        build_book("test-book-full-config");
    });

    let output = output_dir().join("test-book-full-config");

    let content = fs::read_to_string(output.join("multiple_diagrams.html"))
        .expect("Failed to read multiple_diagrams.html");

    let main_content = extract_main_content(&content);

    let svg_count = main_content.matches("<svg").count();
    assert!(
        svg_count >= 5,
        "Should have at least 5 different diagram types rendered, found {svg_count}"
    );

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
    BUILD_FULL_CONFIG.call_once(|| {
        build_book("test-book-full-config");
    });
    let full_config_output = output_dir().join("test-book-full-config");
    let full_config_content = fs::read_to_string(full_config_output.join("dark_theme.html"))
        .expect("Failed to read dark_theme.html");

    BUILD_TEST_BOOK.call_once(|| {
        build_book("test-book");
    });
    let output = output_dir().join("test-book");
    let default_content = fs::read_to_string(output.join("chapter_with_mermaid.html"))
        .expect("Failed to read default chapter");

    assert!(
        full_config_content.contains("<svg"),
        "Full config should have SVG"
    );
    assert!(
        default_content.contains("<svg"),
        "Default config should have SVG"
    );

    assert_ne!(
        full_config_content, default_content,
        "Configuration should affect the final HTML output"
    );
}
