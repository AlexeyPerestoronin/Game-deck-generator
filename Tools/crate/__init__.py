"""Invoke collection for Rust crate build tasks."""

import invoke

from . import actions

collection = invoke.Collection("crate")
collection.add_task(actions.cargo_build)
collection.add_task(actions.trunk_build)
