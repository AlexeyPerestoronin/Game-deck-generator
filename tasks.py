import invoke

from . import (harness)

namespace = invoke.Collection()
namespace.add_collection(harness.collection)
