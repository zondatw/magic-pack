use magic_pack::contents::enums;

#[cfg(test)]
mod test_get_file_type_string {
    use super::*;

    #[test]
    fn zip() {
        assert_eq!("zip", enums::get_file_type_string(enums::FileType::Zip));
    }

    #[test]
    fn tar() {
        assert_eq!("tar", enums::get_file_type_string(enums::FileType::Tar));
    }

    #[test]
    fn bz2() {
        assert_eq!("bz2", enums::get_file_type_string(enums::FileType::Bz2));
    }

    #[test]
    fn gz() {
        assert_eq!("gz", enums::get_file_type_string(enums::FileType::Gz));
    }

    #[test]
    fn tarbz2() {
        assert_eq!("tar.bz2", enums::get_file_type_string(enums::FileType::Tarbz2));
    }

    #[test]
    fn targz() {
        assert_eq!("tar.gz", enums::get_file_type_string(enums::FileType::Targz));
    }
}
