from copy import deepcopy
from utils import array_push, array_unique
from seqNodes import *
import re

class Seq: #ControlTree
    def __init__(self,parent=None,node_env=None,root=None):
        if node_env is None:
            node_env = {}
        # This is for printing
        self.node_env = node_env # All of the defined objects/values in the env

        # This is for selecting a node to swap
        self.all_nodes = [] # This is only relevant for the topmost environment

        ###
        # Dependencies are protected relations
        # methodcall->objectdec
        # funccall->funcdec
        ###
        # This is for swapping a node
        #every node relationship will be mapped into here and relationships will be 
        #accessed through here.
        self.relational_database = []
            
        self.root = root #ControlNode or None
        self.parent_env = parent 
        #self.children_envs = {}
        #This will also be outputted to manipulate control flows
        self.children_envs = {"Function":{},
                              "Flow":{}, # First control flow is the 
                                                           # parent. that just good
                              }
        self.defined_classes = {}
        self.defined_objects = {}

    #These are functions only to be called by the top environment
    #Every time a node is created, this has to be used
    def check_in(self,node): 
        if self.parent_env == None:
            self.all_nodes.append(node)

    def get_rel_db(self):
        #return self.relational_database
        env = self
        while env.parent_env != None:
            env = env.parent_env
        return env.relational_database


    def get_relation(self,key):
        return self.relational_database[key]

    #Every time there is a relationship between two nodes, this has to be used
    def set_relation(self,value):
        self.relational_database.append(value)
        return len(self.relational_database)-1

    #Everything else can be used by other environments

    def get_env_type(self):
        return type(self)

    def set_node(self,name=None,node=None):
        assert(isinstance(node,ASNode) or isinstance(node,Seq))
        if name != None:
            if name in self.node_env:
                old_node = self.node_env[name]
                # VERY non-performant
                self.node_env = {key if key != name else -(len(self.node_env)):
                                 value for key, value in self.node_env.items()}
                node.set_name(old_node.get_name())
            self.node_env[name] = node
        else:
            array_push(self.node_env,node)

    def add_class_def(self,class_name,definition):
        self.defined_classes[class_name] = definition

    def add_object(self,obj_name,node):
        assert(isinstance(obj_name,str))
        self.defined_objects[obj_name] = node

    def get_class_def(self,class_name):
        if class_name in self.defined_classes:
            return self.defined_classes[class_name]
        else:
            return None

    def get_object(self,obj_name):
        assert(obj_name in self.defined_objects)
        return self.defined_objects[obj_name]
     
    def get_node(self,name):
        if name in self.node_env:
            return self.node_env[name]
        else:
            return None

    def get_defined_functions(self):
        return self.children_envs["Function"]

    def get_flow_branches(self):
        return self.children_envs["Flow"]

    def spawn_env(self,env_type,name=None,root=None,flow_type=None):
        new_env = None
        # An if statement spawns a new environment and inherit all the parent's nodes
        # A function definition spawns a new environment and does not inherit the nodes
        if env_type == "Function":
            new_env = FunctionBranch(self,self.relational_database,name,{})
            self.children_envs[env_type][name] = new_env
        elif env_type == "Flow":
            new_env = FlowBranch(self,self.relational_database,self.node_env.copy(),flow_type)
            #This adds it to the environment for printing
            array_push(self.node_env,new_env)
            #for control flow manipulation
            array_push(self.children_envs[env_type],new_env)
            #array_push(self.children_envs[env_type],new_env) 
        else:
            new_env = Seq(self,self.node_env.copy(),root)
        return new_env

    def get_node_env(self):
        return self.node_env

    def output_all_nodes(self):
        #return self.all_nodes
        env = self
        while env.parent_env != None:
            env = env.parent_env
        return env.all_nodes

    # An annoyingingly necessary function because of the way
    # function nodes were implemented
    # Hopefully all these annoyances don't hurt me 희망밖에 없음
    def add_fn_dependency(self,name,dependency):
        self.children_envs["Function"][name] = dependency

    def output_env(self,output_str=[]):
        # This will ensure that any FuncSeqNode is transferrable across graphs
        for fn in [n for n in self.all_nodes if isinstance(n,FuncSeqNode)]:
            if (fn.get_func_name() not in self.children_envs["Function"] and 
                fn.get_dependency() != None):
                self.children_envs["Function"][fn.get_func_name()] = fn.get_dependency()
                
        for func_env in self.children_envs["Function"].values():
            func_env.output_env(output_str)
        #for func_name,func_env in self.children_envs["Function"].items():
        #    func_prototype = "function {}(".format(func_name)
        #    for param in func_env.get_params().values():
        #        func_prototype += param.get_name()
        #        if param.get_value() != None: #i.e. the param has a default
        #            func_prototype += "={},".format(param.get_value())
        #            param.set_value(param.get_name()) # this is a hacky
        #                                              # scumbag trick to redirect
        #                                              # subsequent nodes correctly
        #                                              # but it works cleanly tbh
        #                                              # 좆머거라
        #        else:
        #            func_prototype += ","
        #    func_prototype += ")\n{"
        #    output_str.append(func_prototype)
        #    func_env.output_env(output_str)
        #    ret_value = func_env.get_node("RETURN")
        #    if ret_value != None:
        #        output_str.append("return {};".format(ret_value.get_name()))
        #    output_str.append("}")

        for value in self.node_env.values():
            value.get_expression(output_str)
        #for value in self.node_env.values():
        #    value.get_expression(output_str)

class FunctionBranch(Seq):
    def __init__(self,parent,rel_db,func_name,node_env=None):
        self.func_name = func_name
        self.relational_database = rel_db
        self.node_env = node_env
        if self.node_env == None:
            self.node_env = {}
        self.root = None 
        self.parent_env = parent
        self.all_nodes = []
        self.params = {}
        #I'm assuming functions can't have functions. something's gonna break if not
        self.children_envs = {"Function":{},
                              "Flow":{}}

    def set_params(self,params):
        self.params = params

    def get_params(self):
        ret_dic = {}
        for k,v in self.params.items():
            ret_dic[k] = self.get_relation(v)
        return ret_dic

    def output_env(self,output_str=[]):
        func_prototype = "function {}(".format(self.func_name)
        for param in self.get_params().values():
            func_prototype += param.get_name()
            if param.get_value() != None:
                func_prototype += "={},".format(param.get_value())
                param.set_value(param.get_name())
            else:
                func_prototype += ","
        func_prototype += "){\n"
        output_str.append(func_prototype)
        for func_env in self.children_envs["Function"].values():
            func_env.output_env(output_str)
        for value in self.node_env.values():
            value.get_expression(output_str)
        output_str.append("}")

#FlowBranches are kind a blend of Seq and SeqNodes 
class FlowBranch(Seq):
    def __init__(self,parent,rel_db,node_env=None,flow_type=None):
        self.relational_database = rel_db
        self.flow_type = flow_type
        self.node_env = node_env
        if self.node_env == None:
            self.node_env = {}
        self.init_env = self.node_env.copy()
        self.root = None 
        self.parent_env = parent
        self.all_nodes = []
        self.params = {}
        self.children_envs = {"Flow":{}}
        self.conditional_statements = None
        self.conditional_variable = None
        self.defined_objects = {}

        self.catch_types = None
        self.catch_var = None

        self.foreach_var = None
        self.by_ref_var = False

        self.for_var = None

        self.cases = None

    def set_condition(self,conditional_statements,conditional_variable):
        #conditional statements is a list in the case of a switch statement
        self.conditional_statements = conditional_statements
        self.conditional_variable = conditional_variable

    # this is a thing specific to try catch statements. try{...}catch(ThisException | That $e){}
    # we gotta get this and that this object is getting too big for my preferences but 
    # writing it like this drastically reduces the amount of codethat needs to be written
    # and eases the pain of having a trillion different classes.
    def set_catch_types_var(self,types,var):
        self.catch_types = types
        self.catch_var = var

    def get_flow_type(self):
        return self.flow_type

    def set_foreach_var(self,var):
        self.foreach_var = var

    def set_for_var(self,var):
        self.for_var = var

    def set_by_ref_foreach_var(self,by_ref:bool):
        self.by_ref_var = by_ref

    def set_switch_cases(self,cases:list):
        self.cases = cases
        
    def get_expression(self,output_str=[]):
        ret_expr = ""
        if self.flow_type == "if": 
            tmp = []
            self.conditional_statements.get_expression(tmp)
            cond_expr = "\n".join(tmp)
            ret_expr = "{ce} \nif({cv})".format(ce=cond_expr,
                                                cv=self.conditional_variable)

        elif self.flow_type == "elseif":
            tmp = []
            self.conditional_statements.get_expression(tmp)
            cond_expr = "\n".join(tmp)
            ret_expr = "{ce} \nelseif({cv})".format(ce=cond_expr,
                                                cv=self.conditional_variable)

        elif self.flow_type == "else":
            ret_expr = "else"

        elif self.flow_type == "try":
            ret_expr = "try"

        elif self.flow_type == "catch":
            ret_expr = "catch ("
            for t in self.catch_types:
                ret_expr += t + " | "
            ret_expr = ret_expr[:-2] + "$" + self.catch_var + ")"

        elif self.flow_type == "switch":
            tmp = []
            for stmt in self.conditional_statements:
                stmt.get_expression(tmp)
            cond_expr = "\n".join(tmp)
            ret_expr = "{ce}\nswitch({cv})".format(ce=cond_expr,
                                             cv=self.conditional_variable)

        elif self.flow_type == "case":
            ret_expr = "case {cv}:\n".format(cv=self.conditional_variable)

        elif self.flow_type == "case_default":
            ret_expr = "default:\n".format(cv=self.conditional_variable)

        elif self.flow_type == "foreach":
            tmp = []
            self.conditional_statements.get_expression(tmp)
            cond_expr = "\n".join(tmp)
            cv = self.foreach_var
            if self.by_ref_var:
                tmp_foreach_var="&$"+self.foreach_var
            else:
                tmp_foreach_var="$"+self.foreach_var
            ret_expr = "{ce}\nforeach ({v} as {cv})".format(ce=cond_expr,
                                                            v=self.conditional_variable,
                                                            cv=tmp_foreach_var)

        elif self.flow_type == "while":
            tmp = []
            self.conditional_statements.get_expression(tmp)
            cond_expr = "\n".join(tmp)
            ret_expr = "{ce} \nwhile({cv})".format(ce=cond_expr,
                                                   cv=self.conditional_variable)

        # This one is gonna have to break some of the standards we have set 
        elif self.flow_type == "for":
            tmp = []
            init = self.conditional_statements[0]
            init.get_expression(tmp)
            init = "".join(tmp)
            cond = self.conditional_statements[1]
            loop = self.conditional_statements[2]
            ret_expr = "for({i}{c};{l})".format(i=init,
                                             c=cond,
                                             l=loop)

        if self.flow_type != "case" and self.flow_type != "case_default":
            ret_expr += "{\n"
            tmp_v = []
            for value in self.node_env.values():
                if value not in self.parent_env.node_env.values():
                    value.get_expression(tmp_v)
            body = "\n".join(tmp_v)
            ret_expr += body + "\n}"
        else:
            tmp_v = []
            for value in self.node_env.values():
                if value not in self.parent_env.node_env.values():
                    value.get_expression(tmp_v)
            body = "\n".join(tmp_v)
            ret_expr += body
        output_str.append(ret_expr)

        #flow_branches=[i for i in self.node_env.values() if isinstance(i,FlowBranch)]
        #if len(flow_branches) > 0:
        #    for fb in flow_branches:
        #        fb.get_expression(output_str)
        return
