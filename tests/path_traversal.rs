use std::path::Path;

use magic_pack::utils::is_safe_path;

#[test]
fn is_safe_path_blocks_traversal() {
    assert!(!is_safe_path(Path::new("../evil.txt")));
    assert!(!is_safe_path(Path::new("dir/../evil.txt")));
    assert!(!is_safe_path(Path::new("/absolute/path")));
}

#[test]
fn is_safe_path_allows_relative() {
    assert!(is_safe_path(Path::new("safe.txt")));
    assert!(is_safe_path(Path::new("dir/sub/file.txt")));
}
