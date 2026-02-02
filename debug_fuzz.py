import codecs
import secrets
import subprocess
import os
import sys
from walker import Walker
from mutator import *
import pickle
from random import choice,randint
import pickle

def _is_bad_seed(seed):
    with codecs.open(seed,"r",encoding='utf-8',errors='ignore') as f:
        file_string = f.read()
    if "pkill" in file_string or "unlink" in file_string or "chmod" in file_string:
        return True
    else:
       return False

def _get_target_seed(seeds):
    target_seed = choice(seeds)
    while(_is_bad_seed(target_seed)):
          target_seed = choice(seeds)
    #target_seed = "../../php_seeds/2c96cb0f837020efca59"
    w = Walker(False)
    error,tree = w.analyze(target_seed)
    while not(error) and len(tree.output_all_nodes()) <= 1:
        target_seed = choice(seeds)
        error,tree = w.analyze(target_seed)
    print("target " + target_seed)
    return tree

def _get_splice_input(seeds):
    target_seed = choice(seeds)
    while(_is_bad_seed(target_seed)):
          target_seed = choice(seeds)
    #target_seed = "../../php_seeds/2a44628246018d8ffd60"
    w = Walker(False)
    error,tree = w.analyze(target_seed)
    while len(tree.output_all_nodes()) <= 1:
        target_seed = choice(seeds)
        error,tree = w.analyze(target_seed)
    print("splice " + target_seed)
    return tree
    

def main():
    

    target_directory = '../php_seeds'
    seeds = [os.path.join(target_directory, i) for i in os.listdir(target_directory)]

    is_error = lambda x: os.path.exists(x+".er")
    is_trash = lambda x: os.path.exists(x+".tr")

    strategy = choice([0,1])
    target = None
    if strategy == 0:
        target = _get_target_seed(seeds)
        for _ in range(randint(1,4)):
            splice = _get_splice_input(seeds)
            if stochastic_splice_controlflow(target,splice) == False:
                stochastic_splice_dataflow(target,splice)
    elif strategy == 1:
        # L O L
        trees = [_get_target_seed(seeds) for _ in range(randint(2,8))]
        # Also terrible practice. target should consistently be consistent
        target = rando_max(trees)
    if (choice([True,False,False])):
        modify_operation(target)

    #target = rando_max([_get_target_seed(seeds),_get_target_seed(seeds)])
    tmp = []
    target.output_env(tmp)
    output = "\n".join(tmp)
    output = "<?php\n" + output
    print(output)

if __name__ == "__main__":
    main()
    #print(sys.argv[1])
