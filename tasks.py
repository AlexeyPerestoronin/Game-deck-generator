import invoke

import Tools.harness as harness

namespace = invoke.Collection()
namespace.add_collection(harness.collection)
