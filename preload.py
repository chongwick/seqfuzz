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
import math

def prerun(seeds):
    for s in seeds:
        w = Walker(False)
        error,tree = w.analyze(s)
        if not(error) and len(tree.output_all_nodes()) > 1:
            with open("graphs/{}.pickle".format(secrets.token_hex(10)),"wb") as f:
                pickle.dump(tree,f)
        else:
            with open("graphs/{}.error.pickle".format(secrets.token_hex(10)),"wb") as f:
                pickle.dump(tree,f)


def main():

    target_directory = "php_seeds"
    seeds = [os.path.join(target_directory, i) for i in os.listdir(target_directory)]
    chunks = 16
    chunk_size = math.ceil(len(seeds) / chunks)
    chunk_num=int(sys.argv[1])
    prerun(seeds[chunk_num*chunk_size : (chunk_num+1)*chunk_size])

if __name__ == "__main__":
    main()
    #print(sys.argv[1])
