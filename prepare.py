import secrets
import re
import codecs
import os
import subprocess

banned = ["posix_kill", "chmod", "posix_setrlimit",
        "exec", "passthru", "system", "shell_exec", "popen", "proc_open",
        "unlink", "rmdir", "rename", "chown", "chgrp", "symlink", "pcntl_exec", 
        "pcntl_fork", "posix_kill", "pcntl_signal",]

def is_bad_seed(seed):
    with codecs.open(seed,"r",encoding="utf-8",errors="ignore") as f:
        file_string = f.read()
    for term in banned:
        if term in file_string:
            return True
    return False

def parse_phpt(content, section):
    if section not in content:
        return ""
    start_idx = content.find(section) + len(section)
    x = re.search("--([_A-Z]+)--", content[start_idx:])
    end_idx = x.start() if x != None else len(content) - 1
    ret = content[start_idx:start_idx + end_idx].strip("\n")
    return ret

def load_seed_corpus(seed_corpus_directory):
    #command = ['bash','./corpus_loader.sh',seed_corpus_directory]
    #subprocess.run(command,text=True,timeout=40,capture_output=True)
    os.system("./corpus_loader.sh {}".format(seed_corpus_directory))
    with open("tmp_files.txt",'r') as f:
        files = f.readlines()
    os.remove("tmp_files.txt")
    return [i.split("\n")[0] for i in files]


os.system("git clone https://github.com/php/php-src.git")

for i in ["php-src/" + s for s in load_seed_corpus("php-src") if not is_bad_seed(s)]:
    with codecs.open(i,'r',encoding='utf-8',errors='ignore') as f:
        code = f.read()
    #if "gh" in code or "GH" in code or "bug" in code or "gh" in i or "GH" in i or "bug" in i or "BUG" in i or "BUG" in code or "Gh" in i or "Gh" in code:
    if True:
        with codecs.open("/php_seeds/"+secrets.token_hex(10),"w",encoding='utf-8',errors='ignore') as f:
            f.write(parse_phpt(code,"--FILE--"))
