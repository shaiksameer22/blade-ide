use lsp_types::SemanticTokenType;
fn main() {
    let t = SemanticTokenType::new("mutable");
    println!("{:?}", t.as_str());
}
