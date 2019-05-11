use std::fmt::{self, Debug, Formatter};

use crate::{
    difficulty_bytes_as_u128, u128_bytes, u32_bytes, u64_bytes, BlockHash, Hashable, Transaction,
};

/**
 * Blocks contain this information (7 basic attributes):
 *
 * - Index: This block's location within the list of blocks.
 *
 * - Transactions: Any relevant information of events that have occurred for/in
 *   the block.
 *
 * - Timestamp: Gives our blockchain a sense of time.
 *
 * - Nonce: A special number used for mining (for proof-of-work [PoW]
 *   verification).
 *
 * - Previous block hash: A cryptographic fingerprint of the previous block.
 *
 * - Hash: A cryptographic fingerprint of all the above data concatenated
 *   together.
 *
 * - Difficulty: A measure of how difficult it is to find a hash below a given
 *   target.
 *
 * Difficulty
 * ----------
 *
 * SHA-256 generates a 32-byte hash. Difficulty (in our case) specifies the
 * unsigned 128-bit integer value that the most significant 16 bytes of the hash
 * of a block must be less than before it is considered "valid" (if those bytes
 * are interpreted as a single number instead of a series of bytes). Difficulty
 * will be stored as a field of the Block struct.
 *
 * Difficulty could also be expressed as:
 *
 * - The first n bytes of the hash that must be zero.
 * - The number of bits or bytes at the beginning of the hash that must be zero.
 *
 * These options are essentially different ways of expressing the same thing.
 *
 * Bitcoin stores the difficulty value more compactly than this, but this is
 * simpler and we don't have to worry about space efficiency.
 *
 * Little vs Big Endian
 * --------------------
 *
 * Endianness: Order of bytes stored in memory.
 *
 * Example: 42_u32
 *
 * Hex Representation                          | 0x0000002a
 * ------------------------------------------------------------------
 * Stored in big-endian order                  | 00 00 00 2a
 * Stored in little-endian order (most-common) | 2a 00 00 00
 *
 * If we treat it like a little-endian representation of a number, the most
 * significant 16 bytes of our hash will appear at the end of our hash's 32-byte
 * vector.
 *
 * See: https://crates.io/crates/byteorder
 *
 * Nonce
 * -----
 *
 * A hash is a unique, reproducible fingerprint for some data. Therefore, to
 * make a "valid" hash (per difficulty), we must somehow change the bytes we
 * send to the function (the pre-image). Remember that even one small change to
 * the input changes the resultant hash drastically. This effect is commonly
 * called avalanching.
 *
 * Of course, we can't actually change the information stored in a block
 * willy-nilly. Therefore, we introduce an additional piece of data called a
 * nonce: an arbitrary (but not necessarily random) value added as a field to
 * each block, and hashed along with the data. Since it has been declared
 * arbitrary, we can change it as we please.
 *
 * You can think of it like this: generating the correct hash for a block is
 * like the puzzle, and the nonce is the key to that puzzle. The process of
 * finding that key is called mining.
 */
#[derive(PartialEq)]
pub struct Block {
    pub index: u32,
    pub timestamp: u128,
    pub hash: BlockHash,
    pub previous_block_hash: BlockHash,
    pub nonce: u64,
    pub transactions: Vec<Transaction>,
    pub difficulty: u128,
}

impl Debug for Block {
    /**
     * Returns a formatted result of the block.
     */
    fn fmt(&self, buffer: &mut Formatter) -> fmt::Result {
        write!(
            buffer,
            "Block[{}]: hash {}, timestamp {}, {} transaction(s), nonce {}",
            &self.index,
            &hex::encode(&self.hash),
            &self.timestamp,
            &self.transactions.len(),
            &self.nonce
        )
    }
}

impl Block {
    /**
     * Creates a block with given attributes. Initializes the hash to a
     * vector of 32 zeros.
     */
    pub fn new(
        index: u32,
        timestamp: u128,
        previous_block_hash: BlockHash,
        transactions: Vec<Transaction>,
        difficulty: u128,
    ) -> Self {
        Block {
            index,
            timestamp,
            hash: vec![0; 32],
            previous_block_hash,
            nonce: 0,
            transactions,
            difficulty,
        }
    }

    /**
     * Performs a mining algorithm:
     *
     * 1. Generate a new nonce.
     * 2. Hash bytes (this is the computationally heavy step).
     * 3. Check the hash against the difficulty.
     *
     *   a. Insufficient? Go back to step 1.
     *   b. Sufficient? Continue to step 4.
     *
     * 4. Add a block to the chain.
     * 5. Submit to peers, etc. Since this is out-of-scope for our project and we
     *    have no networking capabilities implemented yet, we'll just skip this
     *    step.
     *
     * A block having been "mined" means that an amount of effort has been put
     * into discovering a nonce "key" that "unlocks" the block's hash-based
     * "puzzle".
     *
     * Mining has the property that it is a hard problem to solve while its
     * solution is easy to check and verify.
     *
     * It has a customizable difficulty that should adapt to the amount of
     * effort being put forth by the miners on the network to maintain the
     * average time it takes to mine a block.
     *
     * Bitcoin adjusts its difficulty every 2,016 blocks such that the next
     * 2,016 blocks should take two weeks to mine.
     */
    pub fn mine(&mut self) {
        for nonce_attempt in 0..(u64::max_value()) {
            self.nonce = nonce_attempt;
            let hash = self.hash();
            if check_difficulty(&hash, self.difficulty) {
                self.hash = hash;

                return;
            }
        }
    }
}

impl Hashable for Block {
    /**
     * Returns a vector of hashable bytes that represents the block.
     */
    fn bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        bytes.extend(&u32_bytes(self.index));
        bytes.extend(&u128_bytes(self.timestamp));
        bytes.extend(&self.previous_block_hash);
        bytes.extend(&u64_bytes(self.nonce));
        bytes.extend(
            self.transactions
                .iter()
                .flat_map(Hashable::bytes)
                .collect::<Vec<u8>>(),
        );
        bytes.extend(&u128_bytes(self.difficulty));

        bytes
    }
}

/**
 * Checks whether the most significant 16 bytes of the block's hash is less than
 * the given difficulty value. If so, it's considered "valid".
 */
pub fn check_difficulty(hash: &[u8], difficulty: u128) -> bool {
    difficulty > difficulty_bytes_as_u128(&hash)
}

#[cfg(test)]
mod block_tests {
    use super::{Block, Transaction};
    use crate::transaction;

    #[test]
    fn constructor() {
        let instance = Block::new(
            1,
            2,
            vec![
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 30, 31, 32,
            ],
            vec![Transaction {
                inputs: vec![transaction::Output {
                    to_address: "Alice".to_owned(),
                    value: 1,
                }],
                outputs: vec![transaction::Output {
                    to_address: "Bob".to_owned(),
                    value: 2,
                }],
            }],
            3,
        );

        assert_eq!(1, instance.index);
        assert_eq!(2, instance.timestamp);
        assert_eq!(vec![0; 32], instance.hash);
        assert_eq!(
            vec![
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 30, 31, 32,
            ],
            instance.previous_block_hash
        );
        assert_eq!(0, instance.nonce);
        assert_eq!(
            vec![Transaction {
                inputs: vec![transaction::Output {
                    to_address: "Alice".to_owned(),
                    value: 1,
                }],
                outputs: vec![transaction::Output {
                    to_address: "Bob".to_owned(),
                    value: 2,
                }],
            }],
            instance.transactions
        );
        assert_eq!(3, instance.difficulty);
    }

    #[test]
    fn debug_format() {
        let block = Block::new(
            1,
            2,
            vec![
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 30, 31, 32,
            ],
            vec![Transaction {
                inputs: vec![transaction::Output {
                    to_address: "Alice".to_owned(),
                    value: 1,
                }],
                outputs: vec![transaction::Output {
                    to_address: "Bob".to_owned(),
                    value: 2,
                }],
            }],
            3,
        );

        let result = format!("{:?}", block);

        assert_eq!(
            "Block[1]: hash \
             0000000000000000000000000000000000000000000000000000000000000000, \
             timestamp 2, 1 transaction(s), nonce 0"
                .to_string(),
            result
        );
    }

    #[test]
    fn mine_with_difficulty_as_0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff() {
        let mut block = Block::new(
            1,
            2,
            vec![
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 30, 31, 32,
            ],
            vec![Transaction {
                inputs: vec![transaction::Output {
                    to_address: "Alice".to_owned(),
                    value: 1,
                }],
                outputs: vec![transaction::Output {
                    to_address: "Bob".to_owned(),
                    value: 2,
                }],
            }],
            0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff,
        );

        block.mine();

        assert_eq!(1, block.index);
        assert_eq!(2, block.timestamp);
        assert_eq!(
            vec![
                39, 78, 235, 119, 157, 146, 83, 4, 155, 240, 87, 117, 84, 101, 122, 41, 63, 16, 23,
                97, 216, 185, 58, 38, 132, 121, 149, 4, 136, 153, 54, 223
            ],
            block.hash
        );
        assert_eq!(
            vec![
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 30, 31, 32,
            ],
            block.previous_block_hash
        );
        assert_eq!(0, block.nonce);
        assert_eq!(
            vec![Transaction {
                inputs: vec![transaction::Output {
                    to_address: "Alice".to_owned(),
                    value: 1,
                }],
                outputs: vec![transaction::Output {
                    to_address: "Bob".to_owned(),
                    value: 2,
                }],
            }],
            block.transactions
        );
        assert_eq!(
            340282366920938463463374607431768211455_u128,
            block.difficulty
        );
    }

    #[test]
    fn mine_with_difficulty_as_0x0000_ffff_ffff_ffff_ffff_ffff_ffff_ffff() {
        let mut block = Block::new(
            1,
            2,
            vec![
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 30, 31, 32,
            ],
            vec![Transaction {
                inputs: vec![transaction::Output {
                    to_address: "Alice".to_owned(),
                    value: 1,
                }],
                outputs: vec![transaction::Output {
                    to_address: "Bob".to_owned(),
                    value: 2,
                }],
            }],
            0x0000_ffff_ffff_ffff_ffff_ffff_ffff_ffff,
        );

        block.mine();

        assert_eq!(1, block.index);
        assert_eq!(2, block.timestamp);
        assert_eq!(
            vec![
                124, 78, 251, 115, 254, 29, 54, 204, 62, 7, 162, 92, 167, 96, 106, 235, 125, 214,
                177, 227, 41, 247, 98, 147, 130, 3, 133, 225, 203, 89, 0, 0
            ],
            block.hash
        );
        assert_eq!(
            vec![
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 30, 31, 32,
            ],
            block.previous_block_hash
        );
        assert_eq!(10525, block.nonce);
        assert_eq!(
            vec![Transaction {
                inputs: vec![transaction::Output {
                    to_address: "Alice".to_owned(),
                    value: 1,
                }],
                outputs: vec![transaction::Output {
                    to_address: "Bob".to_owned(),
                    value: 2,
                }],
            }],
            block.transactions
        );
        assert_eq!(5192296858534827628530496329220095_u128, block.difficulty);
    }
}

#[cfg(test)]
mod hashable_block_tests {
    use super::{Block, Hashable, Transaction};
    use crate::transaction;

    #[test]
    fn bytes() {
        let block = Block::new(
            1,
            2,
            vec![
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 30, 31, 32,
            ],
            vec![Transaction {
                inputs: vec![transaction::Output {
                    to_address: "Alice".to_owned(),
                    value: 1,
                }],
                outputs: vec![transaction::Output {
                    to_address: "Bob".to_owned(),
                    value: 2,
                }],
            }],
            3,
        );

        let result = block.bytes();

        assert_eq!(
            vec![
                1, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8,
                9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
                30, 31, 32, 0, 0, 0, 0, 0, 0, 0, 0, 65, 108, 105, 99, 101, 1, 0, 0, 0, 0, 0, 0, 0,
                66, 111, 98, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0
            ],
            result
        );
    }

    #[test]
    fn hash() {
        let block = Block::new(
            1,
            2,
            vec![
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
                24, 25, 26, 27, 28, 29, 30, 31, 32,
            ],
            vec![Transaction {
                inputs: vec![transaction::Output {
                    to_address: "Alice".to_owned(),
                    value: 1,
                }],
                outputs: vec![transaction::Output {
                    to_address: "Bob".to_owned(),
                    value: 2,
                }],
            }],
            3,
        );

        let result = block.hash();

        assert_eq!(
            vec![
                117, 2, 120, 30, 164, 40, 67, 254, 110, 10, 42, 33, 124, 60, 170, 23, 52, 145, 230,
                21, 127, 125, 2, 199, 114, 39, 202, 78, 118, 53, 16, 204
            ],
            result
        );
    }
}

#[cfg(test)]
mod check_difficulty_tests {
    use super::{check_difficulty, BlockHash};
    use crate::difficulty_bytes_as_u128;

    #[test]
    fn difficulty_less_than_that_of_hash() {
        let data = vec![
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8,
        ];
        let data_difficulty = difficulty_bytes_as_u128(&data);
        let hash: BlockHash = vec![
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 1_u8,
        ];
        let hash_difficulty = difficulty_bytes_as_u128(&hash);
        assert!(data_difficulty < hash_difficulty);

        let result = check_difficulty(&hash, data_difficulty);

        assert_eq!(false, result);
    }

    #[test]
    fn difficulty_equal_to_that_of_hash() {
        let data = vec![
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8,
        ];
        let data_difficulty = difficulty_bytes_as_u128(&data);
        let hash: BlockHash = vec![
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8,
        ];
        let hash_difficulty = difficulty_bytes_as_u128(&hash);
        assert!(data_difficulty == hash_difficulty);

        let result = check_difficulty(&hash, data_difficulty);

        assert_eq!(false, result);
    }

    #[test]
    fn difficulty_greater_than_that_of_hash() {
        let data = vec![
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 1_u8,
        ];
        let data_difficulty = difficulty_bytes_as_u128(&data);
        let hash: BlockHash = vec![
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8,
        ];
        let hash_difficulty = difficulty_bytes_as_u128(&hash);
        assert!(data_difficulty > hash_difficulty);

        let result = check_difficulty(&hash, data_difficulty);

        assert_eq!(true, result);
    }
}
