//! 工作量证明（手册 §3.2 / §6.3-5）：找 `nonce` 使
//! `sha256("<timestamp>:<bodyForHash>:<nonce>")` 的十六进制以 `difficulty` 个 `0` 开头。
//!
//! 使用方（后续切片）：论坛游客回复（`bodyForHash = "<topicId>:<content>"`）与
//! `geek join redeem`（`bodyForHash = "join:<token>:<login>:<email>"`）。

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// 提交给服务端的 PoW 凭据（`{ "timestamp": 毫秒, "nonce": "1–32 字" }`）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proof {
    pub timestamp: u64,
    pub nonce: String,
}

/// nonce 长度上限（手册：1–32 字符）。
const MAX_NONCE_LEN: usize = 32;

fn digest(body_for_hash: &str, timestamp: u64, nonce: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(format!("{timestamp}:{body_for_hash}:{nonce}").as_bytes());
    hasher.finalize().into()
}

/// 十六进制前导 `0` 的个数（哈希以字节计的 4 bit 半字节计数）。
pub fn leading_zero_nibbles(hash: &[u8; 32]) -> u8 {
    let mut count = 0u8;
    for byte in hash {
        if byte >> 4 == 0 {
            count += 1;
        } else {
            break;
        }
        if byte & 0x0f == 0 {
            count += 1;
        } else {
            break;
        }
    }
    count
}

/// 求解：nonce 用十进制计数（`1`、`2`、`3`…）。难度 3 期望约 4096 次哈希，毫秒级。
pub fn solve(body_for_hash: &str, difficulty: u8, timestamp_ms: u64) -> Proof {
    assert!(
        difficulty <= 64,
        "difficulty 不能超过 SHA-256 的 64 个十六进制位"
    );
    let mut counter: u64 = 0;
    loop {
        counter += 1;
        let nonce = counter.to_string();
        if nonce.len() <= MAX_NONCE_LEN
            && leading_zero_nibbles(&digest(body_for_hash, timestamp_ms, &nonce)) >= difficulty
        {
            return Proof {
                timestamp: timestamp_ms,
                nonce,
            };
        }
    }
}

/// 本地校验（测试与自检用；服务端还会检查时间偏差 ≤5 分钟）。
pub fn verify_hash(proof: &Proof, body_for_hash: &str, difficulty: u8) -> bool {
    (1..=MAX_NONCE_LEN).contains(&proof.nonce.len())
        && leading_zero_nibbles(&digest(body_for_hash, proof.timestamp, &proof.nonce)) >= difficulty
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solves_and_verifies() {
        for difficulty in 1..=3u8 {
            let proof = solve("t1001:hello", difficulty, 1_700_000_000_000);
            assert!(
                verify_hash(&proof, "t1001:hello", difficulty),
                "difficulty {difficulty}"
            );
            assert!(proof.nonce.len() <= MAX_NONCE_LEN);
        }
    }

    #[test]
    fn rejects_tampered_body_or_nonce() {
        let proof = solve("t1:x", 2, 1234);
        assert!(!verify_hash(&proof, "t1:y", 2));
        assert!(!verify_hash(
            &Proof {
                nonce: String::new(),
                ..proof.clone()
            },
            "t1:x",
            2
        ));
        assert!(!verify_hash(
            &Proof {
                nonce: "x".repeat(33),
                ..proof
            },
            "t1:x",
            2
        ));
    }

    #[test]
    fn counts_leading_zero_nibbles() {
        assert_eq!(leading_zero_nibbles(&[0x00; 32]), 64);
        assert_eq!(leading_zero_nibbles(&[0x0f; 32]), 1);
        assert_eq!(leading_zero_nibbles(&[0x10; 32]), 0);
    }
}
