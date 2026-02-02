from sequence import *
from seqNodes import *
import random
import json

with open('php_lang_description.json', 'r') as f:
    data = json.load(f)

middle_op_nodes = data['OpNodes']['MiddleOpNodes']
front_op_nodes = data['OpNodes']['FrontOpNodes']
inner_op_nodes = data['OpNodes']['InnerOpNodes']
rear_op_nodes = data['OpNodes']['RearOpNodes']


# this accepts two environments
def stochastic_splice_dataflow(target_env,splicing_env,reps=3):
    for i in range(reps):
        tar_rel_db = target_env.get_rel_db()
        splicing_nodes = splicing_env.output_all_nodes()
        splicing_nodes = splicing_nodes + target_env.output_all_nodes()
        index = 0
        if len(tar_rel_db) > 1:
            index = random.randint(0,len(tar_rel_db)-1)
        chosen_one = random.choice(splicing_nodes)
        if type(chosen_one) == FuncSeqNode and chosen_one.get_dependency() != None:
            target_env.add_fn_dependency(chosen_one.get_func_name(),chosen_one.get_dependency())
        tar_rel_db[index]=chosen_one
        
def stochastic_splice_controlflow(target_env,splicing_env):
    splicing_env_flows = list(splicing_env.get_flow_branches().values())
    if len(splicing_env_flows) == 0:
        return False
    target_list = list(target_env.get_flow_branches().values())
    target_list.append(target_env)
    final_target = random.choice(target_list)
    flow = random.choice(splicing_env_flows)
    # Insert the flow branch into the target environment
    final_target.set_node(node=flow)
    stochastic_splice_dataflow(final_target,flow)

def rando_max(trees:list):
    yggdrasil = Seq()
    all_nodes = []
    all_nodes = [all_nodes + t.output_all_nodes() for t in trees][0]
    relationships = []
    relationships = [relationships + t.get_rel_db() for t in trees][0]

    for _ in range(len(all_nodes)):
        node = random.choice(all_nodes)
        yggdrasil.set_node(node=node)
        yggdrasil.all_nodes.append(node) 
    yggdrasil.relational_database = relationships 
    if stochastic_splice_controlflow(yggdrasil,yggdrasil) == False:
        stochastic_splice_dataflow(yggdrasil,yggdrasil)
    return yggdrasil

def rando_super_max(trees:list):
    top_tree = Seq()
    all_nodes = []
    all_nodes = [all_nodes + t.output_all_nodes() for t in trees][0]
    relationships = []
    relationships = [relationships + t.get_rel_db() for t in trees][0]

def modify_control_param(target_env):
    if len(target_env.children_envs["Flow"]) != 0:
        ...

def modify_control_body(target_env):
    if len(target_env.children_envs["Flow"]) != 0:
        ...

def modify_value(target_env):
    val_nodes = [i for i in target_env.output_all_nodes() if isinstance(i,GenericSeqNode)]
    if len(op_nodes) > 0:
        for _ in range(random.randint(0,len(op_nodes))):
            target = random.choice(op_nodes)
            target.set_value(random.choice(["PHP_INT_MAX","PHP_FLOAT_MAX","-1","0"]))

def modify_operation(target_env):
    op_nodes = [i for i in target_env.output_all_nodes() if isinstance(i,OPNode)]
    if len(op_nodes) > 0:
        for _ in range(random.randint(0,len(op_nodes))):
            target = random.choice(op_nodes)
            if isinstance(target,MiddleOpNode):
                target.set_value(random.choice(list(middle_op_nodes.values())))
            elif isinstance(target,FrontOpNode):
                target.set_value(random.choice(list(front_op_nodes.values())))
            elif isinstance(target,InnerOpNode):
                target.set_value(random.choice(list(inner_op_nodes.values())))
            elif isinstance(target,RearOpNode):
                target.set_value(random.choice(list(rear_op_nodes.values())))
