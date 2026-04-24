#[derive(Clone, Debug)]
pub struct BoardConfig {
    pub width: u8,

    pub height: u8,

    pub not_left_edge: u128,

    pub not_right_edge: u128,

    pub valid_board_mask: u128,

    pub top_row_mask: u128,

    pub bottom_row_mask: u128,
}

impl BoardConfig {
    pub fn new(width: u8, height: u8) -> Self {
        let mut not_left_edge = 0u128;
        let mut not_right_edge = 0u128;
        let mut valid_board_mask = 0u128;

        let bottom_row_mask = (1_u128 << width) - 1;
        let top_row_mask = bottom_row_mask << (width * (height - 1));

        for i in 0..(width * height) {
            let i_128 = i as u128;
            valid_board_mask |= 1 << i_128;

            if i % width != 0 {
                not_left_edge |= 1 << i_128;
            }
            if i % width != width - 1 {
                not_right_edge |= 1 << i_128;
            }
        }

        Self {
            width,
            height,
            not_left_edge,
            not_right_edge,
            valid_board_mask,
            top_row_mask,
            bottom_row_mask,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_config_masks_3x3() {
        // 3x3 board (indices 0-8)
        // 6 7 8 (Top)
        // 3 4 5
        // 0 1 2 (Bottom)
        let config = BoardConfig::new(3, 3);

        // valid_board_mask: first 9 bits set
        assert_eq!(config.valid_board_mask, 0b1_1111_1111);

        // bottom_row_mask: indices 0, 1, 2
        assert_eq!(config.bottom_row_mask, 0b000_000_111);

        // top_row_mask: indices 6, 7, 8
        assert_eq!(config.top_row_mask, 0b111_000_000);

        // not_left_edge: everything except column A (indices 0, 3, 6)
        // Bits 1, 2, 4, 5, 7, 8
        assert_eq!(config.not_left_edge, 0b110_110_110);

        // not_right_edge: everything except column C (indices 2, 5, 8)
        // Bits 0, 1, 3, 4, 6, 7
        assert_eq!(config.not_right_edge, 0b011_011_011);
    }
}
