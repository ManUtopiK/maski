use std::path::{Path, PathBuf};

fn main() {
    // cc registers the files it compiles, but the ANSI renderer it gets is the
    // patched copy in OUT_DIR — the vendored original would go unwatched, and a
    // md4x bump would silently reuse a stale build.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=vendor/md4x/src");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR is set by cargo"));
    let ansi_renderer = patch_ansi_renderer(&out_dir);

    cc::Build::new()
        .file("vendor/md4x/src/md4x.c")
        .file("vendor/md4x/src/entity.c")
        .file(&ansi_renderer)
        .file("vendor/md4x/src/renderers/md4x-heal.c")
        .include("vendor/md4x/src")
        .include("vendor/md4x/src/renderers")
        .warnings(false)
        .compile("md4x");
}

/// md4x's ANSI renderer forgets the line break before a list nested inside a
/// *tight* list item, so the first bullet lands on the parent's line:
///
/// ```text
/// - release              * release  * flags: -r --release
///   - flags: -r      →     * type: boolean
///   - type: boolean
/// ```
///
/// That is exactly how mask's `**OPTIONS**` blocks are written, so every
/// maskfile is affected. Loose lists render fine: their item text sits in a
/// paragraph whose `leave_block()` emits the newline and clears `li_opened`.
///
/// The fix belongs upstream, but it cannot simply be applied to the vendored
/// file: the Nix build wipes `vendor/md4x` and copies the flake input from the
/// store, where files are read-only. So patch a copy in `OUT_DIR` and compile
/// that instead — this works for both the submodule and the Nix path, which
/// pin the same md4x revision.
fn patch_ansi_renderer(out_dir: &Path) -> PathBuf {
    const SOURCE: &str = "vendor/md4x/src/renderers/md4x-ansi.c";

    const HELPER_ANCHOR: &str =
        "/* Render a blank separator line with alert bar prefix when inside an alert. */";

    const HELPER: &str = "\
/* Break the line before a nested list (maski patch, see build.rs). */
static void
render_nested_list_break(MD_ANSI* r)
{
    if(r->li_opened) {
        render_newline(r);
        r->li_opened = 0;
    }
}

";

    const UL_ANCHOR: &str = "\
        case MD_BLOCK_UL:
            if(r->need_newline && r->list_depth == 0) {
                render_separator(r);
                r->need_newline = 0;
            }
            break;";

    const OL_ANCHOR: &str = "            r->ol_counter = ((MD_BLOCK_OL_DETAIL*)detail)->start;";

    let source = std::fs::read_to_string(SOURCE)
        .unwrap_or_else(|e| panic!("cannot read {SOURCE}: {e}"));

    let patched = replace_once(source, HELPER_ANCHOR, &format!("{HELPER}{HELPER_ANCHOR}"));
    let patched = replace_once(
        patched,
        UL_ANCHOR,
        &UL_ANCHOR.replace("            break;", "            render_nested_list_break(r);\n            break;"),
    );
    let patched = replace_once(
        patched,
        OL_ANCHOR,
        &format!("            render_nested_list_break(r);\n{OL_ANCHOR}"),
    );

    let target = out_dir.join("md4x-ansi.c");
    std::fs::write(&target, patched).unwrap_or_else(|e| panic!("cannot write {target:?}: {e}"));
    target
}

/// Substitute an anchor that must appear exactly once. Anything else means md4x
/// moved and the patch needs a fresh look, so fail the build loudly rather than
/// silently shipping the bug back.
fn replace_once(source: String, anchor: &str, replacement: &str) -> String {
    let hits = source.matches(anchor).count();
    assert_eq!(
        hits, 1,
        "md4x patch: expected exactly one match for anchor, found {hits}. \
         md4x changed — revisit patch_ansi_renderer() in build.rs.\nAnchor:\n{anchor}"
    );
    source.replace(anchor, replacement)
}
