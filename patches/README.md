# Shared ImGui renderer source patch

`imgui-smash-buffer-release-order.patch` targets the renderer source in
`Coolsonickirby/imgui-smash` reference revision
`bb47b205ed0be2fb864269ee2d37fc886b81221e`.

It changes `MemoryBuffer::Finalize` to release the buffer, then its pool, then
its backing allocation. It adds no hooks, clocks, network work, rendering waits
or per-frame processing. The installed host's corresponding disassembly was
checked and showed the original backing-allocation-first release order.

This is a **source patch, not an NRO or binary patch**. From the host repository,
check and apply it using the absolute path to this project's patch:

```sh
git apply --check /path/to/kirby_assist_v2/patches/imgui-smash-buffer-release-order.patch
git apply /path/to/kirby_assist_v2/patches/imgui-smash-buffer-release-order.patch
```

Do not apply twice. If the source differs, inspect it rather than forcing the
patch. Rebuilding Kirby Assist does not rebuild the separate ImGui host. No host
build or deployment is performed by this patch or the Kirby build helper.

Validation: a host C++ harness using the actual old/new finalizer functions and
resource dependency assertions rejected the old order and accepted the corrected
order. The patch also passed `git apply --check` against the reference checkout.
This does **not** validate the complete native renderer or prove the console GPU
crash resolved. GPU-completion synchronization before destroying/reusing resources
is a separate unresolved requirement; the patch deliberately does not invent
queue waits or replace shared rendering hooks without further review.

## Silent match-end investigation

The installed renderer was read again and matched the previously disassembled
host byte-for-byte (SHA-256
`61e19e1b593826b1216228c6536cf2603b512154a8fecc548203e245bb86ed3e`).
The release-order patch therefore has not reached that installed host merely
through Kirby Assist rebuilds. An isolated patched source copy was prepared at
`/private/tmp/kirby-matchend-imgui-fix`; it has not been built or deployed.
The patch applied cleanly and the existing dependency-order harnesses passed.

The latest two reports show a game MainThread null access and a separate
NxSystemThread SDK assertion, not a directly identified Kirby save-worker frame.
The active sequences file validates (1555 recipe sections, one fighter-settings
section). These facts do not establish the renderer as the cause or exclude
earlier memory corruption. Native GPU completion/resource reuse remains to be
verified before calling the console crash fixed.
