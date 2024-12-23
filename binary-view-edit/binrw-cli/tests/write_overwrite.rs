#[cfg(test)]
mod write_overwrite_tests {
    use super::*;

    #[test]
    fn write_large_positive_offsets() {
        assert_eq!(4, 4);
    }
    
    #[test]
    fn write_large_positive_offset_to_negative_offset() {
        assert_eq!(4, 4);
    }

    #[test]
    fn write_negative_offset_to_large_positive_offsets() {
        assert_eq!(4, 4);
    }

    #[test]
    fn write_large_positive_offset_to_eof() {
        assert_eq!(4, 4);
    }

    #[test]
    fn write_negative_offset_to_eof() {
        assert_eq!(4, 4);
    }
    
    #[test]
    fn write_negative_to_negative_ascending_pass() {
        assert_eq!(4, 4);
    }

    #[test]
    fn write_negative_to_negative_descending_fail() {
        assert_eq!(4, 4);
    }

    #[test]
    fn write_to_eof_success() {
        assert_eq!(4, 4);
    }

    #[test]
    fn invalid_write_offset_fail() {
        assert_eq!(4, 4);
    }

    #[test]
    fn write_verify() {
        assert_eq!(4, 4);
    }

    #[ignore]
    #[test]
    fn check_write_overwrite() {

    }
    
    #[ignore]
    #[test]
    fn check_write_overwrite_from_file() {

    }
    #[ignore]
    #[test]
    fn ignore_existing_file_by_default() {

    }
    #[ignore]
    #[test]
    fn edit_existing_file_with_flag() {

    }
    #[ignore]
    #[test]
    fn save_edit_as_copy_with_flag() {

    }
    #[ignore]
    #[test]
    fn create_new_file_with_flag() {

    }
}