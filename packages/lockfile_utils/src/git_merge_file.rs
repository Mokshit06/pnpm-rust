use crate::merge_changes::merge_changes;
use crate::read::LockfileFile;
use crate::types::Lockfile;

const MERGE_CONFLICT_PARENT: &'static str = "|||||||";
const MERGE_CONFLICT_END: &'static str = ">>>>>>>";
const MERGE_CONFLICT_THEIRS: &'static str = "=======";
const MERGE_CONFLICT_OURS: &'static str = "<<<<<<<";

pub fn autofix_merge_conflicts(file_content: &str) -> LockfileFile {
    let ParsedMergeFile { ours, theirs } = parse_merge_file(&file_content);

    merge_changes(
        serde_yaml::from_str::<LockfileFile>(&ours).unwrap(),
        serde_yaml::from_str::<LockfileFile>(&theirs).unwrap(),
    )
}

enum MergeState {
    Top,
    Ours,
    Theirs,
    Parent,
}

struct ParsedMergeFile {
    ours: String,
    theirs: String,
}

fn parse_merge_file(file_content: &str) -> ParsedMergeFile {
    let pattern = regex::Regex::new(r"[\n\r]+").unwrap();
    let mut lines = pattern.split(file_content).collect::<Vec<_>>();
    let mut state = MergeState::Top;
    let mut ours = vec![];
    let mut theirs = vec![];

    // `reverse` required because we're using `pop` elements
    // instead of `shift` which pnpm uses and rust doesn't have
    // an equivalent fn
    lines.reverse();

    while let Some(line) = lines.pop() {
        state = if line.starts_with(MERGE_CONFLICT_PARENT) {
            MergeState::Parent
        } else if line.starts_with(MERGE_CONFLICT_OURS) {
            MergeState::Ours
        } else if line.starts_with(MERGE_CONFLICT_THEIRS) {
            MergeState::Theirs
        } else {
            MergeState::Top
        };

        match state {
            MergeState::Top => ours.push(line),
            MergeState::Ours => ours.push(line),
            MergeState::Theirs => theirs.push(line),
            _ => {}
        };
    }

    ParsedMergeFile {
        ours: ours.join("\n"),
        theirs: theirs.join("\n"),
    }
}
