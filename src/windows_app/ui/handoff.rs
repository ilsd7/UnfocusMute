use crate::config::is_normalized_process_name;
use crate::engine::AudioSessionKey;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use windows::Win32::Foundation::HWND;

pub(super) const COPYDATA_ID: usize = 0x5546_4d48;
pub(super) const MAX_BYTES: usize = 1024 * 1024;
const PROTOCOL_VERSION: u32 = 1;

pub(super) struct HandoffSnapshot {
    pub(super) paused: bool,
    pub(super) managed_sessions: HashSet<AudioSessionKey>,
}

pub(super) enum HandoffState {
    Running,
    Receiving {
        source: HWND,
        snapshot: Option<HandoffSnapshot>,
    },
    Prepared,
}

impl HandoffState {
    pub(super) fn new(source: Option<HWND>) -> Self {
        match source {
            Some(source) => Self::Receiving {
                source,
                snapshot: None,
            },
            None => Self::Running,
        }
    }

    pub(super) fn accept(&mut self, source: HWND, received: HandoffSnapshot) -> bool {
        let Self::Receiving {
            source: expected,
            snapshot,
        } = self
        else {
            return false;
        };
        if expected.0 != source.0 || snapshot.is_some() {
            return false;
        }
        *snapshot = Some(received);
        true
    }

    pub(super) fn take_received(&mut self) -> Option<HandoffSnapshot> {
        let received = match self {
            Self::Receiving { snapshot, .. } => snapshot.take(),
            _ => return None,
        };
        *self = Self::Running;
        received
    }

    pub(super) fn prepare(&mut self) -> bool {
        if !matches!(self, Self::Running) {
            return false;
        }
        *self = Self::Prepared;
        true
    }

    pub(super) fn leave_prepared(&mut self) -> bool {
        if !matches!(self, Self::Prepared) {
            return false;
        }
        *self = Self::Running;
        true
    }

    pub(super) fn is_prepared(&self) -> bool {
        matches!(self, Self::Prepared)
    }
}

#[derive(Deserialize, Serialize)]
struct WireSnapshot {
    version: u32,
    paused: bool,
    sessions: Vec<WireSession>,
}

#[derive(Deserialize, Serialize)]
struct WireSession {
    pid: u32,
    process_name: String,
    instance_id: String,
}

impl HandoffSnapshot {
    pub(super) fn capture(paused: bool, managed_sessions: &HashSet<AudioSessionKey>) -> Self {
        Self {
            paused,
            managed_sessions: managed_sessions.clone(),
        }
    }

    pub(super) fn encode(&self) -> Option<Vec<u8>> {
        let sessions = self
            .managed_sessions
            .iter()
            .map(|key| {
                let instance_id = key.instance_id.as_ref()?.clone();
                (!instance_id.is_empty()).then_some(WireSession {
                    pid: key.pid,
                    process_name: key.process_name.clone(),
                    instance_id,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        let bytes = serde_json::to_vec(&WireSnapshot {
            version: PROTOCOL_VERSION,
            paused: self.paused,
            sessions,
        })
        .ok()?;
        (bytes.len() <= MAX_BYTES).then_some(bytes)
    }

    pub(super) fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() || bytes.len() > MAX_BYTES {
            return None;
        }
        let wire: WireSnapshot = serde_json::from_slice(bytes).ok()?;
        if wire.version != PROTOCOL_VERSION {
            return None;
        }

        let mut managed_sessions = HashSet::with_capacity(wire.sessions.len());
        for session in wire.sessions {
            if session.pid == 0
                || !is_normalized_process_name(&session.process_name)
                || session.instance_id.is_empty()
            {
                return None;
            }
            managed_sessions.insert(AudioSessionKey::from_normalized(
                session.pid,
                session.process_name,
                Some(session.instance_id),
            ));
        }
        Some(Self {
            paused: wire.paused,
            managed_sessions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handoff_round_trip_preserves_pause_and_exact_ownership() {
        let sessions = HashSet::from([
            AudioSessionKey::new(42, "game.exe", Some("session-a".to_owned())).unwrap(),
            AudioSessionKey::new(77, "오디오.exe", Some("session-b".to_owned())).unwrap(),
        ]);
        let snapshot = HandoffSnapshot::capture(true, &sessions);

        let decoded = HandoffSnapshot::decode(&snapshot.encode().unwrap()).unwrap();

        assert!(decoded.paused);
        assert_eq!(decoded.managed_sessions, sessions);
    }

    #[test]
    fn handoff_requires_an_exact_instance_id_for_every_managed_session() {
        for instance_id in [None, Some(String::new())] {
            let sessions = HashSet::from([
                AudioSessionKey::new(42, "game.exe", Some("session-a".to_owned())).unwrap(),
                AudioSessionKey::new(77, "chat.exe", instance_id).unwrap(),
            ]);

            assert!(
                HandoffSnapshot::capture(false, &sessions)
                    .encode()
                    .is_none()
            );
        }
    }

    #[test]
    fn handoff_state_accepts_one_snapshot_from_the_expected_source() {
        let mut source_token = 0u8;
        let mut other_token = 0u8;
        let source = HWND(std::ptr::from_mut(&mut source_token).cast());
        let other = HWND(std::ptr::from_mut(&mut other_token).cast());
        let mut state = HandoffState::new(Some(source));

        assert!(!state.accept(other, HandoffSnapshot::capture(false, &HashSet::new())));
        assert!(state.accept(source, HandoffSnapshot::capture(true, &HashSet::new())));
        assert!(!state.accept(source, HandoffSnapshot::capture(false, &HashSet::new())));
        assert!(
            state
                .take_received()
                .is_some_and(|snapshot| snapshot.paused)
        );
        assert!(matches!(state, HandoffState::Running));
    }

    #[test]
    fn prepared_handoff_has_one_exit_transition() {
        let mut state = HandoffState::new(None);

        assert!(state.prepare());
        assert!(!state.prepare());
        assert!(state.leave_prepared());
        assert!(!state.leave_prepared());
    }

    #[test]
    fn handoff_rejects_unknown_protocol_and_invalid_session_identity() {
        for bytes in [
            br#"{"version":2,"paused":false,"sessions":[]}"#.as_slice(),
            br#"{"version":1,"paused":false,"sessions":[{"pid":0,"process_name":"game.exe","instance_id":"session-a"}]}"#,
            br#"{"version":1,"paused":false,"sessions":[{"pid":1,"process_name":"GAME.EXE","instance_id":"session-a"}]}"#,
            br#"{"version":1,"paused":false,"sessions":[{"pid":1,"process_name":"game.exe","instance_id":null}]}"#,
            br#"{"version":1,"paused":false,"sessions":[{"pid":1,"process_name":"game.exe","instance_id":""}]}"#,
        ] {
            assert!(HandoffSnapshot::decode(bytes).is_none());
        }
    }
}
