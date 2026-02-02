import string
import codecs
import secrets
import subprocess
import os
import sys
from walker import Walker
from mutator import *
from random import choice,randint
import json
import pickle


def _is_bad_seed(seed):
    banned = ["posix_kill", "chmod", "posix_setrlimit",
              "exec", "passthru", "system", "shell_exec", "popen", "proc_open",
              "unlink", "rmdir", "rename", "chown", "chgrp", "symlink",
              "eval", "create_function", "assert", "dl", "pcntl_exec",
              "mail", "pcntl_fork", "posix_kill", "pcntl_signal",]
    with codecs.open(seed,"r",encoding='utf-8',errors='ignore') as f:
        file_string = f.read()
    for term in banned:
        if term in file_string:
            return True
    return False

def _get_target_seed(seeds):
    target_seed = choice(seeds)
    w = Walker(False)
    error,tree = w.analyze(target_seed)
    while not(error) and len(tree.output_all_nodes()) <= 1:
        target_seed = choice(seeds)
        error,tree = w.analyze(target_seed)
    return tree

def _get_splice_input(seeds):
    target_seed = choice(seeds)
    w = Walker(False)
    error,tree = w.analyze(target_seed)
    while len(tree.output_all_nodes()) <= 1:
        target_seed = choice(seeds)
        error,tree = w.analyze(target_seed)
    return tree

def _clean():
    os.system(f"git clean -fd -e scripts -e php_seeds -e php -e /ramdisk -e /graphs -e /graphs2")

def _load_pickle(s):
    with open(s,"rb") as f:
        tree = pickle.load(f)
    return tree

def main():

    target_directory = "graphs"
    output_directory = "scripts"
    seeds = [os.path.join(target_directory, i) for i in os.listdir(target_directory)]

    is_error = lambda x: os.path.exists(x+".er")
    is_trash = lambda x: os.path.exists(x+".tr")

    count = 0

    while True:

        count += 1
        if count == 10:
            count = 0
            _clean()
        #php_files = []
        php_file = os.path.join(output_directory,secrets.token_hex(10)+".php")
        #maybe we make a list here where we make the base with the mutated control/data flows
        #and then we do the operation and value modifications afterwards
        try:
            target = None
            strategy = choice([0,1,3])
            if strategy == 3:
                target = _load_pickle(choice(seeds))
                for _ in range(randint(1,100)):
                    splice = _load_pickle(choice(seeds))
                    stochastic_splice_dataflow(target,splice,randint(3,10))
            if strategy == 0:
                target = _load_pickle(choice(seeds))
                for _ in range(randint(1,4)):
                    splice = _load_pickle(choice(seeds))
                    if stochastic_splice_controlflow(target,splice) == False:
                        stochastic_splice_dataflow(target,splice)
            elif strategy == 1:
                # L O L
                trees = [_load_pickle(choice(seeds)) for _ in range(randint(2,8))]
                # Also terrible practice. target should consistently be consistent
                target = rando_max(trees)

            if(random.choice([True,False,False])):
                modify_operation(target)
            tmp = []
            target.output_env(tmp)
            output = "\n".join(tmp)
            output = "<?php\n" + output
            with codecs.open(php_file,"w",encoding='utf-8',errors='ignore') as f:
                f.write(output)

        except Exception as e:
            continue

        for i in range(2):
            command = ['bash','./sanitize.sh',os.path.join(os.getcwd(),php_file),str(i)]#, config]
            child = None
            try:
                child = subprocess.run(command,
                                       text=True,
                                       timeout=40,
                                       capture_output=True)
            except subprocess.TimeoutExpired as exc:
                break
#            try:
#                if is_trash(php_file) or is_error(php_file):
#                    break;
#            except Exception as e:
#                break;
        try:
            if is_trash(php_file):
                os.remove(php_file+".tr")
            elif not(is_error(php_file)):
                os.remove(php_file)
        except Exception as e:
            continue

    #for seed in seeds:

    #print(target_file)
    #a = Walker()
    #allPaths = a.analyze(target_file)
    #print(pretty_print(allPaths))
    #with open("g","wb") as f:
    #    pickle.dump([allPaths],f,protocol=pickle.HIGHEST_PROTOCOL)

    ##target_file = "test.php"
    #target_file = "sample2.php"
    #target_file_2 = "h.php"
    #a1 = Walker()
#a2 = Walker()

    #functions,allPaths = a1.analyze(target_file)
    #functions_2,allPaths_2 = a2.analyze(target_file_2)

    #merged_function_graphs, merged_graph = stochastic_merge(
    #        allPaths,functions,
    #        allPaths_2,functions_2)

    ###print(allPaths_2.getString())
    ###print(pretty_print(functions_2,allPaths_2))
    ##print(merged_graph.getString())
    #print(pretty_print(merged_function_graphs,merged_graph))

if __name__ == "__main__":
    main()
    #print(sys.argv[1])
