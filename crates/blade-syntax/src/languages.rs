use std::path::Path;

#[derive(Debug, Clone)]
pub struct LanguageConfig {
    pub name: &'static str,
    pub extensions: &'static [&'static str],
    pub highlight_query: &'static str,
}

/// Detect language from file extension
pub fn detect_language(path: &Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?;
    match ext {
        "rs" => Some("rust"),
        "py" | "pyw" => Some("python"),
        "js" | "mjs" | "cjs" => Some("javascript"),
        "ts" | "mts" | "cts" => Some("typescript"),
        "tsx" => Some("tsx"),
        "jsx" => Some("jsx"),
        "c" | "h" => Some("c"),
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => Some("cpp"),
        "go" => Some("go"),
        "json" => Some("json"),
        "html" | "htm" => Some("html"),
        "css" => Some("css"),
        "sh" | "bash" | "zsh" => Some("bash"),
        "yaml" | "yml" => Some("yaml"),
        "xml" => Some("xml"),
        "java" => Some("java"),
        "rb" => Some("ruby"),
        "php" => Some("php"),
        "swift" => Some("swift"),
        "kt" | "kts" => Some("kotlin"),
        "zig" => Some("zig"),
        "lua" => Some("lua"),
        "sql" => Some("sql"),
        "dockerfile" | "Dockerfile" => Some("dockerfile"),
        _ => None,
    }
}

pub fn get_language(lang_name: &str) -> Option<(tree_sitter::Language, &'static str)> {
    match lang_name {
        "rust" => Some((tree_sitter_rust::LANGUAGE.into(), tree_sitter_rust::HIGHLIGHTS_QUERY)),
        "python" => Some((tree_sitter_python::LANGUAGE.into(), tree_sitter_python::HIGHLIGHTS_QUERY)),
        "javascript" => Some((tree_sitter_javascript::LANGUAGE.into(), tree_sitter_javascript::HIGHLIGHT_QUERY)),
        "typescript" => Some((tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(), tree_sitter_typescript::HIGHLIGHTS_QUERY)),
        "c" => Some((tree_sitter_c::LANGUAGE.into(), tree_sitter_c::HIGHLIGHT_QUERY)),
        "cpp" => Some((tree_sitter_cpp::LANGUAGE.into(), tree_sitter_cpp::HIGHLIGHT_QUERY)),
        "go" => Some((tree_sitter_go::LANGUAGE.into(), tree_sitter_go::HIGHLIGHTS_QUERY)),
        "json" => Some((tree_sitter_json::LANGUAGE.into(), tree_sitter_json::HIGHLIGHTS_QUERY)),
        "html" => Some((tree_sitter_html::LANGUAGE.into(), tree_sitter_html::HIGHLIGHTS_QUERY)),
        "css" => Some((tree_sitter_css::LANGUAGE.into(), tree_sitter_css::HIGHLIGHTS_QUERY)),
        "bash" => Some((tree_sitter_bash::LANGUAGE.into(), tree_sitter_bash::HIGHLIGHT_QUERY)),
        _ => None,
    }
}
