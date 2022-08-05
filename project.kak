set global lsp_config %{
    [language.rust.settings.rust-analyzer]
    # cargo.target = "x86_64-pc-windows-msvc"
    cargo.features = ["proprietary", "dev", "iyesengine/dynamic"]
    checkOnSave.allTargets = false
}
