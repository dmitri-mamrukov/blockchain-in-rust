pub trait Hashable {
    /**
     * Returns a vector of hashable bytes that represents the hashable instance.
     */
    fn bytes(&self) -> Vec<u8>;

    /**
     * Returns a vector of bytes that represents the hashable instance's hash.
     */
    fn hash(&self) -> Vec<u8> {
        crypto_hash::digest(crypto_hash::Algorithm::SHA256, &self.bytes())
    }
}

#[cfg(test)]
mod hashable_block_tests {
    use super::Hashable;

    struct DummyHashableStruct {}

    impl Hashable for DummyHashableStruct {
        fn bytes(&self) -> Vec<u8> {
            vec![1, 2, 3, 4]
        }
    }

    #[test]
    fn hash() {
        let hashable = DummyHashableStruct {};
        assert_eq!(vec![1, 2, 3, 4], hashable.bytes());

        let result = hashable.hash();

        assert_eq!(32, result.len());
        assert_eq!(
            vec![
                159, 100, 167, 71, 225, 185, 127, 19, 31, 171, 182, 180, 71, 41, 108, 155, 111, 2,
                1, 231, 159, 179, 197, 53, 110, 108, 119, 232, 155, 106, 128, 106
            ],
            result
        );
    }
}
