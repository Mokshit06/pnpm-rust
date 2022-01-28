use std::collections::HashMap;

use types::BaseManifest;

pub struct WorkspacePackage<'a> {
    pub dir: String,
    pub manifest: &'a BaseManifest,
}

pub type WorkspacePackages<'a> = HashMap<String, HashMap<String, WorkspacePackage<'a>>>;
