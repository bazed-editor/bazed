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
    /// As long as edits don't break this undo group, this will stay the same.
    /// Once the undo group is broken, `perform_edit` will write
    /// this ID into `history` and advance the counter.
    ///
    /// This ID always just increments and will never reuse previous IDs, even if we undo and then do other edits.
    cur_undo_gid: usize,
    /// The current position in the history. This is *not* an undo-group-id.
    /// Think of this as a cursor into time:
    /// - every id `history[n]` where `n > current_history_index` is in the future and may be redone to.
    /// - every id `history[n]` where `n < current_history_index` is in the past and may be undone to
    /// - when adding further edits, history gets truncated to end at the current index, and a new step gets added.
    ///
    /// **Invariant**: always < history.len().
    cur_history_idx: usize,
    /// List of undo groups that are currently relevant.
    /// Elements before and including `current_undo_index` are in the history but not undone,
    /// everything after `current_undo_index` is currently undone but may be redone.
    history: Vec<usize>,
    /// Set of undo groups that are currently undone.
    /// This may contain undo groups that are no longer part of the history due do
    /// us undoing and then performing edits.
    currently_undone: BTreeSet<usize>,
}

impl Default for UndoHistory {
    fn default() -> Self {
        Self {
            cur_undo_gid: 0,
            cur_history_idx: 0,
            history: vec![0],
            currently_undone: Default::default(),
        }
    }
}

impl UndoHistory {
    /// This should be called on every edit.
    /// If a new undo group should be created, creates a new undo group id,
    /// truncates any history-elements that are in the future and adds the new group to the history.
    /// Otherwise, just returns the current undo group id.
    pub(super) fn perform_edit(&mut self, new_undo_group: bool) -> usize {
        tracing::trace!(undo_history = ?self, new_undo_group, "Adding an edit to the undo history");
        // When told to create a new undo group, we will.
        // However, we'll also create a new group anyways if we're working off of an undone state
        let needs_new_undo_group =
            new_undo_group || self.cur_history_idx != (self.history.len() - 1);
        if needs_new_undo_group {
            self.cur_undo_gid += 1;
            self.cur_history_idx += 1;
            self.history.truncate(self.cur_history_idx);
            self.history.push(self.cur_undo_gid);
        }
        self.cur_undo_gid
    }

    pub(super) fn currently_undone(&self) -> &BTreeSet<usize> {
        &self.currently_undone
    }

    pub(super) fn current_undo_group_id(&self) -> usize {
        self.cur_undo_gid
    }

    pub(super) fn can_undo(&self) -> bool {
        self.cur_history_idx > 0
    }

    pub(super) fn can_redo(&self) -> bool {
        self.cur_history_idx < self.history.len() - 1
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
        if !self.can_redo() {
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
                    cur_undo_gid: $gid,
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
        h.perform_edit(true);
        assert_hist!(h, gid = 1, idx = 1, history = [0, 1], undone = []);
        h.perform_edit(false);
        assert_hist!(h, gid = 1, idx = 1, history = [0, 1], undone = []);
        h.perform_edit(true);
        assert_hist!(h, gid = 2, idx = 2, history = [0, 1, 2], undone = []);
    }

    #[test]
    fn test_undo() {
        test_util::setup_test();
        let mut h = UndoHistory::default();
        h.perform_edit(true);
        h.perform_edit(true);
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
        h.perform_edit(true);
        h.undo();
        // True or false should not matter here, as we should _always_ create a new
        // group when working off of a past state
        h.perform_edit(false);
        assert_hist!(h, gid = 2, idx = 1, history = [0, 2], undone = [1]);
    }

    #[test]
    fn test_empty_redo() {
        test_util::setup_test();
        let mut h = UndoHistory::default();
        h.perform_edit(true);
        assert!(!h.redo());
        assert_hist!(h, gid = 1, idx = 1, history = [0, 1], undone = []);
    }

    #[test]
    fn test_undo_redo() {
        test_util::setup_test();
        let mut h = UndoHistory::default();
        h.perform_edit(true);
        h.undo();
        assert_hist!(h, gid = 1, idx = 0, history = [0, 1], undone = [1]);
        assert!(h.redo());
        assert_hist!(h, gid = 1, idx = 1, history = [0, 1], undone = []);
    }
}
