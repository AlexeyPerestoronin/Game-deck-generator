import invoke
import shutil
import pathlib

import Tools as tools
import Tools.utils as utils

@invoke.task()
def remove_python_cache(ctx):
    """Remove python cache files (__pycache__) and byte-code (*.pyc)"""
    cwd = pathlib.Path(utils.settings.get_cwd())
    target_dir = cwd / "Tools"

    for p in list(target_dir.rglob("*")):
        if p.is_dir() and p.name == "__pycache__":
            shutil.rmtree(p)

@invoke.task()
def make_task_template(ctx, type: str, name: str ):
    """Creates template for new task"""
    # TODO: необходимо реализовать
    # 
    # Особенности реализации:
    # - для определения корневой директории создания папки с шаблоном задачи необходимо использовать utils.settings.get_task_dir()
    # - в корневой директории необходимо создать папку для задачи по имени "{index} {name}" где:
    # - - index - это номер задачи в корневой директории вычисляемый как следующий по порядку среди всех имеющихся задач
    # - - name - имя задачи переданное в качестве параметра
    # - внутри папки с задачей необходимо создать файл todo.md со следующим содержимым:
    # - - копия WiKi\templates\development.md если type = development
    # - - копия WiKi\templates\planning.md если type = planning
    # - - копия WiKi\templates\refactoring.md если type = refactoring
    # - - если тип отличается от доступных вызвать исключение


namespace = invoke.Collection()
namespace.add_task(remove_python_cache)

namespace.add_collection(tools.collection)
