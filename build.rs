fn main() {
    glib_build_tools::compile_resources(
        &["resources"],
        "resources/wlshud.gresource.xml",
        "wlshud.gresource",
    );
}
