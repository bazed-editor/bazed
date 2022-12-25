use std::collections::BTreeSet;

/// Manages an undo history, including undo groupings and redo.
///
/// # Behavior in a sample editing history
/// ```ignore
/// initial           => id = 1, history = [0            ], idx = 0, undone = {}
/// edit[new_group=t] => id = 1, history = [0, 1         ], idx = 1, undone = {}
/// edit[new_group=f] => id = 1, history = [0, 1         ], idx = 1, undone = {}
/// edit[new_group=t] => id = 2, history = [0, 1, 2      ], idx = 2, undone = {}
/// edit[new_group=f] => id = 2, history = [0, 1, 2      ], idx = 2, undone = {}
/// edit[new_group=t] => id = 3, history = [0, 1, 2, 3   ], idx = 3, undone = {}
/// undo              => id = 3, history = [0, 1, 2, 3   ], idx = 2, undone = { 3 }
/// undo              => id = 3, history = [0, 1, 2, 3   ], idx = 1, undone = { 3, 2 }
/// undo              => id = 3, history = [0, 1, 2, 3   ], idx = 1, undone = { 3, 2 }
/// redo              => id = 3, history = [0, 1, 2, 3   ], idx = 2, undone = { 3 }
/// edit[new_group=t] => id = 4, history = [0, 1, 2,    4], idx = 3, undone = { 3 }
/// ```
#[derive(Debug, PartialEq, Eq)]
pub(super) struct UndoHistory {
    /// The undo group id that current undos will be grouped under.
    ///
    /// This ID always just increments and will never reuse previous IDs, even if we undo and then do other edits.
    undo_gid: usize,
    /// The current position in the history. This is *not* an undo-group-id.
    /// Think of this as a cursor into time:
    /// - every id `history[n]` where `n > current_history_index` is in the future and may be redone to.
    /// - every id `history[n]` where `n < current_history_index` is in the past and may be undone to
    /// - when adding further edits, history gets truncated to end at the current index, and a new step gets added.
    ///
    /// **Invariant**: always < history.len().
    cur_history_idx: usize,
    /// List of undo groups in the history. See documentation of `cur_history_idx` for more details
    history: Vec<usize>,
    /// Set of undo groups that are currently undone.
    /// This may contain undo groups that are no longer part of the history due do
    /// us undoing and then performing edits.
    currently_undone: BTreeSet<usize>,
}

impl Default for UndoHistory {
    fn default() -> Self {
        Self {
            undo_gid: 0,
            cur_history_idx: 0,
            history: vec![0],
            currently_undone: Default::default(),
        }
    }
}

impl UndoHistory {
    /// If a new undo group should be created, creates a new undo group id,
    /// truncates any history-elements that are in the future and adds the new group to the history.
    /// Otherwise, just returns the current undo group id.
    pub(super) fn start_new_undo_group(&mut self) -> usize {
        self.undo_gid += 1;
        self.cur_history_idx += 1;
        self.history.truncate(self.cur_history_idx);
        self.history.push(self.undo_gid);
        tracing::trace!(undo_history = ?self, "Started a new undo group");
        self.undo_gid
    }

    /// Returns the current undo group id,
    /// or starts a new group if we're not currently at the end of the history
    pub(super) fn calculate_undo_id(&mut self) -> usize {
        if !self.at_end_of_history() {
            self.start_new_undo_group();
        }
        self.undo_gid
    }

    /// Check whether we're currently at the end of the history
    pub(super) fn at_end_of_history(&self) -> bool {
        self.cur_history_idx == self.history.len() - 1
    }

    pub(super) fn currently_undone(&self) -> &BTreeSet<usize> {
        &self.currently_undone
    }

    pub(super) fn can_undo(&self) -> bool {
        self.cur_history_idx > 0
    }

    /// Get the id of the point in history that is currently undone to.
    /// I.e. if history is [0, 1, 2], we have undone once, then this will yield 1.
    pub(super) fn get_active_undo_id(&self) -> usize {
        self.history[self.cur_history_idx]
    }

    pub(super) fn undo(&mut self) -> bool {
        if !self.can_undo() {
            return false;
        }
        debug_assert!(self.currently_undone.insert(self.get_active_undo_id()));
        self.cur_history_idx -= 1;
        true
    }

    pub(super) fn redo(&mut self) -> bool {
        if self.at_end_of_history() {
            // If there are no further history-elements to redo to, we cannot redo.
            return false;
        }
        self.cur_history_idx += 1;
        debug_assert!(self.currently_undone.remove(&self.get_active_undo_id()));
        true
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_util;

    macro_rules! set {
        ($($elem:expr),* $(,)?) => {
            BTreeSet::from_iter(vec![$($elem),*])
        };
    }

    /// assert history state with a defined syntax to make tests prettier
    macro_rules! assert_hist {
        ($h:expr,
            gid = $gid:expr,
            idx = $idx:expr,
            history = [$($hist:expr),*],
            undone = [$($undone:expr),*]
        ) => {
            assert_eq!(
                UndoHistory {
                    undo_gid: $gid,
                    history: vec![$($hist),*],
                    cur_history_idx: $idx,
                    currently_undone: set![$($undone),*],
                },
                $h
            )
        };
    }

    #[test]
    fn test_update_history() {
        test_util::setup_test();
        let mut h = UndoHistory::default();
        assert_hist!(h, gid = 0, idx = 0, history = [0], undone = []);
        h.start_new_undo_group();
        assert_hist!(h, gid = 1, idx = 1, history = [0, 1], undone = []);
        // calculating a new undo group id should not do anything as long as idx is history.len() - 1
        h.calculate_undo_id();
        assert_hist!(h, gid = 1, idx = 1, history = [0, 1], undone = []);
        h.start_new_undo_group();
        assert_hist!(h, gid = 2, idx = 2, history = [0, 1, 2], undone = []);
    }

    #[test]
    fn test_undo() {
        test_util::setup_test();
        let mut h = UndoHistory::default();
        h.start_new_undo_group();
        h.start_new_undo_group();
        assert_hist!(h, gid = 2, idx = 2, history = [0, 1, 2], undone = []);
        assert!(h.undo());
        assert_hist!(h, gid = 2, idx = 1, history = [0, 1, 2], undone = [2]);
        assert!(h.undo());
        assert_hist!(h, gid = 2, idx = 0, history = [0, 1, 2], undone = [1, 2]);
    }

    #[test]
    fn test_undo_no_inserts() {
        test_util::setup_test();
        let mut h = UndoHistory::default();
        assert!(!h.undo());
        assert_hist!(h, gid = 0, idx = 0, history = [0], undone = []);
    }

    #[test]
    fn test_undo_edit_undo() {
        test_util::setup_test();
        let mut h = UndoHistory::default();
        h.start_new_undo_group();
        h.undo();
        // This _should_ start a new undo group, as we're working off of an undone state
        h.calculate_undo_id();
        assert_hist!(h, gid = 2, idx = 1, history = [0, 2], undone = [1]);
    }

    #[test]
    fn test_empty_redo() {
        test_util::setup_test();
        let mut h = UndoHistory::default();
        h.start_new_undo_group();
        assert!(!h.redo());
        assert_hist!(h, gid = 1, idx = 1, history = [0, 1], undone = []);
    }

    #[test]
    fn test_undo_redo() {
        test_util::setup_test();
        let mut h = UndoHistory::default();
        h.start_new_undo_group();
        h.undo();
        assert_hist!(h, gid = 1, idx = 0, history = [0, 1], undone = [1]);
        assert!(h.redo());
        assert_hist!(h, gid = 1, idx = 1, history = [0, 1], undone = []);
    }
}
