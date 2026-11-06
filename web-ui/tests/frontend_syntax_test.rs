use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

fn repo_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn syntax_errors(relative: &str) -> Vec<String> {
    let path = repo_path(relative);
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{relative} must be readable: {e}"));

    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, &source, SourceType::cjs()).parse();

    parsed
        .diagnostics
        .iter()
        .map(|err| {
            let offset = (err.labels.first().map_or(0, |span| span.offset()) as usize)
                .min(source.len());
            let line = source[..offset].matches('\n').count() + 1;
            let column = offset - source[..offset].rfind('\n').map_or(0, |i| i + 1) + 1;
            format!("{relative}:{line}:{column}  {}", err.message)
        })
        .collect()
}

#[test]
fn app_js_has_no_syntax_errors() {
    let errors = syntax_errors("static/app.js");
    assert!(
        errors.is_empty(),
        "static/app.js has {} syntax error(s) and will not load in a browser:\n  {}",
        errors.len(),
        errors.join("\n  ")
    );
}

#[test]
fn app_js_and_html_have_complete_i18n_support() {
    let html_path = repo_path("static/index.html");
    let html = std::fs::read_to_string(&html_path)
        .unwrap_or_else(|e| panic!("index.html must be readable: {e}"));

    assert!(
        html.contains("id=\"lang-btn-en\""),
        "index.html must have #lang-btn-en"
    );
    assert!(
        html.contains("id=\"lang-btn-fr\""),
        "index.html must have #lang-btn-fr"
    );
    assert!(
        html.contains("onclick=\"setAppLanguage('en')\""),
        "index.html must have onclick setAppLanguage en"
    );
    assert!(
        html.contains("onclick=\"setAppLanguage('fr')\""),
        "index.html must have onclick setAppLanguage fr"
    );

    let js_path = repo_path("static/app.js");
    let js = std::fs::read_to_string(&js_path)
        .unwrap_or_else(|e| panic!("app.js must be readable: {e}"));

    assert!(js.contains("const I18N = {"));
    assert!(js.contains("en: {"));
    assert!(js.contains("fr: {"));
    assert!(js.contains("localStorage.getItem(\"agent_hub_lang\")"));
    assert!(js.contains("window.setAppLanguage"));
}

#[test]
fn openapi_spec_exists_and_covers_endpoints() {
    let api_yml_path = repo_path("api.yml");
    let content = std::fs::read_to_string(&api_yml_path)
        .unwrap_or_else(|e| panic!("api.yml must be readable: {e}"));

    assert!(content.contains("openapi: 3.1.0"));
    assert!(content.contains("title: Simply Agent Hub API"));
    assert!(content.contains("/api/instances:"));
    assert!(content.contains("/ws/instances/{id}/terminal:"));
    assert!(content.contains("/api/roles/presets:"));
    assert!(content.contains("/api/orchestration/channels:"));
    assert!(content.contains("/api/orchestration/channels/{id}/messages:"));
    assert!(content.contains("/api/orchestration/channels/{id}/prune:"));
    assert!(content.contains("/api/orchestration/channels/{id}/export:"));
    assert!(content.contains("/api/git/status:"));
    assert!(content.contains("/api/git/commit:"));
    assert!(content.contains("/api/git/revert:"));
}
