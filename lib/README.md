# imgui-smash client library

`libimgui_smash.a` is the existing static client library copied, without
modification or compilation, from your local
`smashUltimateCustomDelayMod/ssbu-online-deluxe/lib/` checkout.

SHA-256:
`0042ff0592313228b0746ee9214091249f7c70a1797e1826d990c27ca2fe0e45`

That checkout's Cargo.lock pins imgui-api to
`f0692019d0affdca9521c0eca586486a59432481`. This project's 1.3.0 build instead
pins the compatible upstream `9259902433138ec9c9ef9e4b152de01403f9456f`:
the generated C bindings are byte-identical, but the newer revision imports
`imgui_get_context_export`, which the installed host lacks. Setup-callback
context caching avoids that import. Both static shim and installed host identify
ImGui 1.92.7 WIP. The diagnostic build links this shim; its required dynamic GUI
imports are checked against the installed host's exports before upload.
That does not establish hardware compatibility or crash freedom.

Upstream library: https://github.com/Coolsonickirby/imgui-smash
Upstream API: https://github.com/Coolsonickirby/imgui-api
Reference integration: https://github.com/saad-script/ssbu-online-deluxe

The matching external `libimgui_smash.nro` host must already be installed on the
Switch. Reuse one matching host for all clients; do not install another renamed
copy alongside Online Deluxe's host. Nothing in this change installs a runtime
plugin or imports Deluxe's networking/performance code.

Set `IMGUI_SMASH_LIB_DIR` to use another matching static shim. Default GUI builds
link this archive; `--no-default-features` keeps a headless input-tools build.
