import string
import secrets
import os
import importlib.util
import inspect

def array_push(x,y):
    x[len(x)]=y

def new_name():
    # In production, change this
    #new_name = lambda: ''.join(
    #        secrets.choice(string.ascii_letters + string.digits) for _ in range(4))
    #return "$v_" + str(new_name())
    if not hasattr(new_name, "varnum"):
        new_name.varnum = 0  # initialize the first time
    new_name.varnum += 1
    return "$v_" + str(new_name.varnum)

def array_unique(data):
    seen = set()
    unique_dict = {}

    for key, value in data.items():
        if value not in seen:
            seen.add(value)
            unique_dict[key] = value
    return unique_dict








def import_all_classes_from_dir(directory):
    for filename in os.listdir(directory):
        if filename.endswith(".py") and not filename.startswith("__"):
            module_name = filename[:-3] 
            file_path = os.path.join(directory, filename)

            # Load the module dynamically
            spec = importlib.util.spec_from_file_location(module_name, file_path)
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)

            # Get all classes in the module
            for name, obj in inspect.getmembers(module):
                if inspect.isclass(obj):
                    globals()[name] = obj 

def import_all_modules_from_dir(directory):
    for filename in os.listdir(directory):
        if filename.endswith(".py") and not filename.startswith("__"):
            module_name = filename[:-3]  # Remove .py extension
            file_path = os.path.join(directory, filename)

            spec = importlib.util.spec_from_file_location(module_name, file_path)
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)

            globals()[module_name] = module

