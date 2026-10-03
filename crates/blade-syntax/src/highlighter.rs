use tree_sitter::{Parser, Tree, Language, Query, QueryCursor};
use ropey::Rope;
use streaming_iterator::StreamingIterator;

/// Syntax highlight token with style information
#[derive(Debug, Clone)]
pub struct HighlightToken {
    pub start_byte: usize,
    pub end_byte: usize,
    pub highlight_type: HighlightType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HighlightType {
    Keyword,
    String,
    Comment,
    Function,
    Type,
    Variable,
    Number,
    Operator,
    Punctuation,
    Property,
    Constant,
    Builtin,
    Tag,
    Attribute,
    Namespace,
    Label,
    Error,
    None,
}

/// Manages tree-sitter parsing and syntax highlighting for a buffer
pub struct SyntaxHighlighter {
    pub parser: Parser,
    pub tree: Option<Tree>,
    pub highlight_query: Option<Query>,
    pub language_name: String,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
            tree: None,
            highlight_query: None,
            language_name: String::new(),
        }
    }

    /// Set the language for this highlighter
    pub fn set_language(&mut self, lang: Language, name: &str, query_source: &str)
        -> Result<(), tree_sitter::LanguageError>
    {
        self.parser.set_language(&lang)?;
        self.highlight_query = Query::new(&lang, query_source).ok();
        self.language_name = name.to_string();
        self.tree = None;
        Ok(())
    }

    /// Parse or incrementally re-parse the buffer
    pub fn parse(&mut self, rope: &Rope, huge_file: bool) {
        if huge_file {
            self.tree = None;
            return;
        }
        let text = rope.to_string(); // TODO: Use rope callback for zero-copy
        self.tree = self.parser.parse(&text, self.tree.as_ref());
    }

    /// Get highlight tokens for a given byte range (viewport)
    pub fn highlights(&self, start_byte: usize, end_byte: usize, rope: &Rope) -> Vec<HighlightToken> {
        let mut tokens = Vec::new();

        let (tree, query) = match (&self.tree, &self.highlight_query) {
            (Some(t), Some(q)) => (t, q),
            _ => return tokens,
        };

        let mut cursor = QueryCursor::new();
        cursor.set_byte_range(start_byte..end_byte);

        // Convert rope to string for text provider.
        // For production, we should implement the TextProvider trait for Rope to avoid allocations
        let text_slice = rope.byte_slice(..).to_string();
        let text_bytes = text_slice.as_bytes();

        let mut matches = cursor.matches(query, tree.root_node(), text_bytes);

        while let Some(m) = matches.next() {
            for cap in m.captures {
                let node = cap.node;
                let capture_name_ref: &str = query.capture_names()[cap.index as usize];
                
                let highlight_type = match capture_name_ref {
                    "keyword" | "conditional" | "repeat" => HighlightType::Keyword,
                    "string" | "string.special" => HighlightType::String,
                    "comment" => HighlightType::Comment,
                    "function" | "function.macro" | "function.call" | "method" => HighlightType::Function,
                    "type" | "type.builtin" => HighlightType::Type,
                    "variable" | "variable.parameter" => HighlightType::Variable,
                    "number" | "float" => HighlightType::Number,
                    "operator" => HighlightType::Operator,
                    "punctuation.delimiter" | "punctuation.bracket" => HighlightType::Punctuation,
                    "property" => HighlightType::Property,
                    "constant" | "constant.builtin" => HighlightType::Constant,
                    "tag" | "tag.attribute" => HighlightType::Tag,
                    "label" => HighlightType::Label,
                    "namespace" => HighlightType::Namespace,
                    _ => HighlightType::None,
                };

                if highlight_type != HighlightType::None {
                    tokens.push(HighlightToken {
                        start_byte: node.start_byte(),
                        end_byte: node.end_byte(),
                        highlight_type,
                    });
                }
            }
        }
        
        // Sort tokens by start_byte, this helps when applying them linearly
        tokens.sort_by_key(|t| t.start_byte);
        tokens
    }

    pub fn language_name(&self) -> &str { &self.language_name }
    pub fn has_tree(&self) -> bool { self.tree.is_some() }
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}
