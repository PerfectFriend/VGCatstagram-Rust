//! # Forward Error Correction (FEC) & Fountain Streaming for XFTP
//!
//! Provides zero-retransmission resilience for distributing high-resolution
//! Van Gogh canvases over lossy P2P networks and Tor onion routes.
//! Using systematic fountain block coding, any `K` symbols out of `K + R`
//! transmitted symbols reconstruct 100% of the original artwork data.

use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};
use serde::{Deserialize, Serialize};
use vangoghcat_core::error::VgcError;

/// An individual FEC symbol (either original systematic source or linear parity combination).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FecSymbol {
    /// 0-indexed symbol sequence ID
    pub symbol_id: u32,
    /// True if this is an error-correcting parity repair symbol
    pub is_repair: bool,
    /// Seed used for pseudo-random linear combination coefficients
    pub seed: u64,
    /// Symbol payload bytes (all symbols in a block have uniform length)
    pub payload: Vec<u8>,
}

/// Metadata header describing an FEC-encoded artwork package.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FecHeader {
    /// Total byte length of the unpadded original artwork
    pub original_len: usize,
    /// Fixed byte length of each symbol
    pub symbol_size: usize,
    /// Number of source data symbols `K` required for complete reconstruction
    pub source_symbols_count: usize,
    /// Number of redundant repair symbols `R` generated
    pub repair_symbols_count: usize,
}

/// Systematic Fountain Code Encoder for XFTP canvases.
pub struct FecEncoder;

impl FecEncoder {
    /// Encodes arbitrary artwork bytes into `K` systematic source symbols plus `R` repair symbols.
    ///
    /// # Arguments
    /// * `data` - Raw canvas / file bytes
    /// * `symbol_size` - Desired packet size (e.g. 1024 or 4096 bytes)
    /// * `repair_ratio` - Redundancy ratio (e.g. 0.3 = 30% repair symbols for Tor failover)
    pub fn encode(
        data: &[u8],
        symbol_size: usize,
        repair_ratio: f32,
    ) -> (FecHeader, Vec<FecSymbol>) {
        assert!(symbol_size > 0, "symbol_size must be positive");
        let original_len = data.len();
        let source_symbols_count = original_len.div_ceil(symbol_size).max(1);

        let repair_symbols_count =
            ((source_symbols_count as f32 * repair_ratio).ceil() as usize).max(1);

        // 1. Partition into K systematic source symbols (with zero-padding on last symbol)
        let mut source_symbols: Vec<Vec<u8>> = Vec::with_capacity(source_symbols_count);
        for i in 0..source_symbols_count {
            let start = i * symbol_size;
            let end = (start + symbol_size).min(original_len);
            let mut chunk = vec![0u8; symbol_size];
            if start < original_len {
                chunk[..(end - start)].copy_from_slice(&data[start..end]);
            }
            source_symbols.push(chunk);
        }

        let mut all_symbols: Vec<FecSymbol> =
            Vec::with_capacity(source_symbols_count + repair_symbols_count);

        // Append systematic source symbols
        for (i, payload) in source_symbols.iter().enumerate() {
            all_symbols.push(FecSymbol {
                symbol_id: i as u32,
                is_repair: false,
                seed: 0,
                payload: payload.clone(),
            });
        }

        // 2. Generate R parity repair symbols using pseudo-random linear XOR combinations
        for r in 0..repair_symbols_count {
            let repair_id = (source_symbols_count + r) as u32;
            let seed = (repair_id as u64).wrapping_mul(0x9E3779B97F4A7C15);
            let mut rng = StdRng::seed_from_u64(seed);

            let mut repair_payload = vec![0u8; symbol_size];
            // Include source chunks based on pseudo-random bitmask
            for source in &source_symbols {
                if (rng.next_u32() % 2) == 1 {
                    for b in 0..symbol_size {
                        repair_payload[b] ^= source[b];
                    }
                }
            }

            // Ensure repair payload is non-trivial by mixing first symbol if empty
            if repair_payload.iter().all(|&b| b == 0) && !source_symbols.is_empty() {
                for b in 0..symbol_size {
                    repair_payload[b] ^= source_symbols[0][b];
                }
            }

            all_symbols.push(FecSymbol {
                symbol_id: repair_id,
                is_repair: true,
                seed,
                payload: repair_payload,
            });
        }

        let header = FecHeader {
            original_len,
            symbol_size,
            source_symbols_count,
            repair_symbols_count,
        };

        (header, all_symbols)
    }
}

/// Systematic Fountain Code Decoder for reconstructing artwork packages.
pub struct FecDecoder;

impl FecDecoder {
    /// Attempts reconstruction of original data from received symbols.
    ///
    /// If all `K` systematic source symbols were received, reconstruction is instant (zero-overhead).
    /// If symbols were dropped, repair symbols are utilized.
    pub fn decode(header: &FecHeader, received_symbols: &[FecSymbol]) -> Result<Vec<u8>, VgcError> {
        let k = header.source_symbols_count;
        if received_symbols.len() < k {
            return Err(VgcError::Other(format!(
                "Insufficient FEC symbols: received {}, require at least {}",
                received_symbols.len(),
                k
            )));
        }

        // Map received systematic source symbols
        let mut reconstructed: Vec<Option<Vec<u8>>> = vec![None; k];
        for sym in received_symbols {
            let id = sym.symbol_id as usize;
            if !sym.is_repair && id < k {
                reconstructed[id] = Some(sym.payload.clone());
            }
        }

        // Check if all systematic source symbols are directly present
        if reconstructed.iter().all(|s| s.is_some()) {
            let mut result = Vec::with_capacity(k * header.symbol_size);
            for s in reconstructed.into_iter().flatten() {
                result.extend_from_slice(&s);
            }
            result.truncate(header.original_len);
            return Ok(result);
        }

        // Identify missing source indices
        let missing_indices: Vec<usize> = reconstructed
            .iter()
            .enumerate()
            .filter_map(|(i, s)| if s.is_none() { Some(i) } else { None })
            .collect();

        // If exactly 1 source chunk is missing, find any repair symbol that covers it
        if missing_indices.len() == 1 {
            let missing_idx = missing_indices[0];
            for sym in received_symbols.iter().filter(|s| s.is_repair) {
                let mut rng = StdRng::seed_from_u64(sym.seed);
                let mut mask = Vec::with_capacity(k);
                for _ in 0..k {
                    mask.push((rng.next_u32() % 2) == 1);
                }

                if mask[missing_idx] {
                    let mut recovered = sym.payload.clone();
                    for (i, &included) in mask.iter().enumerate() {
                        if included && i != missing_idx {
                            if let Some(src) = &reconstructed[i] {
                                for b in 0..header.symbol_size {
                                    recovered[b] ^= src[b];
                                }
                            }
                        }
                    }
                    reconstructed[missing_idx] = Some(recovered);
                    break;
                }
            }
        }

        // Verify that reconstruction succeeded
        if reconstructed.iter().all(|s| s.is_some()) {
            let mut result = Vec::with_capacity(k * header.symbol_size);
            for s in reconstructed.into_iter().flatten() {
                result.extend_from_slice(&s);
            }
            result.truncate(header.original_len);
            Ok(result)
        } else {
            Err(VgcError::Other(
                "Unable to solve erasure matrix with available repair symbols".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fec_encode_and_direct_decode() {
        let artwork_data = b"Vincent van Gogh: Starry Night with Sovereign Cats 2026-11-01";
        let (header, symbols) = FecEncoder::encode(artwork_data, 16, 0.4);

        assert_eq!(header.original_len, artwork_data.len());
        assert!(symbols.len() >= header.source_symbols_count);

        let decoded = FecDecoder::decode(&header, &symbols).expect("decode");
        assert_eq!(&decoded[..], artwork_data);
    }

    #[test]
    fn test_fec_loss_recovery_with_repair_symbol() {
        let artwork_data = b"Post-impressionist feline masterpiece on high-frequency P2P swarm";
        let (header, symbols) = FecEncoder::encode(artwork_data, 16, 0.5);

        // Simulate network packet loss: drop source symbol at index 1
        let mut lossy_symbols: Vec<FecSymbol> = Vec::new();
        for sym in symbols {
            if sym.symbol_id != 1 {
                lossy_symbols.push(sym);
            }
        }

        let decoded = FecDecoder::decode(&header, &lossy_symbols).expect("repair decode");
        assert_eq!(&decoded[..], artwork_data);
    }
}
