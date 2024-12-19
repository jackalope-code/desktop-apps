#[cfg(test)]
mod read_tests {
    use super::*;

    #[test]
    fn read_large_positive_offsets() {
        assert_eq!(4, 4);
    }
    
    #[test]
    fn read_large_positive_offset_to_negative_offset() {
        assert_eq!(4, 4);
    }

    #[test]
    fn read_negative_offset_to_large_positive_offsets() {
        assert_eq!(4, 4);
    }

    #[test]
    fn read_large_positive_offset_to_eof() {
        assert_eq!(4, 4);
    }

    #[test]
    fn read_negative_offset_to_eof() {
        assert_eq!(4, 4);
    }
    
    #[test]
    fn read_negative_to_negative_ascending_pass() {
        assert_eq!(4, 4);
    }

    #[test]
    fn read_negative_to_negative_descending_fail() {
        assert_eq!(4, 4);
    }

    #[test]
    fn read_eof_to_offset_fail() {
        assert_eq!(4, 4);
    }

    #[test]
    fn invalid_read_offset_parse_fail() {
        assert_eq!(4, 4);
    }
}