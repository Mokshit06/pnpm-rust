mod parse_pref;
use parse_pref::parse_pref;

fn resolve_git(pref: &str) {
    let parsed_spec = parse_pref(pref);
}
