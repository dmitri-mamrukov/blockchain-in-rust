use std::collections::HashSet;

use crate::{u64_bytes, Address, BlockHash, Hashable};

/**
 * Represents a transaction output that has the recipient's address and the
 * value to transfer to the recipient.
 */
#[derive(Clone, Debug, PartialEq)]
pub struct Output {
    pub to_address: Address,
    pub value: u64,
}

impl Hashable for Output {
    /**
     * Returns a vector of hashable bytes that represents the transaction
     * output.
     */
    fn bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        bytes.extend(self.to_address.as_bytes());
        bytes.extend(&u64_bytes(self.value));

        bytes
    }
}

/**
 * Represents a blockchain transaction.
 *
 * Example:
 *
 * Alice has 50 coins. Bob has 7 coins. Alice sends Bob 12 coins.
 *
 * Is that all? Not quite. Blockchain != spreadsheet.
 *
 * Transactions only contain two important pieces of information:
 *
 * - Set of inputs (which are unused outputs from previous transactions).
 *
 * - Set of outputs (which are new outputs that can be used in future
 *   transactions).
 *
 * From here we can calculate:
 *
 * - the value of the transaction: sum of inputs.
 * - the value of the fee: sum of inputs - sum of outputs.
 *
 * Fee
 * ---
 *
 * Whenever you send a transaction, you want to include something for whoever is
 * going to mine the block to incentivize them to include your transaction in
 * the next block right because it takes work for the miner to mine a block so
 * they don't just want to do it for free.
 *
 * Illustration
 * ------------
 *
 * A transaction will take a set of outputs as inputs and generate a set of
 * outputs in turn.
 *
 * Example:
 *
 * A mining fee is 2 coins. This transaction starts with 50 coins.
 *
 * [50] -> [12] (inputs)
 *         [36] (outputs)
 *
 * 50 -> 50 - 2 (fee) = 48 (remaining coins). 12 coins (inputs) are going to be
 * spent. So 48 - 12 = 36 (outputs).
 *
 * If a transaction does not have room in it for a fee for the miner, what
 * incentive does the miner have to add the transaction to their block?
 *
 * Coinbase Transactions
 * ---------------------
 *
 *  A blockchain's history has to start somewhere.
 *
 * - Do not require inputs
 *
 * - Produce an output
 *
 * - Allow the miner to collect all the transaction fees in that block and
 *   that block's block reward (coin genesis).
 *
 * Transaction Verification Requirements
 * -------------------------------------
 *
 * We have to protect against:
 *
 * - Overspending (where did the money come from?). For example, Alice sends a
 *   transaction, and it say "subtract 5 coins from Alice's balance but add
 *   twelve coins to Bob's balance". That's dishonest because you're generating
 *   these extra seven coins out of nowhere.
 *
 *   So the sum of the values of the inputs must be greater than or equal to the
 *   sum of the values of the generated outputs: I can't input 5 coins and be
 *   able to output 7.
 *
 * - Double-spending (is the money available?). For example, Alice makes two
 *   transactions, and one of them is "Alice sends 50 coins to Chris" and the
 *   other one is "Alice sends 50 coins to Bob". Both transactions execute
 *   at the same time, and the effect that Alice sends the same digital assets
 *   (50 coins) to both people to get physical assets from both Chris and Bob.
 *
 *   So make sure that any one output is never used as an input more than once.
 *   This can be done by maintaining a pool of unspent outputs and rejecting
 *   any transaction that tries to spend outputs that don't exist in the pool.
 *
 * - Impersonation (who owns the money and who is sending it?). It's like
 *   identity theft. For example, Alice generates a transaction that says that
 *   Chris sent Alice the entire contents of Chris's wallet and Alice tells this
 *   to Bob. Bob has no way to contact Chris directly, so all Bob knows to do is
 *   trust Alice and updates his ledger and now as far as the network is
 *   concerned Chris is out of his money, so we have to fix that problem.
 *
 *   This can be solved by adding a cryptographic "signature" (to mathematically
 *   verify) to outputs to verify they're being spent by their owner.
 *
 *   We can't assume that whoever sent us the transaction over the network is
 *   also the person who created the transaction.
 *
 *   For now, we'll kind of ignore solving this problem. We might come back to
 *   it when we go over smart contracts.
 *
 * (In Bitcoin, there are more transaction verification requirements but for
 * our project, we're going to cover these three.)
 */
#[derive(Debug, PartialEq)]
pub struct Transaction {
    pub inputs: Vec<Output>,
    pub outputs: Vec<Output>,
}

impl Transaction {
    /**
     * Returns the sum of the transaction's inputs.
     */
    pub fn input_value(&self) -> u64 {
        self.inputs.iter().map(|input| input.value).sum()
    }

    /**
     * Returns the sum of the transaction's outputs.
     */
    pub fn output_value(&self) -> u64 {
        self.outputs.iter().map(|output| output.value).sum()
    }

    /**
     * Returns a set of hashes of the transaction's inputs.
     */
    pub fn input_hashes(&self) -> HashSet<BlockHash> {
        self.inputs
            .iter()
            .map(Hashable::hash)
            .collect::<HashSet<BlockHash>>()
    }

    /**
     * Returns a set of hashes of the transaction's outputs.
     */
    pub fn output_hashes(&self) -> HashSet<BlockHash> {
        self.outputs
            .iter()
            .map(Hashable::hash)
            .collect::<HashSet<BlockHash>>()
    }

    /**
     * Returns a flag that states whether this transaction is a coinbase one.
     * A coinbase transaction has empty inputs.
     */
    pub fn is_coinbase(&self) -> bool {
        self.inputs.is_empty()
    }
}

impl Hashable for Transaction {
    /**
     * Returns a vector of hashable bytes that represents the transaction.
     */
    fn bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        bytes.extend(
            self.inputs
                .iter()
                .flat_map(Hashable::bytes)
                .collect::<Vec<u8>>(),
        );
        bytes.extend(
            self.outputs
                .iter()
                .flat_map(Hashable::bytes)
                .collect::<Vec<u8>>(),
        );

        bytes
    }
}

#[cfg(test)]
mod output_constructor_tests {
    use super::Output;

    #[test]
    fn constructor() {
        let instance = Output {
            to_address: "test-recipient-address".to_string(),
            value: 1,
        };

        assert_eq!(1, instance.value);
    }
}

#[cfg(test)]
mod hashable_output_tests {
    use super::{Hashable, Output};

    #[test]
    fn bytes() {
        let output = Output {
            to_address: "test-recipient-address".to_string(),
            value: 1,
        };

        let result = output.bytes();

        assert_eq!(
            vec![
                116, 101, 115, 116, 45, 114, 101, 99, 105, 112, 105, 101, 110, 116, 45, 97, 100,
                100, 114, 101, 115, 115, 1, 0, 0, 0, 0, 0, 0, 0
            ],
            result
        );
    }
}

#[cfg(test)]
mod transaction_constructor_tests {
    use super::{Output, Transaction};

    #[test]
    fn constructor() {
        let instance = Transaction {
            inputs: vec![Output {
                to_address: "test-recipient-address1".to_string(),
                value: 1,
            }],
            outputs: vec![Output {
                to_address: "test-recipient-address2".to_string(),
                value: 2,
            }],
        };

        assert_eq!(
            vec![Output {
                to_address: "test-recipient-address1".to_string(),
                value: 1,
            }],
            instance.inputs
        );
        assert_eq!(
            vec![Output {
                to_address: "test-recipient-address2".to_string(),
                value: 2,
            }],
            instance.outputs
        );
    }
}

#[cfg(test)]
mod transaction_tests {
    use std::collections::HashSet;

    use super::{BlockHash, Hashable, Output, Transaction};

    #[test]
    fn input_value_with_zero_elements() {
        let transaction = Transaction {
            inputs: vec![],
            outputs: vec![],
        };

        let result = transaction.input_value();

        assert_eq!(0, result);
    }

    #[test]
    fn input_value_with_three_elements() {
        let transaction = Transaction {
            inputs: vec![
                Output {
                    to_address: "test-recipient-address1".to_string(),
                    value: 1,
                },
                Output {
                    to_address: "test-recipient-address2".to_string(),
                    value: 2,
                },
                Output {
                    to_address: "test-recipient-address3".to_string(),
                    value: 3,
                },
            ],
            outputs: vec![],
        };

        let result = transaction.input_value();

        assert_eq!(6, result);
    }

    #[test]
    fn output_value_with_zero_elements() {
        let transaction = Transaction {
            inputs: vec![],
            outputs: vec![],
        };

        let result = transaction.output_value();

        assert_eq!(0, result);
    }

    #[test]
    fn output_value_with_three_elements() {
        let transaction = Transaction {
            inputs: vec![],
            outputs: vec![
                Output {
                    to_address: "test-recipient-address1".to_string(),
                    value: 1,
                },
                Output {
                    to_address: "test-recipient-address2".to_string(),
                    value: 2,
                },
                Output {
                    to_address: "test-recipient-address3".to_string(),
                    value: 3,
                },
            ],
        };

        let result = transaction.output_value();

        assert_eq!(6, result);
    }

    #[test]
    fn input_hashes_with_zero_elements() {
        let transaction = Transaction {
            inputs: vec![],
            outputs: vec![],
        };

        let result = transaction.input_hashes();

        assert_eq!(HashSet::<BlockHash>::new(), result);
    }

    #[test]
    fn input_hashes_with_three_elements() {
        let transaction = Transaction {
            inputs: vec![
                Output {
                    to_address: "test-recipient-address1".to_string(),
                    value: 1,
                },
                Output {
                    to_address: "test-recipient-address2".to_string(),
                    value: 2,
                },
                Output {
                    to_address: "test-recipient-address3".to_string(),
                    value: 3,
                },
            ],
            outputs: vec![],
        };
        let mut expected_set = HashSet::<BlockHash>::new();
        for input in &transaction.inputs {
            expected_set.insert(input.hash());
        }

        let result = transaction.input_hashes();

        assert_eq!(expected_set, result);
    }

    #[test]
    fn output_hashes_with_zero_elements() {
        let transaction = Transaction {
            inputs: vec![],
            outputs: vec![],
        };

        let result = transaction.output_hashes();

        assert_eq!(HashSet::<BlockHash>::new(), result);
    }

    #[test]
    fn output_hashes_with_three_elements() {
        let transaction = Transaction {
            inputs: vec![],
            outputs: vec![
                Output {
                    to_address: "test-recipient-address1".to_string(),
                    value: 1,
                },
                Output {
                    to_address: "test-recipient-address2".to_string(),
                    value: 2,
                },
                Output {
                    to_address: "test-recipient-address3".to_string(),
                    value: 3,
                },
            ],
        };
        let mut expected_set = HashSet::<BlockHash>::new();
        for output in &transaction.outputs {
            expected_set.insert(output.hash());
        }

        let result = transaction.output_hashes();

        assert_eq!(expected_set, result);
    }

    #[test]
    fn is_coinbase_with_zero_elements() {
        let transaction = Transaction {
            inputs: vec![],
            outputs: vec![],
        };

        let result = transaction.is_coinbase();

        assert_eq!(true, result);
    }

    #[test]
    fn is_coinbase_with_one_element() {
        let transaction = Transaction {
            inputs: vec![Output {
                to_address: "test-recipient-address".to_string(),
                value: 1,
            }],
            outputs: vec![],
        };

        let result = transaction.is_coinbase();

        assert_eq!(false, result);
    }
}

#[cfg(test)]
mod hashable_transaction_tests {
    use super::{Hashable, Output, Transaction};

    #[test]
    fn bytes() {
        let transaction = Transaction {
            inputs: vec![Output {
                to_address: "test-recipient-address1".to_string(),
                value: 1,
            }],
            outputs: vec![Output {
                to_address: "test-recipient-address2".to_string(),
                value: 2,
            }],
        };

        let result = transaction.bytes();

        assert_eq!(
            vec![
                116, 101, 115, 116, 45, 114, 101, 99, 105, 112, 105, 101, 110, 116, 45, 97, 100,
                100, 114, 101, 115, 115, 49, 1, 0, 0, 0, 0, 0, 0, 0, 116, 101, 115, 116, 45, 114,
                101, 99, 105, 112, 105, 101, 110, 116, 45, 97, 100, 100, 114, 101, 115, 115, 50, 2,
                0, 0, 0, 0, 0, 0, 0
            ],
            result
        );
    }
}
