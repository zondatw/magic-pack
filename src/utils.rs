pub fn is_safe_path(path: &std::path::Path) -> bool {
    use std::path::Component;
    !path.components().any(|c| {
        matches!(
            c,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    })
}
