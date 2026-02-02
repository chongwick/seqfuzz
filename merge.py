from ASNode import ASNode
from random import choice,randint
from opNodes import *
from leafNodes import *
from specialNodes import *
from env import ProgramEnv

def stochastic_merge(graph_0:ProgramEnv, graph_1:ProgramEnv):
    merged_graph = graph_0.getSelfCopy()

    #Collect graph_2's leafNodes
    #graph_1_nodes = {"OPNodes":[],"leafNodes":[]}
    graph_nodes = []
    for path in graph_1.cond_paths.values():
        for node in path.env.values():
            graph_nodes.append(node)
            #if isinstance(node,OPNode):
            #    graph_1_nodes["OPNodes"].append(node)
            #else:
            #    graph_1_nodes["leafNodes"].append(node)

    #Different conditonal paths through a program -> onePath objects
    graph_0_indexes = []
    for path_idx, path_obj in graph_0.cond_paths.items():
        #One of the paths
        '''Perhaps we want to merge these as well... maybe later... maybe more effective for js'''
        one_path_condition = path_obj.condition#.getExpression()
        #ASNodes
        for node_idx, node_obj in path_obj.env.items(): #maybe need a getter func
            graph_0_indexes.append((path_idx,node_idx))

    for i in range(randint(0,len(graph_0_indexes))):
        r_n_idx = choice(graph_0_indexes) #replacement node index
        r_node = graph_0.cond_paths[r_n_idx[0]].env[r_n_idx[1]]
        #Avoid function nodes
        while(not(isinstance(r_node,ASNode))):
            replacement_node = choice(graph_0_indexes)
            r_node = graph_0.cond_paths[r_n_idx[0]].env[r_n_idx[1]]

        merged_graph.cond_paths[r_n_idx[0]].env[r_n_idx[1]] = choice(graph_nodes)

    return merged_graph

    ##Here we replace opnodes with opnodes, leafnodes with leafnodes.
    ##I think we could interchange them, however.
    #if isinstance(r_node,OPNode):
    #    merged_graph.cond_paths[r_n_idx[0]].env[r_n_idx[1]] = (
    #            choice(graph_1_nodes["OPNodes"]))
    #else:
    #    merged_graph.cond_paths[r_n_idx[0]].env[r_n_idx[1]] = (
    #            choice(graph_1_nodes["leafNodes"]))

    #merged_graph.cond_paths[r_n_idx[0]].env[r_n_idx[1]] = (
    #    choice(graph_1_nodes[choice(["OPNodes","leafNodes"])]))
