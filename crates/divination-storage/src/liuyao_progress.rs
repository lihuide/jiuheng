use super::{Storage, StorageError};
use chrono::Utc;
use divination_core::{EntropyEvent, LiuyaoTranscript};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use uuid::Uuid;

fn invalid(message: &str) -> StorageError {
    StorageError::ArtifactIntegrity(message.to_owned())
}

fn load(
    connection: &Connection,
    question_id: Uuid,
) -> Result<Option<LiuyaoTranscript>, StorageError> {
    let mut query = connection.prepare(
        "SELECT event_count, transcript_json, content_hash FROM liuyao_checkpoints
         WHERE question_id = ?1 ORDER BY event_count",
    )?;
    let rows = query.query_map([question_id.to_string()], |row| {
        Ok((
            row.get::<_, usize>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;
    let mut previous: Option<LiuyaoTranscript> = None;
    for (expected_count, row) in rows.enumerate() {
        let (count, json, hash) = row?;
        if count != expected_count || hash != blake3::hash(json.as_bytes()).to_hex().as_str() {
            return Err(invalid(
                "六爻续接记录不连续或内容校验失败，原始记录未被修改",
            ));
        }
        let transcript: LiuyaoTranscript = serde_json::from_str(&json)?;
        if transcript.question_id != question_id || transcript.events.len() != count {
            return Err(invalid("六爻续接记录与问题或次数不一致"));
        }
        transcript
            .replay_lines()
            .map_err(|error| invalid(&error.to_string()))?;
        if let Some(prior) = &previous {
            if transcript.session_nonce_hex != prior.session_nonce_hex
                || transcript.algorithm_version != prior.algorithm_version
                || !transcript.events.starts_with(&prior.events)
            {
                return Err(invalid("六爻续接记录改变了已经保存的爻"));
            }
        }
        previous = Some(transcript);
    }
    if let Some(transcript) = &previous {
        if transcript.is_complete() {
            let final_json: Option<(String, String)> = connection.query_row(
                "SELECT entropy_events_json, generated_values_json FROM random_artifacts WHERE question_id = ?1 AND method = 'liuyao'",
                [question_id.to_string()], |row| Ok((row.get(0)?, row.get(1)?)),
            ).optional()?;
            let Some((saved_transcript, saved_cast)) = final_json else {
                return Err(invalid("完整续接记录缺少最终排盘，已暂停继续"));
            };
            if serde_json::from_str::<LiuyaoTranscript>(&saved_transcript)? != *transcript
                || serde_json::from_str::<divination_core::LiuyaoCast>(&saved_cast)?
                    != transcript
                        .finish()
                        .map_err(|error| invalid(&error.to_string()))?
            {
                return Err(invalid("最终排盘与续接记录不一致"));
            }
        }
    }
    Ok(previous)
}

fn insert(connection: &Connection, transcript: &LiuyaoTranscript) -> Result<(), StorageError> {
    let json = serde_json::to_string(transcript)?;
    connection.execute(
        "INSERT INTO liuyao_checkpoints(question_id, event_count, transcript_json, content_hash, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![transcript.question_id.to_string(), transcript.events.len(), json,
            blake3::hash(json.as_bytes()).to_hex().to_string(), Utc::now().to_rfc3339()],
    )?;
    Ok(())
}

impl Storage {
    pub fn load_liuyao_progress(
        &self,
        question_id: Uuid,
    ) -> Result<Option<LiuyaoTranscript>, StorageError> {
        load(&self.connection, question_id)
    }

    pub fn begin_or_resume_liuyao(
        &mut self,
        question_id: Uuid,
    ) -> Result<LiuyaoTranscript, StorageError> {
        if let Some(artifact) = self.load_liuyao_artifact(question_id)? {
            if let Some(progress) = self.load_liuyao_progress(question_id)? {
                if progress != artifact.transcript {
                    return Err(invalid("已完成排盘与续接记录不一致"));
                }
            }
            return Ok(artifact.transcript);
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(transcript) = load(&tx, question_id)? {
            tx.commit()?;
            return Ok(transcript);
        }
        let locked: Option<bool> = tx
            .query_row(
                "SELECT locked FROM questions WHERE id = ?1",
                [question_id.to_string()],
                |row| row.get(0),
            )
            .optional()?;
        if locked != Some(true) {
            return Err(invalid("只能继续已经锁定的问题"));
        }
        let transcript = LiuyaoTranscript::with_random_nonce(question_id);
        insert(&tx, &transcript)?;
        tx.commit()?;
        Ok(transcript)
    }

    pub fn append_liuyao_event(
        &mut self,
        expected: &LiuyaoTranscript,
        event: &EntropyEvent,
    ) -> Result<LiuyaoTranscript, StorageError> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut transcript = load(&tx, expected.question_id)?
            .ok_or_else(|| invalid("六爻会话尚未保存，请重新打开这条记录"))?;
        if transcript != *expected {
            let mut retried = expected.clone();
            retried
                .push_event(event.clone())
                .map_err(|error| invalid(&error.to_string()))?;
            if transcript == retried {
                tx.commit()?;
                return Ok(transcript);
            }
            return Err(invalid("这条记录已在其他窗口更新，请重新读取已保存进度"));
        }
        transcript
            .push_event(event.clone())
            .map_err(|error| invalid(&error.to_string()))?;
        insert(&tx, &transcript)?;
        if transcript.is_complete() {
            let cast = transcript
                .finish()
                .map_err(|error| invalid(&error.to_string()))?;
            tx.execute(
                "INSERT INTO random_artifacts(question_id, method, algorithm, algorithm_version,
                    seed_hex, entropy_events_json, generated_values_json, created_at)
                 VALUES (?1, 'liuyao', 'three_coins', ?2, ?3, ?4, ?5, ?6)",
                params![
                    expected.question_id.to_string(),
                    transcript.algorithm_version,
                    cast.master_seed_hex,
                    serde_json::to_string(&transcript)?,
                    serde_json::to_string(&cast)?,
                    Utc::now().to_rfc3339()
                ],
            )?;
        }
        tx.commit()?;
        Ok(transcript)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{entropy_event, sample_question};
    use divination_core::{build_event_fingerprint, KeywordClassifier};

    fn locked(storage: &mut Storage) -> Uuid {
        let question = sample_question();
        let classification = KeywordClassifier::default()
            .classify(question.category, &question.text)
            .unwrap();
        storage
            .insert_locked_question(
                &question,
                &classification,
                &build_event_fingerprint(&question, &classification),
                None,
            )
            .unwrap();
        question.id
    }

    fn database_path() -> std::path::PathBuf {
        let folder = std::env::temp_dir().join(format!("jiuheng-resume-test-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&folder).unwrap();
        folder.join("test.sqlite3")
    }

    #[test]
    fn resumes_every_saved_prefix_after_reopen_without_changing_any_line() {
        let path = database_path();
        let (id, mut expected) = {
            let mut storage = Storage::open(&path).unwrap();
            let id = locked(&mut storage);
            (id, storage.begin_or_resume_liuyao(id).unwrap())
        };
        for sequence in 0..6 {
            let mut storage = Storage::open(&path).unwrap();
            let old = storage.begin_or_resume_liuyao(id).unwrap();
            assert_eq!(old, expected);
            let lines = old.replay_lines().unwrap();
            expected.push_event(entropy_event(sequence)).unwrap();
            assert_eq!(
                storage
                    .append_liuyao_event(&old, &entropy_event(sequence))
                    .unwrap(),
                expected
            );
            assert_eq!(
                storage
                    .append_liuyao_event(&old, &entropy_event(sequence))
                    .unwrap(),
                expected
            );
            assert_eq!(
                &expected.replay_lines().unwrap()[..sequence as usize],
                lines
            );
            let summary = storage.recent_questions(1).unwrap().remove(0);
            assert_eq!(summary.liuyao_saved_lines, sequence as usize + 1);
            assert_eq!(summary.has_liuyao, sequence == 5);
        }
        {
            let mut storage = Storage::open(&path).unwrap();
            assert_eq!(storage.begin_or_resume_liuyao(id).unwrap(), expected);
            assert_eq!(
                storage.load_liuyao_artifact(id).unwrap().unwrap().cast,
                expected.finish().unwrap()
            );
            assert!(storage
                .append_liuyao_event(&expected, &entropy_event(6))
                .is_err());
            let count: usize = storage
                .connection
                .query_row("SELECT COUNT(*) FROM random_artifacts", [], |row| {
                    row.get(0)
                })
                .unwrap();
            assert_eq!(count, 1);
        }
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn stale_window_cannot_replace_or_reroll_committed_progress() {
        let path = database_path();
        {
            let mut first = Storage::open(&path).unwrap();
            let id = locked(&mut first);
            let initial = first.begin_or_resume_liuyao(id).unwrap();
            let mut second = Storage::open(&path).unwrap();
            assert_eq!(second.begin_or_resume_liuyao(id).unwrap(), initial);
            let saved = first
                .append_liuyao_event(&initial, &entropy_event(0))
                .unwrap();
            let mut changed = entropy_event(0);
            changed.press_duration_nanos += 1;
            assert!(second.append_liuyao_event(&initial, &changed).is_err());
            assert_eq!(second.begin_or_resume_liuyao(id).unwrap(), saved);
            assert!(first.begin_or_resume_liuyao(Uuid::new_v4()).is_err());
            let mut reroll = LiuyaoTranscript::with_random_nonce(id);
            for sequence in 0..6 {
                reroll.push_event(entropy_event(sequence)).unwrap();
            }
            assert!(first
                .save_liuyao_artifact(id, &reroll, &reroll.finish().unwrap())
                .is_err());
            assert!(first
                .connection
                .execute(
                    "DELETE FROM liuyao_checkpoints WHERE question_id = ?1",
                    [id.to_string()]
                )
                .is_err());
            assert!(first
                .connection
                .execute(
                    "UPDATE liuyao_checkpoints SET content_hash = 'x' WHERE question_id = ?1",
                    [id.to_string()]
                )
                .is_err());
            assert_eq!(first.load_liuyao_progress(id).unwrap().unwrap(), saved);
        }
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn failed_final_save_rolls_back_and_same_operation_can_be_retried() {
        let mut storage = Storage::open_in_memory().unwrap();
        let id = locked(&mut storage);
        let mut saved = storage.begin_or_resume_liuyao(id).unwrap();
        for sequence in 0..5 {
            saved = storage
                .append_liuyao_event(&saved, &entropy_event(sequence))
                .unwrap();
        }
        storage.connection.execute_batch("CREATE TRIGGER simulated_failure BEFORE INSERT ON random_artifacts BEGIN SELECT RAISE(ABORT, 'simulated disk failure'); END;").unwrap();
        assert!(storage
            .append_liuyao_event(&saved, &entropy_event(5))
            .is_err());
        assert_eq!(storage.load_liuyao_progress(id).unwrap().unwrap(), saved);
        assert!(storage.load_liuyao_artifact(id).unwrap().is_none());
        storage
            .connection
            .execute_batch("DROP TRIGGER simulated_failure;")
            .unwrap();
        let completed = storage
            .append_liuyao_event(&saved, &entropy_event(5))
            .unwrap();
        assert_eq!(
            storage
                .load_liuyao_artifact(id)
                .unwrap()
                .unwrap()
                .transcript,
            completed
        );
        assert_eq!(
            storage
                .append_liuyao_event(&saved, &entropy_event(5))
                .unwrap(),
            completed
        );
    }

    #[test]
    fn damaged_checkpoint_is_rejected_without_resetting_saved_material() {
        let mut storage = Storage::open_in_memory().unwrap();
        let id = locked(&mut storage);
        let initial = storage.begin_or_resume_liuyao(id).unwrap();
        let saved = storage
            .append_liuyao_event(&initial, &entropy_event(0))
            .unwrap();
        storage
            .connection
            .execute_batch("DROP TRIGGER prevent_liuyao_checkpoint_update;")
            .unwrap();
        storage.connection.execute("UPDATE liuyao_checkpoints SET content_hash = 'damaged' WHERE question_id = ?1 AND event_count = 1", [id.to_string()]).unwrap();
        assert!(storage.begin_or_resume_liuyao(id).is_err());
        assert!(storage
            .append_liuyao_event(&saved, &entropy_event(1))
            .is_err());
        let json: String = storage.connection.query_row("SELECT transcript_json FROM liuyao_checkpoints WHERE question_id = ?1 AND event_count = 1", [id.to_string()], |row| row.get(0)).unwrap();
        assert_eq!(
            serde_json::from_str::<LiuyaoTranscript>(&json).unwrap(),
            saved
        );
        let mut changed = saved.clone();
        changed.session_nonce_hex = "aa".repeat(32);
        let json = serde_json::to_string(&changed).unwrap();
        storage.connection.execute("UPDATE liuyao_checkpoints SET transcript_json = ?1, content_hash = ?2 WHERE question_id = ?3 AND event_count = 1", params![json, blake3::hash(json.as_bytes()).to_hex().to_string(), id.to_string()]).unwrap();
        assert!(storage.begin_or_resume_liuyao(id).is_err());
    }

    #[test]
    fn upgrades_v12_database_and_preserves_legacy_completed_and_locked_records() {
        let path = database_path();
        let (id, completed, pending_id) = {
            let mut storage = Storage::open(&path).unwrap();
            let id = locked(&mut storage);
            let mut completed = LiuyaoTranscript::new(id, [42; 32]);
            for sequence in 0..6 {
                completed.push_event(entropy_event(sequence)).unwrap();
            }
            storage
                .save_liuyao_artifact(id, &completed, &completed.finish().unwrap())
                .unwrap();
            let pending_id = locked(&mut storage);
            storage.connection.execute_batch("DROP TRIGGER enforce_liuyao_checkpoint_finalization; DROP TABLE liuyao_checkpoints; DELETE FROM schema_migrations WHERE version = 13;").unwrap();
            (id, completed, pending_id)
        };
        {
            let mut upgraded = Storage::open(&path).unwrap();
            assert_eq!(upgraded.begin_or_resume_liuyao(id).unwrap(), completed);
            assert!(upgraded.load_liuyao_progress(id).unwrap().is_none());
            assert_eq!(
                upgraded
                    .begin_or_resume_liuyao(pending_id)
                    .unwrap()
                    .events
                    .len(),
                0
            );
            assert_eq!(upgraded.question_count().unwrap(), 2);
            let schema: u32 = upgraded
                .connection
                .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                    row.get(0)
                })
                .unwrap();
            assert_eq!(schema, 14);
        }
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
