use crate::models::{GitActionKind, GitActionRecord};

pub(crate) fn compact_git_action_for_ai(mut action: GitActionRecord) -> GitActionRecord {
    if action.kind == GitActionKind::Commit {
        action.detail = None;
    }
    action
}
