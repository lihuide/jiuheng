use super::{Storage, StorageError};
use chrono::Utc;
use divination_core::*;
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum TaiyiRequest {
    AccumulatedYear {
        count: u64,
    },
    TraditionalYear {
        year: i32,
        profile: TaiyiYearEpochProfile,
    },
    ExplicitBoard {
        count: u64,
        dun: TaiyiDun,
    },
}

impl TaiyiRequest {
    pub fn label(&self) -> String {
        match self {
            Self::AccumulatedYear { count } => format!("年计 · 明确积年 {count}"),
            Self::TraditionalYear { year, .. } => {
                format!("年计 · 传统年标签 {year} · 开元十二年锚点候选")
            }
            Self::ExplicitBoard { count, dun } => {
                format!("七十二局 · {} · 明确局序 {count}", dun.zh_name())
            }
        }
    }

    pub fn calculate(&self) -> Result<TaiyiCalculation, StorageError> {
        match *self {
            Self::AccumulatedYear { count } => taiyi_annual_frame_from_accumulated_count(count)
                .map(|frame| TaiyiCalculation::Annual(Box::new(frame)))
                .map_err(|e| invalid(&e.to_string())),
            Self::TraditionalYear { year, profile } => {
                taiyi_annual_frame_from_traditional_year_candidate(year, profile)
                    .map(|frame| TaiyiCalculation::Traditional(Box::new(frame)))
                    .map_err(|e| invalid(&e.to_string()))
            }
            Self::ExplicitBoard { count, dun } => {
                taiyi_seventy_two_board_frame_from_count(count, dun)
                    .map(|frame| TaiyiCalculation::Board(Box::new(frame)))
                    .map_err(|e| invalid(&e.to_string()))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "frame", rename_all = "snake_case")]
pub enum TaiyiCalculation {
    Annual(Box<TaiyiAnnualFrame>),
    Traditional(Box<TaiyiTraditionalYearEpochCandidate>),
    Board(Box<TaiyiSeventyTwoBoardFrame>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaiyiRecord {
    pub version: u8,
    pub id: Uuid,
    pub question_id: Uuid,
    pub created_at: String,
    pub request: TaiyiRequest,
    pub calculation: TaiyiCalculation,
}

fn invalid(message: &str) -> StorageError {
    StorageError::TaiyiRecord(message.into())
}

fn request_hash(request: &TaiyiRequest) -> Result<String, StorageError> {
    Ok(blake3::hash(serde_json::to_string(request)?.as_bytes())
        .to_hex()
        .to_string())
}

fn load(connection: &Connection, question_id: Uuid) -> Result<Vec<TaiyiRecord>, StorageError> {
    let mut query = connection.prepare("SELECT id, request_hash, snapshot_json, content_hash, created_at FROM taiyi_candidates WHERE question_id = ?1 ORDER BY created_at, id")?;
    let rows = query.query_map([question_id.to_string()], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
        ))
    })?;
    let mut records = Vec::new();
    for row in rows {
        let (id, input_hash, json, hash, created_at) = row?;
        if hash != blake3::hash(json.as_bytes()).to_hex().as_str() {
            return Err(invalid("保存内容校验失败，已停止读取"));
        }
        let record: TaiyiRecord = serde_json::from_str(&json)?;
        if record.version != 1
            || record.id.to_string() != id
            || record.question_id != question_id
            || record.created_at != created_at
            || request_hash(&record.request)? != input_hash
            || record.request.calculate()? != record.calculation
        {
            return Err(invalid("记录身份、版本或原参数复算不一致，已停止读取"));
        }
        records.push(record);
    }
    Ok(records)
}

impl Storage {
    pub fn load_taiyi_candidates(
        &self,
        question_id: Uuid,
    ) -> Result<Vec<TaiyiRecord>, StorageError> {
        load(&self.connection, question_id)
    }

    pub fn save_taiyi_candidate(
        &mut self,
        question_id: Uuid,
        request: TaiyiRequest,
    ) -> Result<TaiyiRecord, StorageError> {
        let calculation = request.calculate()?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let locked: Option<bool> = tx
            .query_row(
                "SELECT locked FROM questions WHERE id = ?1",
                [question_id.to_string()],
                |row| row.get(0),
            )
            .optional()?;
        if locked != Some(true) {
            return Err(invalid("请先锁定占问记录"));
        }
        if let Some(existing) = load(&tx, question_id)?
            .into_iter()
            .find(|r| r.request == request)
        {
            tx.commit()?;
            return Ok(existing);
        }
        let record = TaiyiRecord {
            version: 1,
            id: Uuid::new_v4(),
            question_id,
            created_at: Utc::now().to_rfc3339(),
            request,
            calculation,
        };
        let json = serde_json::to_string(&record)?;
        tx.execute("INSERT INTO taiyi_candidates(id, question_id, request_hash, snapshot_json, content_hash, created_at) VALUES(?1, ?2, ?3, ?4, ?5, ?6)", params![record.id.to_string(), question_id.to_string(), request_hash(&record.request)?, json, blake3::hash(json.as_bytes()).to_hex().to_string(), record.created_at])?;
        tx.commit()?;
        Ok(record)
    }
}

#[cfg(test)]
#[path = "taiyi_records_tests.rs"]
mod tests;
