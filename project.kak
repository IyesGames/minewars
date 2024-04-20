set global lsp_config %{
    [language.rust.settings.rust-analyzer]
    # cargo.target = "x86_64-pc-windows-gnu"
    # checkOnSave.allTargets = false
    cargo.features = [ "proprietary", "dev" ]
}
