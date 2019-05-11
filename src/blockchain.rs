use std::collections::HashSet;

use crate::{check_difficulty, Block, BlockHash, Hashable};

#[derive(Debug, PartialEq)]
pub enum BlockValidationErr {
    MismatchedIndex,
    InvalidHash,
    AchronologicalTimestamp,
    MismatchedPreviousHash,
    InvalidGenesisBlockFormat,
    InvalidInput,
    InsufficientInputValue,
    InvalidCoinbaseTransaction,
    FeeExceedsCoinbaseTransactionOutputValue,
}

/**
 * A blockchain is just a block vector, which acts as a distributed ledger.
 */
#[derive(Default)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    unspent_outputs: HashSet<BlockHash>,
}

impl Blockchain {
    pub fn new() -> Self {
        Blockchain {
            blocks: vec![],
            unspent_outputs: HashSet::new(),
        }
    }

    /**
     * Block Verification
     * ------------------
     *
     * Each supposed valid block has a nonce attached to it that we assume took
     * an approximately certain amount of effort to generate. This
     * "approximately certain amount of effort" is described by the difficulty
     * value.
     *
     * We will verify four things now:
     *
     * 1. Actual index == stored index value (note that Bitcoin blocks don't
     *    store their index).
     *
     * 2. Block's hash fits stored difficulty value (we'll just trust the
     *    difficulty for now) (insecure).
     *
     * 3. Time is always increasing (in real life [IRL] network latency/sync
     *    demands leniency here).
     *
     * 4. Actual previous block's hash == stored previous_block_hash value
     *    (except for the genesis block).
     *
     * Security Notes
     * --------------
     *
     * This is not secure! There are some things to take into account:
     *
     * - The difficulty stored in a block is not validated.
     *
     * - The value of the coinbase transaction is not validated.
     *
     * - "Coin ownership" is neither enforced nor existent.
     *
     * - Two otherwise identical outputs from different transactions are
     *   indistinguishable.
     */
    pub fn update_with_block(&mut self, block: Block) -> Result<(), BlockValidationErr> {
        let index = self.blocks.len();

        if block.index != index as u32 {
            return Err(BlockValidationErr::MismatchedIndex);
        } else if !check_difficulty(&block.hash(), block.difficulty) {
            return Err(BlockValidationErr::InvalidHash);
        } else if self.is_genesis_block(index) {
            if block.previous_block_hash != vec![0; 32] {
                return Err(BlockValidationErr::InvalidGenesisBlockFormat);
            }
        } else {
            let previous_block = &self.blocks[index - 1];
            if block.timestamp <= previous_block.timestamp {
                return Err(BlockValidationErr::AchronologicalTimestamp);
            } else if block.previous_block_hash != previous_block.hash {
                return Err(BlockValidationErr::MismatchedPreviousHash);
            }
        }

        if let Some((coinbase, transactions)) = block.transactions.split_first() {
            if !coinbase.is_coinbase() {
                return Err(BlockValidationErr::InvalidCoinbaseTransaction);
            }

            let mut block_spent: HashSet<BlockHash> = HashSet::new();
            let mut block_created: HashSet<BlockHash> = HashSet::new();
            let mut total_fee = 0;

            for transaction in transactions {
                let input_hashes = transaction.input_hashes();
                if !(&input_hashes - &self.unspent_outputs).is_empty() {
                    return Err(BlockValidationErr::InvalidInput);
                }

                let input_value = transaction.input_value();
                let output_value = transaction.output_value();
                if output_value > input_value {
                    return Err(BlockValidationErr::InsufficientInputValue);
                }

                let fee = input_value - output_value;
                total_fee += fee;

                block_spent.extend(input_hashes);
                block_created.extend(transaction.output_hashes());
            }

            if coinbase.output_value() < total_fee {
                return Err(BlockValidationErr::FeeExceedsCoinbaseTransactionOutputValue);
            } else {
                block_created.extend(coinbase.output_hashes());
            }

            self.unspent_outputs
                .retain(|output| !block_spent.contains(output));
            self.unspent_outputs.extend(block_created);
        }

        self.blocks.push(block);

        Ok(())
    }

    fn is_genesis_block(&self, index: usize) -> bool {
        index == 0
    }
}

#[cfg(test)]
mod blockchain_constructor_tests {
    use std::collections::HashSet;

    use super::{Block, BlockHash, Blockchain};

    fn assert_default_constructor(instance: Blockchain) {
        assert_eq!(Vec::<Block>::new(), instance.blocks);
        assert_eq!(HashSet::<BlockHash>::new(), instance.unspent_outputs);
    }

    #[test]
    fn constructor_with_new() {
        let instance = Blockchain::new();

        assert_default_constructor(instance);
    }

    #[test]
    fn constructor_with_default() {
        let instance: Blockchain = Default::default();

        assert_default_constructor(instance);
    }
}

#[cfg(test)]
mod blockchain_update_with_block_tests {
    use crate::transaction::Output;
    use crate::{now, Transaction};

    use super::{check_difficulty, Block, BlockHash, BlockValidationErr, Blockchain, Hashable};

    const IMPOSSIBLE_DIFFICULTY: u128 = 0x0000_0000_0000_0000_0000_0000_0000_0000;
    const DIFFICULTY: u128 = 0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ffff;

    struct BlockOutputConfig {
        unspent_output_value: u64,
        output_value: u64,
        expected_difference: u64,
    }

    fn genesis_block_hash() -> BlockHash {
        vec![0; 32]
    }

    fn current_time() -> u128 {
        now().expect("Failure to get the current time in milliseconds.")
    }

    fn create_coinbase_transaction() -> Transaction {
        Transaction {
            inputs: vec![],
            outputs: vec![],
        }
    }

    fn create_block_with_impossible_difficulty(
        index: u32,
        timestamp: u128,
        previous_block_hash: BlockHash,
        transactions: Vec<Transaction>,
    ) -> Block {
        let block = Block::new(
            index,
            timestamp,
            previous_block_hash,
            transactions,
            IMPOSSIBLE_DIFFICULTY,
        );
        assert_eq!(false, check_difficulty(&block.hash(), block.difficulty));

        block
    }

    fn create_block_with_valid_difficulty(
        index: u32,
        timestamp: u128,
        previous_block_hash: BlockHash,
        transactions: Vec<Transaction>,
    ) -> Block {
        let block = Block::new(
            index,
            timestamp,
            previous_block_hash,
            transactions,
            DIFFICULTY,
        );
        assert_eq!(true, check_difficulty(&block.hash(), block.difficulty));

        block
    }

    fn add_block_to_blockchain(blockchain: &mut Blockchain, block: Block) {
        let original_length = blockchain.blocks.len();

        let result = blockchain.update_with_block(block);

        assert_eq!(true, result.is_ok());
        assert_eq!(Ok(()), result);
        assert_eq!(original_length + 1, blockchain.blocks.len());
    }

    fn assert_add_block_with_sufficient_inputs(config: BlockOutputConfig) {
        let timestamp = current_time();
        let genesis_block = create_block_with_valid_difficulty(
            0,
            timestamp,
            genesis_block_hash(),
            vec![Transaction {
                inputs: vec![],
                outputs: vec![
                    Output {
                        to_address: "Alice".to_string(),
                        value: 1,
                    },
                    Output {
                        to_address: "Bob".to_string(),
                        value: 2,
                    },
                ],
            }],
        );
        let mut blockchain = Blockchain::new();
        let block = create_block_with_valid_difficulty(
            1,
            timestamp + 1,
            genesis_block.hash.clone(),
            vec![
                Transaction {
                    inputs: vec![],
                    outputs: vec![Output {
                        to_address: "Chris".to_owned(),
                        value: config.unspent_output_value,
                    }],
                },
                Transaction {
                    inputs: vec![
                        Output {
                            to_address: "Alice".to_owned(),
                            value: 1,
                        },
                        Output {
                            to_address: "Bob".to_owned(),
                            value: 2,
                        },
                    ],
                    outputs: vec![Output {
                        to_address: "Chris".to_owned(),
                        value: config.output_value,
                    }],
                },
            ],
        );
        assert!(block.transactions[1].input_value() >= block.transactions[1].output_value());
        assert_eq!(
            block.transactions[1].input_value(),
            block.transactions[1].output_value() + config.expected_difference
        );
        add_block_to_blockchain(&mut blockchain, genesis_block);
        add_block_to_blockchain(&mut blockchain, block);
    }

    #[test]
    fn add_block_with_invalid_previous_block_hash() {
        let wrong_block_hash = vec![];
        let genesis_block =
            create_block_with_valid_difficulty(0, current_time(), wrong_block_hash, vec![]);
        let mut blockchain = Blockchain::new();

        let result = blockchain.update_with_block(genesis_block);

        assert_eq!(true, result.is_err());
        assert_eq!(Err(BlockValidationErr::InvalidGenesisBlockFormat), result);
    }

    #[test]
    fn add_block_with_index_as_one_to_empty_blockchain() {
        let wrong_index = 1;
        let genesis_block = Block::new(wrong_index, 2, vec![], vec![], 3);
        let mut blockchain = Blockchain::new();

        let result = blockchain.update_with_block(genesis_block);

        assert_eq!(true, result.is_err());
        assert_eq!(Err(BlockValidationErr::MismatchedIndex), result);
    }

    #[test]
    fn add_block_with_index_as_zero_to_one_block_blockchain() {
        let timestamp = current_time();
        let genesis_block =
            create_block_with_valid_difficulty(0, timestamp, genesis_block_hash(), vec![]);
        let wrong_index = 0;
        let block = create_block_with_valid_difficulty(
            wrong_index,
            timestamp + 1,
            genesis_block.hash.clone(),
            vec![],
        );
        let mut blockchain = Blockchain::new();
        add_block_to_blockchain(&mut blockchain, genesis_block);

        let result = blockchain.update_with_block(block);

        assert_eq!(true, result.is_err());
        assert_eq!(Err(BlockValidationErr::MismatchedIndex), result);
    }

    #[test]
    fn add_block_with_invalid_hash() {
        let timestamp = current_time();
        let genesis_block =
            create_block_with_valid_difficulty(0, timestamp, genesis_block_hash(), vec![]);
        let block = create_block_with_impossible_difficulty(
            1,
            timestamp + 1,
            genesis_block.hash.clone(),
            vec![],
        );
        let mut blockchain = Blockchain::new();
        add_block_to_blockchain(&mut blockchain, genesis_block);

        let result = blockchain.update_with_block(block);

        assert_eq!(true, result.is_err());
        assert_eq!(Err(BlockValidationErr::InvalidHash), result);
    }

    #[test]
    fn add_block_with_timestamp_earlier_than_previous_timestamp() {
        let timestamp = current_time();
        let genesis_block =
            create_block_with_valid_difficulty(0, timestamp, genesis_block_hash(), vec![]);
        let wrong_timestamp = timestamp - 1;
        let block = create_block_with_valid_difficulty(
            1,
            wrong_timestamp,
            genesis_block.hash.clone(),
            vec![],
        );
        let mut blockchain = Blockchain::new();
        add_block_to_blockchain(&mut blockchain, genesis_block);

        let result = blockchain.update_with_block(block);

        assert_eq!(true, result.is_err());
        assert_eq!(Err(BlockValidationErr::AchronologicalTimestamp), result);
    }

    #[test]
    fn add_block_with_timestamp_equal_to_previous_timestamp() {
        let timestamp = current_time();
        let genesis_block =
            create_block_with_valid_difficulty(0, timestamp, genesis_block_hash(), vec![]);
        let wrong_timestamp = timestamp - 1;
        let block = create_block_with_valid_difficulty(
            1,
            wrong_timestamp,
            genesis_block.hash.clone(),
            vec![],
        );
        let mut blockchain = Blockchain::new();
        add_block_to_blockchain(&mut blockchain, genesis_block);

        let result = blockchain.update_with_block(block);

        assert_eq!(true, result.is_err());
        assert_eq!(Err(BlockValidationErr::AchronologicalTimestamp), result);
    }

    #[test]
    fn add_block_with_mismatched_previous_hash() {
        let timestamp = current_time();
        let genesis_block =
            create_block_with_valid_difficulty(0, timestamp, genesis_block_hash(), vec![]);
        let wrong_previous_hash = vec![1, 2, 3];
        let block =
            create_block_with_valid_difficulty(1, timestamp + 1, wrong_previous_hash, vec![]);
        let mut blockchain = Blockchain::new();
        add_block_to_blockchain(&mut blockchain, genesis_block);

        let result = blockchain.update_with_block(block);

        assert_eq!(true, result.is_err());
        assert_eq!(Err(BlockValidationErr::MismatchedPreviousHash), result);
    }

    #[test]
    fn add_block_with_transaction_that_has_non_empty_inputs() {
        let timestamp = current_time();
        let wrong_inputs = vec![Output {
            to_address: "Alice".to_string(),
            value: 1,
        }];
        let genesis_block = create_block_with_valid_difficulty(
            0,
            timestamp,
            genesis_block_hash(),
            vec![Transaction {
                inputs: wrong_inputs,
                outputs: vec![],
            }],
        );
        let mut blockchain = Blockchain::new();

        let result = blockchain.update_with_block(genesis_block);

        assert_eq!(true, result.is_err());
        assert_eq!(Err(BlockValidationErr::InvalidCoinbaseTransaction), result);
    }

    #[test]
    fn add_block_with_transactions_where_first_one_has_non_empty_inputs_case1() {
        let timestamp = current_time();
        let wrong_inputs = vec![Output {
            to_address: "Alice".to_string(),
            value: 1,
        }];
        let genesis_block = create_block_with_valid_difficulty(
            0,
            timestamp,
            genesis_block_hash(),
            vec![
                Transaction {
                    inputs: wrong_inputs,
                    outputs: vec![],
                },
                Transaction {
                    inputs: vec![],
                    outputs: vec![],
                },
                Transaction {
                    inputs: vec![],
                    outputs: vec![],
                },
            ],
        );
        let mut blockchain = Blockchain::new();

        let result = blockchain.update_with_block(genesis_block);

        assert_eq!(true, result.is_err());
        assert_eq!(Err(BlockValidationErr::InvalidCoinbaseTransaction), result);
    }

    #[test]
    fn add_block_with_transactions_where_first_one_has_non_empty_inputs_case2() {
        let timestamp = current_time();
        let wrong_inputs = vec![Output {
            to_address: "Alice".to_string(),
            value: 1,
        }];
        let genesis_block = create_block_with_valid_difficulty(
            0,
            timestamp,
            genesis_block_hash(),
            vec![
                create_coinbase_transaction(),
                Transaction {
                    inputs: vec![],
                    outputs: vec![],
                },
                Transaction {
                    inputs: vec![],
                    outputs: vec![],
                },
            ],
        );
        let block = create_block_with_valid_difficulty(
            1,
            timestamp + 1,
            genesis_block.hash.clone(),
            vec![
                Transaction {
                    inputs: wrong_inputs,
                    outputs: vec![],
                },
                Transaction {
                    inputs: vec![],
                    outputs: vec![],
                },
                Transaction {
                    inputs: vec![],
                    outputs: vec![],
                },
            ],
        );
        let mut blockchain = Blockchain::new();
        add_block_to_blockchain(&mut blockchain, genesis_block);

        let result = blockchain.update_with_block(block);

        assert_eq!(true, result.is_err());
        assert_eq!(Err(BlockValidationErr::InvalidCoinbaseTransaction), result);
    }

    #[test]
    fn add_block_with_outputs_less_than_fee_case1() {
        let timestamp = current_time();
        let genesis_block = create_block_with_valid_difficulty(
            0,
            timestamp,
            genesis_block_hash(),
            vec![Transaction {
                inputs: vec![],
                outputs: vec![Output {
                    to_address: "Alice".to_string(),
                    value: 1,
                }],
            }],
        );
        let mut blockchain = Blockchain::new();
        let block = create_block_with_valid_difficulty(
            1,
            timestamp + 1,
            genesis_block.hash.clone(),
            vec![
                create_coinbase_transaction(),
                Transaction {
                    inputs: vec![Output {
                        to_address: "Alice".to_owned(),
                        value: 1,
                    }],
                    outputs: vec![],
                },
            ],
        );
        add_block_to_blockchain(&mut blockchain, genesis_block);

        let result = blockchain.update_with_block(block);

        assert_eq!(true, result.is_err());
        assert_eq!(
            Err(BlockValidationErr::FeeExceedsCoinbaseTransactionOutputValue),
            result
        );
    }

    #[test]
    fn add_block_with_outputs_less_than_fee_case2() {
        let timestamp = current_time();
        let genesis_block = create_block_with_valid_difficulty(
            0,
            timestamp,
            genesis_block_hash(),
            vec![Transaction {
                inputs: vec![],
                outputs: vec![Output {
                    to_address: "Alice".to_string(),
                    value: 1,
                }],
            }],
        );
        let mut blockchain = Blockchain::new();
        let mut coinbase_transaction = create_coinbase_transaction();
        coinbase_transaction.outputs = vec![Output {
            to_address: "Chris".to_owned(),
            value: 0,
        }];
        let block = create_block_with_valid_difficulty(
            1,
            timestamp + 1,
            genesis_block.hash.clone(),
            vec![
                coinbase_transaction,
                Transaction {
                    inputs: vec![Output {
                        to_address: "Alice".to_owned(),
                        value: 1,
                    }],
                    outputs: vec![],
                },
            ],
        );
        add_block_to_blockchain(&mut blockchain, genesis_block);

        let result = blockchain.update_with_block(block);

        assert_eq!(true, result.is_err());
        assert_eq!(
            Err(BlockValidationErr::FeeExceedsCoinbaseTransactionOutputValue),
            result
        );
    }

    #[test]
    fn add_block_with_second_transaction_that_has_non_empty_inputs() {
        let timestamp = current_time();
        let genesis_block = create_block_with_valid_difficulty(
            0,
            timestamp,
            genesis_block_hash(),
            vec![
                create_coinbase_transaction(),
                Transaction {
                    inputs: vec![Output {
                        to_address: "Alice".to_string(),
                        value: 1,
                    }],
                    outputs: vec![],
                },
            ],
        );
        let mut blockchain = Blockchain::new();

        let result = blockchain.update_with_block(genesis_block);

        assert_eq!(true, result.is_err());
        assert_eq!(Err(BlockValidationErr::InvalidInput), result);
    }

    #[test]
    fn add_block_with_insufficient_inputs_case1() {
        let timestamp = current_time();
        let genesis_block = create_block_with_valid_difficulty(
            0,
            timestamp,
            genesis_block_hash(),
            vec![
                create_coinbase_transaction(),
                Transaction {
                    inputs: vec![],
                    outputs: vec![Output {
                        to_address: "Alice".to_string(),
                        value: 1,
                    }],
                },
            ],
        );
        let mut blockchain = Blockchain::new();

        let result = blockchain.update_with_block(genesis_block);

        assert_eq!(true, result.is_err());
        assert_eq!(Err(BlockValidationErr::InsufficientInputValue), result);
    }

    #[test]
    fn add_block_with_insufficient_inputs_case2() {
        let timestamp = current_time();
        let genesis_block = create_block_with_valid_difficulty(
            0,
            timestamp,
            genesis_block_hash(),
            vec![Transaction {
                inputs: vec![],
                outputs: vec![
                    Output {
                        to_address: "Alice".to_string(),
                        value: 1,
                    },
                    Output {
                        to_address: "Bob".to_string(),
                        value: 2,
                    },
                ],
            }],
        );
        let mut blockchain = Blockchain::new();
        let block = create_block_with_valid_difficulty(
            1,
            timestamp + 1,
            genesis_block.hash.clone(),
            vec![
                Transaction {
                    inputs: vec![],
                    outputs: vec![Output {
                        to_address: "Chris".to_owned(),
                        value: 4,
                    }],
                },
                Transaction {
                    inputs: vec![
                        Output {
                            to_address: "Alice".to_owned(),
                            value: 1,
                        },
                        Output {
                            to_address: "Bob".to_owned(),
                            value: 2,
                        },
                    ],
                    outputs: vec![Output {
                        to_address: "Chris".to_owned(),
                        value: 4,
                    }],
                },
            ],
        );
        add_block_to_blockchain(&mut blockchain, genesis_block);

        let result = blockchain.update_with_block(block);

        assert_eq!(true, result.is_err());
        assert_eq!(Err(BlockValidationErr::InsufficientInputValue), result);
    }

    #[test]
    fn add_block_with_exactly_sufficient_inputs() {
        assert_add_block_with_sufficient_inputs(BlockOutputConfig {
            unspent_output_value: 4,
            output_value: 3,
            expected_difference: 0,
        });
    }

    #[test]
    fn add_block_with_more_than_sufficient_inputs_case1() {
        assert_add_block_with_sufficient_inputs(BlockOutputConfig {
            unspent_output_value: 4,
            output_value: 2,
            expected_difference: 1,
        });
    }

    #[test]
    fn add_block_with_more_than_sufficient_inputs_case2() {
        assert_add_block_with_sufficient_inputs(BlockOutputConfig {
            unspent_output_value: 4,
            output_value: 1,
            expected_difference: 2,
        });
    }

    #[test]
    fn add_block_with_more_than_sufficient_inputs_case3() {
        assert_add_block_with_sufficient_inputs(BlockOutputConfig {
            unspent_output_value: 4,
            output_value: 0,
            expected_difference: 3,
        });
    }

    #[test]
    fn add_one_block_without_transactions_to_blockchain() {
        let timestamp = current_time();
        let genesis_block =
            create_block_with_valid_difficulty(0, timestamp, genesis_block_hash(), vec![]);
        let mut blockchain = Blockchain::new();

        add_block_to_blockchain(&mut blockchain, genesis_block);
    }

    #[test]
    fn add_two_blocks_without_transactions_to_blockchain() {
        let timestamp = current_time();
        let genesis_block =
            create_block_with_valid_difficulty(0, timestamp, genesis_block_hash(), vec![]);
        let block = create_block_with_valid_difficulty(
            1,
            timestamp + 1,
            genesis_block.hash.clone(),
            vec![],
        );
        let mut blockchain = Blockchain::new();

        add_block_to_blockchain(&mut blockchain, genesis_block);
        add_block_to_blockchain(&mut blockchain, block);
    }

    #[test]
    fn add_three_blocks_without_transactions_to_blockchain() {
        let timestamp = current_time();
        let genesis_block =
            create_block_with_valid_difficulty(0, timestamp, genesis_block_hash(), vec![]);
        let block1 = create_block_with_valid_difficulty(
            1,
            timestamp + 1,
            genesis_block.hash.clone(),
            vec![],
        );
        let block2 =
            create_block_with_valid_difficulty(2, timestamp + 2, block1.hash.clone(), vec![]);
        let mut blockchain = Blockchain::new();

        add_block_to_blockchain(&mut blockchain, genesis_block);
        add_block_to_blockchain(&mut blockchain, block1);
        add_block_to_blockchain(&mut blockchain, block2);
    }

    #[test]
    fn add_one_block_with_one_transaction_to_blockchain() {
        let timestamp = current_time();
        let genesis_block = create_block_with_valid_difficulty(
            0,
            timestamp,
            genesis_block_hash(),
            vec![Transaction {
                inputs: vec![],
                outputs: vec![Output {
                    to_address: "Alice".to_string(),
                    value: 1,
                }],
            }],
        );
        let mut blockchain = Blockchain::new();
        let mut coinbase_transaction = create_coinbase_transaction();
        coinbase_transaction.outputs = vec![Output {
            to_address: "Chris".to_owned(),
            value: 1,
        }];
        let block = create_block_with_valid_difficulty(
            1,
            timestamp + 1,
            genesis_block.hash.clone(),
            vec![
                coinbase_transaction,
                Transaction {
                    inputs: vec![Output {
                        to_address: "Alice".to_owned(),
                        value: 1,
                    }],
                    outputs: vec![],
                },
            ],
        );
        add_block_to_blockchain(&mut blockchain, genesis_block);
        add_block_to_blockchain(&mut blockchain, block);
    }

    #[test]
    fn add_one_block_with_two_transactions_to_blockchain() {
        let timestamp = current_time();
        let genesis_block = create_block_with_valid_difficulty(
            0,
            timestamp,
            genesis_block_hash(),
            vec![Transaction {
                inputs: vec![],
                outputs: vec![
                    Output {
                        to_address: "Alice".to_string(),
                        value: 1,
                    },
                    Output {
                        to_address: "Bob".to_owned(),
                        value: 2,
                    },
                ],
            }],
        );
        let mut blockchain = Blockchain::new();
        let mut coinbase_transaction = create_coinbase_transaction();
        coinbase_transaction.outputs = vec![Output {
            to_address: "Chris".to_owned(),
            value: 3,
        }];
        let block = create_block_with_valid_difficulty(
            1,
            timestamp + 1,
            genesis_block.hash.clone(),
            vec![
                coinbase_transaction,
                Transaction {
                    inputs: vec![Output {
                        to_address: "Alice".to_owned(),
                        value: 1,
                    }],
                    outputs: vec![],
                },
                Transaction {
                    inputs: vec![Output {
                        to_address: "Bob".to_owned(),
                        value: 2,
                    }],
                    outputs: vec![],
                },
            ],
        );
        add_block_to_blockchain(&mut blockchain, genesis_block);
        add_block_to_blockchain(&mut blockchain, block);
    }

    #[test]
    fn add_one_block_with_three_transactions_to_blockchain() {
        let timestamp = current_time();
        let genesis_block = create_block_with_valid_difficulty(
            0,
            timestamp,
            genesis_block_hash(),
            vec![Transaction {
                inputs: vec![],
                outputs: vec![
                    Output {
                        to_address: "Alice".to_string(),
                        value: 1,
                    },
                    Output {
                        to_address: "Bob".to_owned(),
                        value: 2,
                    },
                    Output {
                        to_address: "John".to_owned(),
                        value: 3,
                    },
                ],
            }],
        );
        let mut blockchain = Blockchain::new();
        let mut coinbase_transaction = create_coinbase_transaction();
        coinbase_transaction.outputs = vec![Output {
            to_address: "Chris".to_owned(),
            value: 6,
        }];
        let block = create_block_with_valid_difficulty(
            1,
            timestamp + 1,
            genesis_block.hash.clone(),
            vec![
                coinbase_transaction,
                Transaction {
                    inputs: vec![Output {
                        to_address: "Alice".to_owned(),
                        value: 1,
                    }],
                    outputs: vec![],
                },
                Transaction {
                    inputs: vec![Output {
                        to_address: "Bob".to_owned(),
                        value: 2,
                    }],
                    outputs: vec![],
                },
                Transaction {
                    inputs: vec![Output {
                        to_address: "John".to_owned(),
                        value: 3,
                    }],
                    outputs: vec![],
                },
            ],
        );
        add_block_to_blockchain(&mut blockchain, genesis_block);
        add_block_to_blockchain(&mut blockchain, block);
    }
}
