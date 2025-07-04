pub(crate) fn ls_tree_invoke(name_only: bool) -> anyhow::Result<()> {
    anyhow::ensure!(name_only, "only name only is supported");

    Ok(())
}
