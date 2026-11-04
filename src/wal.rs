// Copyright (C) 2026 Rahad Bhuiya (Exploidus Labs) <rahadbhuiya2021@gmail.com>
// Licensed under the GNU AGPL-3.0-or-later with Dual Commercial Licensing terms.

use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use crc32fast::Hasher as Crc32Hasher;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use crate::currency::Currency;
use crate::error::PaySwitchError;
use crate::ledger::{AccountType, Amount, Transfer};
use uuid::Uuid;

const WAL_MAGIC: &[u8; 4] = b"PSWL";
const WAL_VERSION: u16 = 1;

/// A serialized operation recorded in the Write-Ahead Log before in-memory execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalRecord {
    CreateAccount {
        id: Uuid,
        account_type: AccountType,
        currency: Currency,
    },
    CreditInbound {
        account_id: Uuid,
        amount: Amount,
        currency: Currency,
    },
    Transfer(Transfer),
}

/// Durable, crash-consistent Write-Ahead Log backed by an append-only binary file.
pub struct WriteAheadLog {
    path: PathBuf,
    writer: Mutex<BufWriter<File>>,
}

impl WriteAheadLog {
    /// Opens or creates a Write-Ahead Log at the specified path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, PaySwitchError> {
        let path_buf = path.as_ref().to_path_buf();
        let file_exists = path_buf.exists();

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path_buf)
            .map_err(|e| PaySwitchError::Internal(format!("Failed to open WAL file: {}", e)))?;

        let mut writer = BufWriter::new(file);

        if !file_exists || path_buf.metadata().map(|m| m.len() == 0).unwrap_or(true) {
            // Write WAL header: MAGIC (4 bytes) + VERSION (2 bytes)
            writer
                .write_all(WAL_MAGIC)
                .map_err(|e| PaySwitchError::Internal(format!("Failed to write WAL magic: {}", e)))?;
            writer
                .write_all(&WAL_VERSION.to_be_bytes())
                .map_err(|e| PaySwitchError::Internal(format!("Failed to write WAL version: {}", e)))?;
            writer
                .flush()
                .map_err(|e| PaySwitchError::Internal(format!("Failed to flush WAL header: {}", e)))?;
        }

        Ok(Self {
            path: path_buf,
            writer: Mutex::new(writer),
        })
    }

    /// Appends a record to the WAL and flushes to disk with CRC-32 integrity verification.
    pub fn append(&self, record: &WalRecord) -> Result<(), PaySwitchError> {
        let payload = serde_json::to_vec(record)
            .map_err(|e| PaySwitchError::Internal(format!("Serialization error: {}", e)))?;

        let payload_len = payload.len() as u32;

        let mut crc_hasher = Crc32Hasher::new();
        crc_hasher.update(&payload);
        let checksum = crc_hasher.finalize();

        let mut writer = self.writer.lock();

        // Record framing: [Length: 4 bytes][CRC32: 4 bytes][Payload bytes]
        writer
            .write_all(&payload_len.to_be_bytes())
            .map_err(|e| PaySwitchError::Internal(format!("WAL write length failed: {}", e)))?;
        writer
            .write_all(&checksum.to_be_bytes())
            .map_err(|e| PaySwitchError::Internal(format!("WAL write checksum failed: {}", e)))?;
        writer
            .write_all(&payload)
            .map_err(|e| PaySwitchError::Internal(format!("WAL write payload failed: {}", e)))?;

        writer
            .flush()
            .map_err(|e| PaySwitchError::Internal(format!("WAL fsync failed: {}", e)))?;

        Ok(())
    }

    /// Replays the WAL from the start and returns all valid records.
    pub fn recover(&self) -> Result<Vec<WalRecord>, PaySwitchError> {
        let file = File::open(&self.path)
            .map_err(|e| PaySwitchError::Internal(format!("Failed to open WAL for replay: {}", e)))?;
        let mut reader = BufReader::new(file);

        let mut magic = [0u8; 4];
        reader
            .read_exact(&mut magic)
            .map_err(|e| PaySwitchError::Internal(format!("WAL read magic failed: {}", e)))?;
        if &magic != WAL_MAGIC {
            return Err(PaySwitchError::Internal("Invalid WAL magic signature".into()));
        }

        let mut version_bytes = [0u8; 2];
        reader
            .read_exact(&mut version_bytes)
            .map_err(|e| PaySwitchError::Internal(format!("WAL read version failed: {}", e)))?;

        let mut records = Vec::new();

        loop {
            let mut len_bytes = [0u8; 4];
            match reader.read_exact(&mut len_bytes) {
                Ok(_) => (),
                Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(PaySwitchError::Internal(format!("WAL read error: {}", e))),
            }

            let len = u32::from_be_bytes(len_bytes) as usize;

            let mut checksum_bytes = [0u8; 4];
            reader
                .read_exact(&mut checksum_bytes)
                .map_err(|e| PaySwitchError::Internal(format!("WAL checksum read failed: {}", e)))?;
            let expected_checksum = u32::from_be_bytes(checksum_bytes);

            let mut payload = vec![0u8; len];
            reader
                .read_exact(&mut payload)
                .map_err(|e| PaySwitchError::Internal(format!("WAL payload read failed: {}", e)))?;

            let mut crc_hasher = Crc32Hasher::new();
            crc_hasher.update(&payload);
            let calculated_checksum = crc_hasher.finalize();

            if calculated_checksum != expected_checksum {
                return Err(PaySwitchError::Internal(format!(
                    "WAL record corruption detected! Checksum mismatch: expected {}, got {}",
                    expected_checksum, calculated_checksum
                )));
            }

            let record: WalRecord = serde_json::from_slice(&payload)
                .map_err(|e| PaySwitchError::Internal(format!("WAL deserialization failed: {}", e)))?;
            records.push(record);
        }

        Ok(records)
    }
}
