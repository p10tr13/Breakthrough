#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ply {
    pub from: u8,
    pub to: u8,
}

impl Ply {
    pub fn decode(encoded: &str, board_width: u8) -> Option<Self> {
        let bytes = encoded.as_bytes();
        let mut i = 0;

        let from_col = (bytes.get(i)?).checked_sub(b'a')?;
        i += 1;

        let mut from_row = 0u8;
        while let Some(&b) = bytes.get(i) {
            if b.is_ascii_digit() {
                from_row = from_row * 10 + (b - b'0');
                i += 1;
            } else {
                break;
            }
        }

        let to_col = (bytes.get(i)?).checked_sub(b'a')?;
        i += 1;

        let mut to_row = 0u8;
        while let Some(&b) = bytes.get(i) {
            if b.is_ascii_digit() {
                to_row = to_row * 10 + (b - b'0');
                i += 1;
            } else {
                break;
            }
        }

        if from_row == 0 || to_row == 0 {
            return None;
        }

        Some(Self {
            from: (from_row - 1) * board_width + from_col,
            to: (to_row - 1) * board_width + to_col,
        })
    }

    pub fn encode(&self, board_width: u8) -> String {
        let from_row = self.from / board_width;
        let from_col = self.from % board_width;
        let to_row = self.to / board_width;
        let to_col = self.to % board_width;

        format!(
            "{}{}{}{}",
            (from_col + b'a') as char,
            from_row + 1,
            (to_col + b'a') as char,
            to_row + 1
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_single_digit() {
        let board_width = 8;
        let ply = Ply { from: 0, to: 9 };

        let encoded = ply.encode(board_width);
        assert_eq!(encoded, "a1b2");

        let decoded = Ply::decode(&encoded, board_width).unwrap();
        assert_eq!(decoded, ply);
    }

    #[test]
    fn test_roundtrip_multi_digit() {
        let board_width = 11;

        let ply = Ply {
            from: 10 * 11 + 0, // a11
            to: 10 * 11 + 1,   // b11
        };

        let encoded = ply.encode(board_width);
        assert_eq!(encoded, "a11b11");

        let decoded = Ply::decode(&encoded, board_width).unwrap();
        assert_eq!(decoded, ply);
    }

    #[test]
    fn test_mixed_sizes() {
        let board_width = 11;

        let ply = Ply {
            from: 0, // a1
            to: 12,  // b2 (1 * 11 + 1)
        };

        let encoded = ply.encode(board_width);
        assert_eq!(encoded, "a1b2");

        let decoded = Ply::decode(&encoded, board_width).unwrap();
        assert_eq!(decoded, ply);
    }

    #[test]
    fn test_decode_a10_b11() {
        let board_width = 11;

        let ply = Ply::decode("a10b11", board_width).unwrap();

        assert_eq!(ply.from, 9 * 11 + 0);
        assert_eq!(ply.to, 10 * 11 + 1);
    }

    #[test]
    fn test_invalid_empty() {
        assert!(Ply::decode("", 8).is_none());
    }

    #[test]
    fn test_invalid_format() {
        assert!(Ply::decode("!!??", 8).is_none());
        assert!(Ply::decode("a", 8).is_none());
        assert!(Ply::decode("a1b", 8).is_none());
    }

    #[test]
    fn test_large_board_19x19() {
        let board_width = 19;

        let ply = Ply {
            from: 180, // j10
            to: 200,   // k11
        };

        let encoded = ply.encode(board_width);
        let decoded = Ply::decode(&encoded, board_width).unwrap();

        assert_eq!(decoded, ply);
    }
}
