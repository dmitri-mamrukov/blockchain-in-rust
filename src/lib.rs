use std::time::{SystemTime, SystemTimeError, UNIX_EPOCH};

mod block;
mod blockchain;
mod hashable;
pub mod transaction;

pub use crate::block::check_difficulty;
pub use crate::block::Block;
pub use crate::blockchain::Blockchain;
pub use crate::hashable::Hashable;
pub use crate::transaction::Transaction;

type BlockHash = Vec<u8>;
type Address = String;

/**
 * Returns the current time in milliseconds.
 */
pub fn now() -> Result<u128, SystemTimeError> {
    let duration = SystemTime::now().duration_since(UNIX_EPOCH)?;

    Ok(duration.as_millis())
}

/**
 * Returns a little-endian array of the data's 4 bytes.
 */
pub fn u32_bytes(data: u32) -> [u8; 4] {
    [
        data as u8,
        (data >> 8) as u8,
        (data >> 16) as u8,
        (data >> 24) as u8,
    ]
}

/**
 * Returns a little-endian array of the data's 8 bytes.
 */
pub fn u64_bytes(data: u64) -> [u8; 8] {
    [
        data as u8,
        (data >> 8) as u8,
        (data >> 16) as u8,
        (data >> 24) as u8,
        (data >> 32) as u8,
        (data >> 40) as u8,
        (data >> 48) as u8,
        (data >> 56) as u8,
    ]
}

/**
 * Returns a little-endian array of the data's 16 bytes.
 */
pub fn u128_bytes(data: u128) -> [u8; 16] {
    [
        data as u8,
        (data >> 8) as u8,
        (data >> 16) as u8,
        (data >> 24) as u8,
        (data >> 32) as u8,
        (data >> 40) as u8,
        (data >> 48) as u8,
        (data >> 56) as u8,
        (data >> 64) as u8,
        (data >> 72) as u8,
        (data >> 80) as u8,
        (data >> 88) as u8,
        (data >> 96) as u8,
        (data >> 104) as u8,
        (data >> 112) as u8,
        (data >> 120) as u8,
    ]
}

/**
 * The function assumes that the byte vector has 32 bytes.
 *
 * Performs ORing the most significant 16 bytes as a u128 result that represents
 * the difficulty:
 *
 * v[31] | v[30] | v[29] | ... | v[18] | v[17] | v[16]
 */
pub fn difficulty_bytes_as_u128(v: &[u8]) -> u128 {
    u128::from(v[31]) << 120
        | u128::from(v[30]) << 112
        | u128::from(v[29]) << 104
        | u128::from(v[28]) << 96
        | u128::from(v[27]) << 88
        | u128::from(v[26]) << 80
        | u128::from(v[25]) << 72
        | u128::from(v[24]) << 64
        | u128::from(v[23]) << 56
        | u128::from(v[22]) << 48
        | u128::from(v[21]) << 40
        | u128::from(v[20]) << 32
        | u128::from(v[19]) << 24
        | u128::from(v[18]) << 16
        | u128::from(v[17]) << 8
        | u128::from(v[16])
}

#[cfg(test)]
mod now_tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::now;

    #[test]
    fn current_time() {
        let milliseconds_since_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let result = now();

        assert!(result.is_ok());
        assert!(milliseconds_since_epoch >= result.unwrap());
    }
}

#[cfg(test)]
mod u32_bytes_tests {
    use super::u32_bytes;

    #[test]
    fn with_0x00000000() {
        let data = 0x00000000_u32;

        let result = u32_bytes(data);

        assert_eq!([0x00, 0x00, 0x00, 0x00], result);
    }

    #[test]
    fn with_0x03020100() {
        let data = 0x03020100_u32;

        let result = u32_bytes(data);

        assert_eq!([0x00, 0x01, 0x02, 0x03], result);
    }

    #[test]
    fn with_0x30201000() {
        let data = 0x30201000_u32;

        let result = u32_bytes(data);

        assert_eq!([0x00, 0x10, 0x20, 0x30], result);
    }

    #[test]
    fn with_0x30211203() {
        let data = 0x30211203_u32;

        let result = u32_bytes(data);

        assert_eq!([0x03, 0x12, 0x21, 0x30], result);
    }
}

#[cfg(test)]
mod u64_bytes_tests {
    use super::u64_bytes;

    #[test]
    fn with_0x0000000000000000() {
        let data = 0x0000000000000000_u64;

        let result = u64_bytes(data);

        assert_eq!([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], result);
    }

    #[test]
    fn with_0x0706050403020100() {
        let data = 0x0706050403020100_u64;

        let result = u64_bytes(data);

        assert_eq!([0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07], result);
    }

    #[test]
    fn u64_bytes_with_0x7060504030201000() {
        let data = 0x7060504030201000_u64;

        let result = u64_bytes(data);

        assert_eq!([0x00, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70], result);
    }

    #[test]
    fn with_0x7061524334251607() {
        let data = 0x7061524334251607_u64;

        let result = u64_bytes(data);

        assert_eq!([0x07, 0x16, 0x25, 0x34, 0x43, 0x52, 0x61, 0x70], result);
    }
}

#[cfg(test)]
mod u128_bytes_tests {
    use super::u128_bytes;

    #[test]
    fn with_0x00000000000000000000000000000000() {
        let data = 0x00000000000000000000000000000000_u128;

        let result = u128_bytes(data);

        assert_eq!(
            [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00
            ],
            result
        );
    }

    #[test]
    fn with_0x0f0e0d0c0b0a09080706050403020100() {
        let data = 0x0f0e0d0c0b0a09080706050403020100_u128;

        let result = u128_bytes(data);

        assert_eq!(
            [
                0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
                0x0e, 0x0f
            ],
            result
        );
    }

    #[test]
    fn with_0xf0e0d0c0b0a090807060504030201000() {
        let data = 0xf0e0d0c0b0a090807060504030201000_u128;

        let result = u128_bytes(data);

        assert_eq!(
            [
                0x00, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0,
                0xe0, 0xf0
            ],
            result
        );
    }

    #[test]
    fn with_0xf0e1d2c3b4a5968778695a4b3c2d1e0f() {
        let data = 0xf0e1d2c3b4a5968778695a4b3c2d1e0f_u128;

        let result = u128_bytes(data);

        assert_eq!(
            [
                0x0f, 0x1e, 0x2d, 0x3c, 0x4b, 0x5a, 0x69, 0x78, 0x87, 0x96, 0xa5, 0xb4, 0xc3, 0xd2,
                0xe1, 0xf0
            ],
            result
        );
    }
}

#[cfg(test)]
mod difficulty_bytes_as_u128_tests {
    use super::difficulty_bytes_as_u128;

    #[test]
    fn with_zero_bytes() {
        let data = vec![
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8,
        ];

        let result = difficulty_bytes_as_u128(&data);

        assert_eq!(0_u128, result);
    }

    #[test]
    fn with_one_byte_at_index_16() {
        let data = vec![
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 1_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8,
        ];

        let result = difficulty_bytes_as_u128(&data);

        assert_eq!(2_u128.pow(0), result);
    }

    #[test]
    fn with_one_byte_at_index_17() {
        let data = vec![
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 1_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8,
        ];

        let result = difficulty_bytes_as_u128(&data);

        assert_eq!(2_u128.pow(8), result);
    }

    #[test]
    fn with_one_byte_at_index_18() {
        let data = vec![
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 1_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8,
        ];

        let result = difficulty_bytes_as_u128(&data);

        assert_eq!(2_u128.pow(16), result);
    }

    #[test]
    fn with_one_byte_at_index_19() {
        let data = vec![
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 1_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8,
        ];

        let result = difficulty_bytes_as_u128(&data);

        assert_eq!(2_u128.pow(24), result);
    }

    #[test]
    fn with_one_byte_at_index_20() {
        let data = vec![
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 1_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8,
        ];

        let result = difficulty_bytes_as_u128(&data);

        assert_eq!(2_u128.pow(32), result);
    }

    #[test]
    fn with_one_byte_at_index_21() {
        let data = vec![
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 1_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8,
        ];

        let result = difficulty_bytes_as_u128(&data);

        assert_eq!(2_u128.pow(40), result);
    }

    #[test]
    fn with_one_byte_at_index_22() {
        let data = vec![
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 1_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8,
        ];

        let result = difficulty_bytes_as_u128(&data);

        assert_eq!(2_u128.pow(48), result);
    }

    #[test]
    fn with_one_byte_at_index_23() {
        let data = vec![
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 1_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8,
        ];

        let result = difficulty_bytes_as_u128(&data);

        assert_eq!(2_u128.pow(56), result);
    }

    #[test]
    fn with_one_byte_at_index_24() {
        let data = vec![
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 1_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8,
        ];

        let result = difficulty_bytes_as_u128(&data);

        assert_eq!(2_u128.pow(64), result);
    }

    #[test]
    fn with_one_byte_at_index_25() {
        let data = vec![
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 1_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8,
        ];

        let result = difficulty_bytes_as_u128(&data);

        assert_eq!(2_u128.pow(72), result);
    }

    #[test]
    fn with_one_byte_at_index_26() {
        let data = vec![
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 1_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8,
        ];

        let result = difficulty_bytes_as_u128(&data);

        assert_eq!(2_u128.pow(80), result);
    }

    #[test]
    fn with_one_byte_at_index_27() {
        let data = vec![
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 1_u8,
            0_u8, 0_u8, 0_u8, 0_u8,
        ];

        let result = difficulty_bytes_as_u128(&data);

        assert_eq!(2_u128.pow(88), result);
    }

    #[test]
    fn with_one_byte_at_index_28() {
        let data = vec![
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            1_u8, 0_u8, 0_u8, 0_u8,
        ];

        let result = difficulty_bytes_as_u128(&data);

        assert_eq!(2_u128.pow(96), result);
    }

    #[test]
    fn with_one_byte_at_index_29() {
        let data = vec![
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 1_u8, 0_u8, 0_u8,
        ];

        let result = difficulty_bytes_as_u128(&data);

        assert_eq!(2_u128.pow(104), result);
    }

    #[test]
    fn with_one_byte_at_index_30() {
        let data = vec![
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 1_u8, 0_u8,
        ];

        let result = difficulty_bytes_as_u128(&data);

        assert_eq!(2_u128.pow(112), result);
    }

    #[test]
    fn with_one_byte_at_index_31() {
        let data = vec![
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 1_u8,
        ];

        let result = difficulty_bytes_as_u128(&data);

        assert_eq!(2_u128.pow(120), result);
    }

    #[test]
    fn with_increasing_bytes() {
        let data = vec![
            0_u8, 1_u8, 2_u8, 3_u8, 4_u8, 5_u8, 6_u8, 7_u8, 8_u8, 9_u8, 10_u8, 11_u8, 12_u8, 13_u8,
            14_u8, 15_u8, 16_u8, 17_u8, 18_u8, 19_u8, 20_u8, 21_u8, 22_u8, 23_u8, 24_u8, 25_u8,
            26_u8, 27_u8, 28_u8, 29_u8, 30_u8, 31_u8,
        ];
        let expected_result: u128 = u128::from(16_u8)
            | u128::from(17_u8) << 8
            | u128::from(18_u8) << 16
            | u128::from(19_u8) << 24
            | u128::from(20_u8) << 32
            | u128::from(21_u8) << 40
            | u128::from(22_u8) << 48
            | u128::from(23_u8) << 56
            | u128::from(24_u8) << 64
            | u128::from(25_u8) << 72
            | u128::from(26_u8) << 80
            | u128::from(27_u8) << 88
            | u128::from(28_u8) << 96
            | u128::from(29_u8) << 104
            | u128::from(30_u8) << 112
            | u128::from(31_u8) << 120;
        assert_eq!(41362427191743139026751447860679676176, expected_result);

        let result = difficulty_bytes_as_u128(&data);

        assert_eq!(expected_result, result);
    }
}
