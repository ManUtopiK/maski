fn main() {
    cc::Build::new()
        .file("vendor/md4x/src/md4x.c")
        .file("vendor/md4x/src/entity.c")
        .file("vendor/md4x/src/renderers/md4x-ansi.c")
        .file("vendor/md4x/src/renderers/md4x-heal.c")
        .include("vendor/md4x/src")
        .include("vendor/md4x/src/renderers")
        .warnings(false)
        .compile("md4x");
}
