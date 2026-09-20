import invoke

@invoke.task(
    help = {
        "path": "path to Cargo.toml for target crate",
        "debug": "build flag in debug mode (by default: False)"
    }
)
def cargo_build(ctx, path: str, debug: bool = False):
    """Build Rust crate via cargo"""
    # TODO: need to implement

@invoke.task(
    help = {
        "path": "path to Cargo.toml for target crate",
        "debug": "build flag in debug mode (by default: False)"
    }
)
def trunk_build(ctx, path: str, debug: bool = False):
    """Build Rust crate via trunk"""
    # TODO: need to implement