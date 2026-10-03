pub mod highlighter;
pub mod languages;

pub fn expand_selection(
    tree: &tree_sitter::Tree,
    start_byte: usize,
    end_byte: usize,
) -> Option<std::ops::Range<usize>> {
    let node = tree.root_node().named_descendant_for_byte_range(start_byte, end_byte)?;
    let parent = node.parent()?;
    Some(parent.start_byte()..parent.end_byte())
}
