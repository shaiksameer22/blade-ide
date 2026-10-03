use lsp_types::Uri;
use std::str::FromStr;

fn main() {
    let u = Uri::from_str("file:///foo/bar");
}
