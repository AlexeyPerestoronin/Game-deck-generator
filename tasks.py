import invoke

import harness

namespace = invoke.Collection()
namespace.add_collection(harness.collection)
