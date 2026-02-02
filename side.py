import random
import sys
from walker import Walker
import pickle

def main():
    
    #Each graph of subgraphs should be a tuple -> (all_nodes,rel_db)
    w1 = Walker(True)
    #er1,env1 = w1.analyze("test1.php")
    er1,env1 = w1.analyze("g.php")
    w2 = Walker(True)
    er2,env2 = w2.analyze("test2.php")
    print(env1.output_all_nodes())
    print(env2.output_all_nodes())
    db1 = env1.get_rel_db()
    db2 = env2.get_rel_db()
    prog = (env1.output_all_nodes(),env1.get_rel_db())
    with open("test.pickle",'wb') as f:
        pickle.dump(prog,f)
    #dfwparams
    index = random.randint(0,len(db1)-1)
    while db1[index].get_type() == 'param':
        index = random.randint(0,len(db1)-1)
    db1[index]=random.choice(env2.output_all_nodes())
    #print(db1,db2)

    out1 = []
    out2 = []
    env1.output_env(out1)
    print("\n".join(out1))
    #print(env2.output_all_nodes())


    #d = env.get_node_env()
    #l = list(d.items())
    #random.shuffle(l)
    #d_shuffled = dict(l)
    #env.node_env=d_shuffled
    #print(env.node_env)
    #print(env.children_envs)
    #for value in env.node_env.values():
    #    value.get_expression(h)
    #print("\n".join(h))
    #print(env.node_env['g'].getExpression())
    #error,allPaths = a.analyze(target_file)
    #print(allPaths.getNodeString())
    #print(pretty_print(allPaths))

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
