define-command find -override -menu -params 1 -shell-script-candidates %{ fd -tf -L --no-ignore-vcs } %{ edit %arg{1} }

set global lsp_config %{
    [language.rust.settings]
    roots = [".raroot"]
    [language.rust.settings.rust-analyzer]
    # cargo.target = "x86_64-pc-windows-gnu"
    checkOnSave.allTargets = false
    cargo.features = [ "proprietary", "dev" ]
}
