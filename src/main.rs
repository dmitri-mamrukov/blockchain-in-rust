use blockchainlib::{now, transaction, Block, Blockchain, Hashable, Transaction};

#[allow(unused_assignments)]
/**
 * Writing a Working Example
 *
 * We need to:
 *
 * 1. Create a genesis block with transactions.
 *
 * 2. Mine it.
 *
 * 3. Add it to the blockchain.
 *
 * 4. Create another block with more transactions (particularly some that use
 *    transactions from the first block).
 *
 * 5. Mine that one.
 *
 * 6. Add it to the blockchain.
 */
fn main() {
    // FF = 11111111 11111111 (2 bytes = 16 bits)
    // 16 bytes * 8 bits = 128 bits
    //
    // If the difficulty were 0x0000_0000_0000_0000_0000_0000_0000_0000, then
    // mining will never succeed because it's impossible to satisfy
    // difficulty = 0 > some value of the block's hash.
    // So we have to use a reasonable difficulty value for illustration
    // purposes.
    let difficulty: u128 = 0x00ff_ffff_ffff_ffff_ffff_ffff_ffff_ffff;

    let mut genesis_block = Block::new(
        0,
        now().expect("Failure to get the current time in milliseconds."),
        vec![0; 32],
        vec![Transaction {
            inputs: vec![],
            outputs: vec![
                transaction::Output {
                    to_address: "Alice".to_owned(),
                    value: 1,
                },
                transaction::Output {
                    to_address: "Bob".to_owned(),
                    value: 2,
                },
            ],
        }],
        difficulty,
    );
    println!("Genesis block: {:?}", &genesis_block);

    println!("Genesis block before mining: {:?}", &genesis_block);

    genesis_block.mine();

    println!("Genesis block after mining: {:?}", &genesis_block);

    println!();

    println!("Building a blockchain");

    let mut last_hash = genesis_block.hash().clone();
    let mut blockchain = Blockchain::new();
    blockchain
        .update_with_block(genesis_block)
        .expect("Failed to add the genesis block.");

    let mut block = Block::new(
        1,
        now().expect("Failure to get the current time in milliseconds."),
        last_hash,
        vec![
            Transaction {
                inputs: vec![],
                outputs: vec![transaction::Output {
                    to_address: "Chris".to_owned(),
                    value: 4,
                }],
            },
            Transaction {
                inputs: vec![
                    transaction::Output {
                        to_address: "Alice".to_owned(),
                        value: 1,
                    },
                    transaction::Output {
                        to_address: "Bob".to_owned(),
                        value: 2,
                    },
                ],
                outputs: vec![transaction::Output {
                    to_address: "Chris".to_owned(),
                    value: 3,
                }],
            },
        ],
        difficulty,
    );

    block.mine();

    println!("Mined block {:?}", &block);

    last_hash = block.hash.clone();

    blockchain
        .update_with_block(block)
        .expect("Failed to add a block.");
}
