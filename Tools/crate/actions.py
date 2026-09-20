"""Invoke tasks for building Rust crates via cargo and trunk."""

import pathlib

import invoke

__all__ = ["cargo_build", "trunk_build"]


def _profile_flag(debug: bool) -> str:
    # cargo/trunk default to a debug profile; release is opt-in via debug=False.
    return "" if debug else " --release"


@invoke.task(help={"path": "path to Cargo.toml for target crate", "debug": "build flag in debug mode (by default: False)"})
def cargo_build(ctx, path: str, debug: bool = False):
    """Build Rust crate via cargo"""
    ctx.run(f'cargo build --manifest-path "{path}"{_profile_flag(debug)}')


@invoke.task(help={"path": "path to Cargo.toml for target crate", "debug": "build flag in debug mode (by default: False)"})
def trunk_build(ctx, path: str, debug: bool = False):
    """Build Rust crate via trunk"""
    crate_dir = pathlib.Path(path).parent
    with ctx.cd(crate_dir):
        ctx.run(f"trunk build{_profile_flag(debug)}")
