#[cfg(all(unix, not(target_os = "macos")))]
mod unix_non_utf8 {
    use std::fs;
    use std::path::PathBuf;
    use std::os::unix::ffi::OsStringExt;

    use magic_pack::contents::enums::FileType;
    use magic_pack::modules;

    #[test]
    fn non_utf8_path_gz_roundtrip() {
        let root = PathBuf::from("target/tests/path_interface");
        fs::create_dir_all(&root).expect("create test dir");

        let mut name_bytes = b"file_\x80.txt".to_vec();
        let src_name = std::ffi::OsString::from_vec(name_bytes.split_off(0));
        let src = root.join(src_name);
        fs::write(&src, b"non-utf8").expect("write src");

        let archive = root.join("non_utf8.gz");
        modules::compress(FileType::Gz, &src, &archive);

        let output = root.join("out.txt");
        modules::decompress(FileType::Gz, &archive, &output);

        let contents = fs::read_to_string(&output).expect("read output");
        assert_eq!(contents, "non-utf8");
    }
}

#[cfg(any(not(unix), target_os = "macos"))]
mod non_utf8_skip {
    #[test]
    fn non_utf8_path_gz_roundtrip() {
        // macOS filesystems reject non-UTF8 byte sequences in filenames.
        // Non-UTF8 filenames are otherwise a Unix-specific behavior.
        assert!(true);
    }
}
