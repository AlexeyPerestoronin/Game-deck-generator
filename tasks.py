import invoke


@invoke.task()
def test(ctx, unsaved: bool = False):
    print("it's working...")


namespace = invoke.Collection()
namespace.add_task(test)