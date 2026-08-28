use crate::config::{
    TargetProcess, is_normalized_process_name, is_supported_normalized_target_process_name,
    target_index_by_identity,
};
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
    pub(super) ownership_incomplete: bool,
    pub(super) target_mute_states: Option<Vec<HandoffTargetMuteState>>,
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
    #[serde(default)]
    ownership_incomplete: bool,
    sessions: Vec<WireSession>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    target_mute_states: Option<Vec<HandoffTargetMuteState>>,
}

#[derive(Deserialize, Serialize)]
struct WireSession {
    pid: u32,
    process_name: String,
    instance_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(super) struct HandoffTargetMuteState {
    process_name: String,
    pid: Option<u32>,
    muted: bool,
}

impl HandoffSnapshot {
    pub(super) fn capture(
        paused: bool,
        managed_sessions: &HashSet<AudioSessionKey>,
        dirty_targets: Option<&[TargetProcess]>,
    ) -> Self {
        let mut ownership_incomplete = false;
        let managed_sessions = managed_sessions
            .iter()
            .filter_map(|key| match key.instance_id.as_deref() {
                Some(instance_id) if !instance_id.is_empty() => Some(key.clone()),
                _ => {
                    // A coarse key depends on in-process COM identity, which cannot distinguish
                    // sibling sessions after the old process exits.
                    ownership_incomplete = true;
                    None
                }
            })
            .collect();
        Self {
            paused,
            managed_sessions,
            ownership_incomplete,
            target_mute_states: dirty_targets.map(|targets| {
                targets
                    .iter()
                    .map(|target| HandoffTargetMuteState {
                        process_name: target.name.clone(),
                        pid: target.pid,
                        muted: target.managed_muted,
                    })
                    .collect()
            }),
        }
    }

    pub(super) fn encode(&self) -> Option<Vec<u8>> {
        let sessions = self
            .managed_sessions
            .iter()
            .filter_map(|key| {
                let instance_id = key.instance_id.as_ref()?;
                (!instance_id.is_empty()).then(|| WireSession {
                    pid: key.pid,
                    process_name: key.process_name.clone(),
                    instance_id: instance_id.clone(),
                })
            })
            .collect();
        let bytes = serde_json::to_vec(&WireSnapshot {
            version: PROTOCOL_VERSION,
            paused: self.paused,
            ownership_incomplete: self.ownership_incomplete,
            sessions,
            target_mute_states: self.target_mute_states.clone(),
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
        if wire.target_mute_states.as_ref().is_some_and(|states| {
            states.iter().any(|state| {
                !is_supported_normalized_target_process_name(&state.process_name)
                    || state.pid == Some(0)
            })
        }) {
            return None;
        }
        Some(Self {
            paused: wire.paused,
            managed_sessions,
            ownership_incomplete: wire.ownership_incomplete,
            target_mute_states: wire.target_mute_states,
        })
    }
}

pub(super) fn apply_target_mute_states(
    targets: &mut [TargetProcess],
    states: &[HandoffTargetMuteState],
) -> bool {
    let mut changed = false;
    for state in states {
        let Some(index) = target_index_by_identity(targets, &state.process_name, state.pid) else {
            continue;
        };
        changed |= targets[index].managed_muted != state.muted;
        targets[index].managed_muted = state.muted;
    }
    changed
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
        let snapshot = HandoffSnapshot::capture(true, &sessions, None);

        let decoded = HandoffSnapshot::decode(&snapshot.encode().unwrap()).unwrap();

        assert!(decoded.paused);
        assert_eq!(decoded.managed_sessions, sessions);
        assert!(!decoded.ownership_incomplete);
        assert!(decoded.target_mute_states.is_none());
    }

    #[test]
    fn handoff_preserves_exact_ownership_and_marks_coarse_sessions_for_recovery() {
        let mut chat = TargetProcess::new("chat.exe").unwrap();
        chat.managed_muted = true;
        let dirty_targets = [chat];
        for instance_id in [None, Some(String::new())] {
            let exact = AudioSessionKey::new(42, "game.exe", Some("session-a".to_owned())).unwrap();
            let sessions = HashSet::from([
                exact.clone(),
                AudioSessionKey::new(77, "chat.exe", instance_id).unwrap(),
            ]);
            let snapshot = HandoffSnapshot::capture(true, &sessions, Some(&dirty_targets));

            let decoded = HandoffSnapshot::decode(&snapshot.encode().unwrap()).unwrap();

            assert!(decoded.paused);
            assert_eq!(decoded.managed_sessions, HashSet::from([exact]));
            assert!(decoded.ownership_incomplete);
            assert_eq!(
                decoded.target_mute_states,
                Some(vec![HandoffTargetMuteState {
                    process_name: "chat.exe".to_owned(),
                    pid: None,
                    muted: true,
                }])
            );
        }
    }

    #[test]
    fn transferred_target_mute_states_override_only_matching_targets() {
        let mut source_chat = TargetProcess::new("chat.exe").unwrap();
        let mut source_game = TargetProcess::new("game.exe").unwrap();
        source_game.managed_muted = true;
        let mut source_removed = TargetProcess::new("removed.exe").unwrap();
        source_removed.managed_muted = true;
        let source = [source_chat.clone(), source_game.clone(), source_removed];
        let snapshot = HandoffSnapshot::capture(false, &HashSet::new(), Some(&source));
        let decoded = HandoffSnapshot::decode(&snapshot.encode().unwrap()).unwrap();

        source_chat.managed_muted = true;
        source_game.managed_muted = false;
        let music = TargetProcess::new("music.exe").unwrap();
        let mut destination = [source_chat, source_game, music];
        let states = decoded.target_mute_states.unwrap();

        assert!(apply_target_mute_states(&mut destination, &states));
        assert!(!destination[0].managed_muted);
        assert!(destination[1].managed_muted);
        assert!(!destination[2].managed_muted);
        assert!(!apply_target_mute_states(&mut destination, &states));
    }

    #[test]
    fn handoff_accepts_snapshot_without_recovery_hint() {
        let decoded =
            HandoffSnapshot::decode(br#"{"version":1,"paused":true,"sessions":[]}"#).unwrap();

        assert!(decoded.paused);
        assert!(decoded.managed_sessions.is_empty());
        assert!(!decoded.ownership_incomplete);
        assert!(decoded.target_mute_states.is_none());
    }

    #[test]
    fn handoff_state_accepts_one_snapshot_from_the_expected_source() {
        let mut source_token = 0u8;
        let mut other_token = 0u8;
        let source = HWND(std::ptr::from_mut(&mut source_token).cast());
        let other = HWND(std::ptr::from_mut(&mut other_token).cast());
        let mut state = HandoffState::new(Some(source));

        assert!(!state.accept(
            other,
            HandoffSnapshot::capture(false, &HashSet::new(), None)
        ));
        assert!(state.accept(
            source,
            HandoffSnapshot::capture(true, &HashSet::new(), None)
        ));
        assert!(!state.accept(
            source,
            HandoffSnapshot::capture(false, &HashSet::new(), None)
        ));
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
            br#"{"version":2,"paused":false,"ownership_incomplete":false,"sessions":[]}"#.as_slice(),
            br#"{"version":1,"paused":false,"ownership_incomplete":false,"sessions":[{"pid":0,"process_name":"game.exe","instance_id":"session-a"}]}"#,
            br#"{"version":1,"paused":false,"ownership_incomplete":false,"sessions":[{"pid":1,"process_name":"GAME.EXE","instance_id":"session-a"}]}"#,
            br#"{"version":1,"paused":false,"ownership_incomplete":false,"sessions":[{"pid":1,"process_name":"game.exe","instance_id":null}]}"#,
            br#"{"version":1,"paused":false,"ownership_incomplete":false,"sessions":[{"pid":1,"process_name":"game.exe","instance_id":""}]}"#,
            br#"{"version":1,"paused":false,"ownership_incomplete":false,"sessions":[],"target_mute_states":[{"process_name":"GAME.EXE","pid":null,"muted":true}]}"#,
        ] {
            assert!(HandoffSnapshot::decode(bytes).is_none());
        }
    }
}
