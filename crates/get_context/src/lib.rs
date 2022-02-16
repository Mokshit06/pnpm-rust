pub mod read_lockfiles {
    use std::future::Future;

    use lockfile_utils::types::Lockfile;

    pub struct PnpmContext {
        current_lockfile: Lockfile,
        exists_current_lockfile: bool,
        exists_wanted_lockfile: bool,
        wanted_lockfile: Lockfile,
    }

    pub fn read_lockfiles() {
        let _file_reads = Vec::<Box<dyn Future<Output = Option<Lockfile>>>>::new();
        let _lockfile_had_conflicts = false;

        // file_reads.push(async {
        //     tokio::
        // });
    }
}
