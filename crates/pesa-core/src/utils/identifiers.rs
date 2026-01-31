use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use rand::{Rng, distributions::Alphanumeric};
use serde::{Deserialize, Serialize};

/// Various kinds of Ids that are used to identify this particular transaction
/// Safaricom for some reason has all these IDs instead of universal ids for daraja pipeline
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Identifiers {
    pub originator_conversation_id: String,
    pub conversation_id: String,
    /// Transaction engine id: MJW2SDY6ZL
    pub transaction_id: String,
    /// Pesa playground sandbox request id
    pub request_id: String,
}

impl Identifiers {
    pub fn new() -> Self {
        Self {
            request_id: Self::generate_request_id(),
            transaction_id: Self::generate_transaction_id(),
            conversation_id: Self::generate_conversation_id(),
            originator_conversation_id: Self::generate_originator_conversation_id(),
        }
    }
    pub fn generate_originator_conversation_id() -> String {
        let mut rng = rand::thread_rng();

        let part1: String = (0..4)
            .map(|_| rng.gen_range(0..16))
            .map(|x| format!("{:x}", x))
            .collect();
        let part2: String = (0..8)
            .map(|_| rng.gen_range(0..16))
            .map(|x| format!("{:x}", x))
            .collect();
        let part3: String = (0..2)
            .map(|_| rng.gen_range(0..16))
            .map(|x| format!("{:x}", x))
            .collect();
        format!("{}-{}-{}", part1, part2, part3)
    }
    /// Generates time based safaricom mpesa transaction Id: MJW2SDY6ZL
    pub fn generate_transaction_id() -> String {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();

        let timestamp_str = Self::to_base36(now_ms as u64);

        let rand_suffix: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .map(|c| (c as char).to_ascii_uppercase())
            .take(10 - timestamp_str.len())
            .collect();
        format!("{}{}", timestamp_str, rand_suffix)
    }

    // Base36 helper
    fn to_base36(mut num: u64) -> String {
        let mut chars = Vec::new();
        let base = 36;
        while num > 0 {
            let rem = num % base;
            chars.push(match rem {
                0..=9 => (b'0' + rem as u8) as char,
                _ => (b'A' + (rem as u8 - 10)) as char,
            });
            num /= base;
        }
        chars.reverse();
        chars.into_iter().collect()
    }

    pub fn generate_request_id() -> String {
        let mut rng = rand::thread_rng();

        let part1: String = (0..4)
            .map(|_| rng.gen_range(0..16))
            .map(|x| format!("{:x}", x))
            .collect();
        let part2: String = (0..4)
            .map(|_| rng.gen_range(0..16))
            .map(|x| format!("{:x}", x))
            .collect();
        let part3: String = (0..4)
            .map(|_| rng.gen_range(0..16))
            .map(|x| format!("{:x}", x))
            .collect();
        let part4: String = (0..4)
            .map(|_| rng.gen_range(0..16))
            .map(|x| format!("{:x}", x))
            .collect();
        let part5: String = (0..16)
            .map(|_| rng.gen_range(0..16))
            .map(|x| format!("{:x}", x))
            .collect();

        format!("{}-{}-{}-{}{}", part1, part2, part3, part4, part5)
    }

    pub fn generate_conversation_id_prefix(prefix: &str) -> String {
        let mut rng = rand::thread_rng();

        let part2 = Utc::now().format("%Y%m%d").to_string();

        let part3: String = (0..10)
            .map(|_| rng.gen_range(0..16))
            .map(|x| format!("{:x}", x))
            .collect();
        format!("{}_{}_{}", prefix, part2, part3)
    }

    pub fn generate_conversation_id() -> String {
        Self::generate_conversation_id_prefix("AG")
    }
}
