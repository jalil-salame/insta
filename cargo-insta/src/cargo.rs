use std::path::PathBuf;

pub(crate) use cargo_metadata::Package;

/// Find snapshot roots within a package
// (I'm not sure how necessary this is; relative to just using all paths?)
pub(crate) fn find_snapshot_roots(package: &Package) -> Vec<PathBuf> {
    let mut roots = Vec::new();

    // the manifest path's parent is always a snapshot container.  For
    // a rationale see GH-70.  But generally a user would expect to be
    // able to put a snapshot into foo/snapshots instead of foo/src/snapshots.
    if let Some(manifest) = package.manifest_path.parent() {
        roots.push(manifest.as_std_path().to_path_buf());
    }

    // additionally check all targets.
    for target in &package.targets {
        // custom build scripts we can safely skip over.  In the past this
        // caused issues with duplicate paths but that's resolved in other
        // ways now.  We do not want to pick up snapshots in such places
        // though.
        if target.kind.iter().any(|kind| kind == "custom-build") {
            continue;
        }

        // this gives us the containing source folder.  Typically this is
        // something like crate/src.
        let root = target.src_path.parent().unwrap().as_std_path();
        roots.push(root.to_path_buf());
    }

    // TODO: I think this root reduction is duplicative over the logic in
    // `make_snapshot_walker`; could try removing.

    // reduce roots to avoid traversing into paths twice.  If we have both
    // /foo and /foo/bar as roots we would only walk into /foo.  Otherwise
    // we would encounter paths twice.  If we don't skip them here we run
    // into issues where the existence of a build script causes a snapshot
    // to be picked up twice since the same path is determined.  (GH-15)
    roots.sort_by_key(|x| x.as_os_str().len());
    let mut reduced_roots = vec![];
    for root in roots {
        if !reduced_roots.iter().any(|x| root.starts_with(x)) {
            reduced_roots.push(root);
        }
    }

    reduced_roots
}
