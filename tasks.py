import invoke

import Tools.harness as harness
import Tools.crate as crate

namespace = invoke.Collection()
namespace.add_collection(harness.collection)
namespace.add_collection(crate.collection)
