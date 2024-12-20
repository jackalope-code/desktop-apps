#[cfg(test)]
mod file_id_tests {
    use super::*;

    #[test]
    fn id_jpg() {
        assert_eq!(4, 4);
    }

    #[test]
    fn id_bmp() {
        assert_eq!(4, 4);
    }

    #[test]
    fn id_gif() {
        assert_eq!(4, 4);
    }

    #[test]
    fn id_png() {
        assert_eq!(4, 4);
    }
    
    #[test]
    fn id_exe() {
        assert_eq!(4, 4);
    }
    
    #[test]
    fn id_class() {
        // cargo run type test/NoOp.class
        assert_eq!(4, 4);
    }
}