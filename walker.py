import os
import json
import sys
import subprocess
import string
import secrets
import codecs
from sequence import *
from seqNodes import *

new_name = lambda: ''.join(secrets.choice(string.ascii_letters + string.digits) for _ in range(8))

SG_Variables = {"_GET":None,
                "_POST":None,
                "_FILES":None,
                "_COOKIE":None,
                "_REQUEST":None,
                "_SERVER":None,
                "_SESSION":None,}

class Walker():
    def __init__(self,debug):
        with open('php_lang_description.json', 'r') as f:
            data = json.load(f)
        # These are the most basic OpNodes
        self.middle_op_nodes = data['OpNodes']['MiddleOpNodes']
        self.front_op_nodes = data['OpNodes']['FrontOpNodes']
        self.inner_op_nodes = data['OpNodes']['InnerOpNodes']
        self.rear_op_nodes = data['OpNodes']['RearOpNodes']
        self.debug = debug
        self.file_string = None
        self.master = Seq()
        self.dangling_call = True
        self.curr_obj_name = None

    def get_node_type(self, node):
        return node['nodeType']

    def parse_file_string(self,start_token,stop_token):
        return self.file_string[start_token:stop_token+1]

    # Let me make this very clear. Return Statements Return Nodes
    # Easy.
    # The confusing part is the relational database and it's not that bad when
    # you actually take the time to think about it.
    def eval_node(self,node, env, class_id=""):
        if self.debug == True: print("eval_node:", self.get_node_type(node))

        node_type = self.get_node_type(node)
        if node_type in self.middle_op_nodes:
            return self.eval_middle_op(node,env,class_id)
        elif node_type in self.front_op_nodes:
            return self.eval_front_op(node,env,class_id)
        elif node_type in self.rear_op_nodes:
            return self.eval_rear_op(node,env,class_id)
        elif node_type in self.inner_op_nodes:
            return self.eval_inner_op(node,env,class_id)
        elif (node_type == "Scalar_Int" or
              node_type == "Scalar_LNumber" or
              node_type == "Scalar_Float"):
            return self.eval_constant(node,env,class_id)
        elif (node_type == "Stmt_Declare" or
              node_type == "Expr_Eval" or
              node_type == "Stmt_Use" or
              node_type == "Stmt_Interface" or
              node_type == "Stmt_Namespace" or 
              node_type == "Expr_Isset" or
              node_type == "Stmt_Enum"): #change this one if it becomes relevant
            return self.eval_parse(node,env,class_id)
        else:
            method = getattr(self, f"eval_{node_type}",self.eval_default)
            return method(node,env,class_id)

    def eval_Stmt_Expression(self,node,env,class_id=""):
        self.dangling_call = True
        return self.eval_node(node['expr'],env)

    def eval_Expr_Assign(self,node,env,class_id=""):
        self.dangling_call = False
        # CTSU
        # I don't really know here. If we rewrite an environment variable
        # the original statement is removed. Maybe we want a second variable
        # to take the value/statement of the old variable
        var = node['var']
        expr = node['expr']
        assert(self.get_node_type(var) == "Expr_Variable" or
               self.get_node_type(var) == "Expr_PropertyFetch" or
               self.get_node_type(var) == "Expr_ArrayDimFetch" or
               self.get_node_type(var) == "Expr_List") # I don't even understand
        name = None
        if self.get_node_type(var) == "Expr_Variable":
            name = var['name']
        elif self.get_node_type(var) == "Expr_PropertyFetch":
            name = []
            while(self.get_node_type(var)) == "Expr_PropertyFetch":
                name.append(var['name']['name'])
                var = var['var']
            name.append(var['name'])
            name.reverse()
            name = "->".join(f"{x}" for x in name)
        elif self.get_node_type(var) == "Expr_ArrayDimFetch":
            # To be true to the original, we really should make a copy of a leafnodearray
            # but idrgafatp
            array_var = self.eval_node(var['var'],env)
            array_dim = None
            if var['dim'] != None:
                array_dim = self.eval_node(var['dim'],env)
            else:
                null_node = GenericSeqNode("null",None)
                self.master.check_in(null_node)
                array_dim = null_node

        if (self.get_node_type(expr) == "Expr_New" or 
            self.get_node_type(expr) == "Expr_MethodCall" or
            self.get_node_type(expr) == "Expr_StaticCall"):
            self.curr_obj_name = name
        evaled_expr = self.eval_node(expr, env)
        if self.get_node_type(var) == "Expr_ArrayDimFetch":
            # this is confusing, but we can't know the type of the 
            # variable from the function argument, but by virtue of this
            # being an array operation, we will have to retroactively
            # reassign the array_var to be an array type seqnode if 
            # it is not already ihtj
            if not isinstance(array_var,ArraySeqNode):
                array_var = ArraySeqNode(self.master.get_rel_db())
                env.set_node(node['var']['var']['name'],array_var)
            array_var.add_value(
                    self.master.set_relation(array_dim),
                    self.master.set_relation(evaled_expr))
        else:
            env.set_node(name,evaled_expr)
        return

    #Property fetch, class declaration. these are all gonna be a little bit dirty for now
    def eval_Expr_PropertyFetch(self,node,env,class_id=""):
        print(json.dumps(node,indent=4))
        obj_name = None
        ret_node = None
        if self.get_node_type(node['var']) != "Expr_Variable":
            obj_name = self.eval_node(node['var'],env)
        else:
            obj_name = node['var']['name']
        property_name = node['name']['name']
        full_name = obj_name+property_name
        if full_name in env.get_node_env():
            ret_node = env.get_node(full_name)
        else:
            ret_node = PropertyFetchNode(self.master.get_rel_db(),
                                         obj_name,
                                         property_name,
                                         env.get_object(obj_name))
            self.master.check_in(ret_node)
            env.set_node(full_name,ret_node)
        return ret_node

    def eval_middle_op(self,node,env,class_id=""):
        node_type = self.get_node_type(node)
        #let's start here because this is the easiest
        #instead of giving the actual nodes left_value and right_value to 
        #ret_node, we are going to give left_value, right_value to the 
        #relational database and give ret_node the locations of these nodes
        #this requires that every node has access to the relational database
        #we'll go in verbose, slow motion here. don't hurt yourself
        #Basically, every time a node appears, we want it in the database

        #Get the ASnodes 
        left_value = self.eval_node(node['left'],env)
        right_value = self.eval_node(node['right'],env)
        #Store the nodesi n the database and get their keys
        left_relation_key = self.master.set_relation(left_value)
        right_relation_key = self.master.set_relation(right_value)
        #Define a new opnode with pointers to the ASNodes and the database
        ret_node = MiddleOpNode(self.master.get_rel_db(),
                                left_relation_key,
                                right_relation_key,
                                self.middle_op_nodes[node_type],)
        self.master.check_in(ret_node)
        return ret_node

    def eval_front_op(self,node,env,class_id=""):
        node_type = self.get_node_type(node)
        node_value = None
        if node_type == "Stmt_Unset":
            for v in node['vars']:
                if v['nodeType'] == "Expr_PropertyFetch":
                    node_value = env.get_node(v['var']['name'])
                    node_value = GenericSeqNode(node_value.get_name()+"->"+v['name']['name'],None)
                    node_relation_key = self.master.set_relation(node_value)
                    ret_node =  FrontOpNode(self.master.get_rel_db(),
                                            node_relation_key,
                                            self.front_op_nodes[node_type],)
                    self.master.check_in(ret_node)
                    env.set_node(node=ret_node)
                else:
                    node_value = env.get_node(v['name'])
                    node_relation_key = self.master.set_relation(node_value)
                    ret_node =  FrontOpNode(self.master.get_rel_db(),
                                            node_relation_key,
                                            self.front_op_nodes[node_type],)
                    self.master.check_in(ret_node)
                    env.set_node(node=ret_node)
            return
        else:
            if 'expr' in node:
                node_value = self.eval_node(node['expr'],env)
            else:
                node_value = self.eval_node(node['var'],env)
            node_relation_key = self.master.set_relation(node_value)
            ret_node =  FrontOpNode(self.master.get_rel_db(),
                                    node_relation_key,
                                    self.front_op_nodes[node_type],)
            self.master.check_in(ret_node)
            return ret_node

    def eval_inner_op(self,node,env,class_id):
        node_type = self.get_node_type(node)
        print(json.dumps(node,indent=4))
        node_value = self.eval_node(node['expr'],env)
        node_relation_key = self.master.set_relation(node_value)
        ret_node = InnerOpNode(self.master.get_rel_db(),
                               node_relation_key,
                               self.inner_op_nodes[node_type],)
        env.set_node(node=ret_node)
        self.master.check_in(ret_node)
        return 


    def eval_rear_op(self,node,env,class_id):
        node_type = self.get_node_type(node)
        node_value = self.eval_node(node['var'],env)
        node_relation_key = self.master.set_relation(node_value)
        ret_node = RearOpNode(self.master.get_rel_db(),
                              node_relation_key,
                              self.rear_op_nodes[node_type],)
        env.set_node(node=ret_node)
        self.master.check_in(ret_node)
        return ret_node

    def eval_constant(self,node,env,class_id):
        ret_node = GenericSeqNode(node['value'],self.get_node_type(node))
        self.master.check_in(ret_node)
        return ret_node

    def eval_name_fullyqualified(self,node,env,class_id):
        ret_node = GenericSeqNode("\\" + node['name'],self.get_node_type(node))
        self.master.check_in(ret_node)
        return ret_node

    def eval_Scalar_String(self,node,env,class_id):
        # We need to get the literal string here: repr
        ret_node = GenericSeqNode(repr(node['value']),self.get_node_type(node))
        self.master.check_in(ret_node)
        return ret_node

    def eval_Scalar_InterpolatedString(self,node,env,class_id):
        string_of_nodes = []
        for part in node['parts']:
            if self.get_node_type(part) == "InterpolatedStringPart":
                string_of_nodes.append(repr(part['value']))
            else:
                # watch me now
                string_of_nodes.append(self.master.set_relation(self.eval_node(part,env)))
        ret_node = InterpolatedStringNode(self.master.get_rel_db(),
                                          string_of_nodes,)
        return ret_node
        
    def eval_Scalar_MagicConst_Dir(self,node,env,class_id):
        ret_node = GenericSeqNode("__DIR__",self.get_node_type(node))
        self.master.check_in(ret_node)
        return ret_node

    def eval_Scalar_MagicConst_File(self,node,env,class_id):
        ret_node = GenericSeqNode("__FILE__",self.get_node_type(node))
        self.master.check_in(ret_node)
        return ret_node

    def eval_Expr_Variable(self,node,env,class_id):
        # Note, we only return nodes (mainly)
        # we only set the relationship on return values
        return env.get_node(node['name'])

    def eval_Expr_FuncCall(self,node,env,class_id):
        func_name = node['name']['name']
        dependency = None
        if func_name in self.master.get_defined_functions():
            dependency = self.master.get_defined_functions()[func_name]
        params = []
        args = node['args']
        if len(args) != 0:
            for v in args:
                if 'value' not in v:
                    params.append(self.master.set_relation(self.eval_node(v,env)))
                else:
                    params.append(self.master.set_relation(self.eval_node(v['value'],env)))
                #params.append(self.eval_node(v['value'],env))
        ret_node = FuncSeqNode(self.master.get_rel_db(),
                                func_name,
                                params,
                                dependency)
        self.master.check_in(ret_node)
        if self.dangling_call:
            self.dangling_call = False
            env.set_node(node=ret_node)
        return ret_node 

    def eval_Stmt_Function(self,node,env,class_id):
        # We simply want to build the internal environment of the function.
        # Chill out. it's good. for now, the undefined parameters are just
        # null nodes. when the function is called, we'll simply update
        # the environment. easy peasy. -lebron james
        func_name = node['name']['name']
        func_env = env.spawn_env("Function",func_name)
        func_params = {}
        if node['params'] != None:
            for p in node['params']:
                if p['default'] == None:
                    empty_node = GenericSeqNode(None,'param')
                    self.master.check_in(empty_node)
                    func_params[p['var']['name']] = empty_node
                else:
                    v = self.eval_node(p['default'],func_env)
                    v.set_type('param')
                    func_params[p['var']['name']] = v

        for k,v in func_params.items():
            func_env.set_node(k,v)
        for k,v in func_params.items():
            func_params[k]=self.master.set_relation(v)
        func_env.set_params(func_params)

        for stmt in node['stmts']:
            self.eval_node(stmt,func_env)
        return

    def eval_Stmt_Class(self,node,env,class_id):
        #print(json.dumps(node,indent=4))
        #class_name = node['name']['name']
        #properties = []
        #methods = []
        #for stmt in node['stmts']:
        #    if stmt['nodeType'] == "Stmt_Property":
        #        print(stmt['props'][0]['name']['name'])
        #quit()
        # This is dirty, but come back to this later if time permits
        #print(json.dumps(node,indent=4))
        start_token = node['attributes']['startFilePos']
        end_token = node['attributes']['endFilePos']
        expression_string = self.parse_file_string(start_token,end_token)
        class_name = node['name']['name']
        self.master.add_class_def(class_name,expression_string)
        return

    #def eval_Stmt_Trait(self,node,env,class_id):
    #    ...

    def eval_Expr_New(self,node,env,class_id):
        args = node['args']
        class_name = node['class']['name']
        arg_list = []
        if len(args) != 0:
            for v in args:
                a = self.master.set_relation(self.eval_node(v['value'],env))
                arg_list.append(a)
        class_def = self.master.get_class_def(class_name) # this is just a string
        ret_node = ObjDecSeqNode(self.master.get_rel_db(),
                                  self.curr_obj_name,
                                  class_name,
                                  arg_list,
                                  class_def,)
        if self.curr_obj_name != None:
            self.master.add_object(self.curr_obj_name,ret_node)
        self.master.check_in(ret_node)
        self.curr_obj_name = None
        return ret_node

    def eval_Expr_StaticCall(self,node,env,class_id):
        class_name = node['class']['name']
        call_name = node['name']['name']
        args = node['args']
        arg_list = []
        if len(args) != 0:
            for v in args:
                a = self.master.set_relation(self.eval_node(v['value'],env))
                arg_list.append(a)
        class_def = self.master.get_class_def(class_name)
        ret_node = StaticCallSeqNode(self.master.get_rel_db(),
                                  self.curr_obj_name,
                                  class_name+"::"+call_name,
                                  arg_list,
                                  class_def,)
        if self.curr_obj_name != None:
            self.master.add_object(self.curr_obj_name,ret_node)
        self.master.check_in(ret_node)
        self.curr_obj_name = None
        return ret_node

    def eval_Expr_MethodCall(self,node,env,class_id):
        #this is gonna be a dirty manipulation of the Expr_StaticCall
        dependency = None
        obj_name = ""
        if self.get_node_type(node['var']) == "Expr_StaticCall":
            dependency = self.eval_node(node['var'],env)
        if 'name' in node['var']:
            obj_name = node['var']['name']
        else:
            obj_name = new_name()
            self.curr_obj_name = obj_name
            obj_dec_node = self.eval_node(node['var'],env)
            self.curr_obj_name = None
            env.set_node(obj_name,obj_dec_node)
        method_name = node['name']['name']
        args = node['args']
        arg_list = []
        if len(args) != 0:
            for v in args:
                a = self.master.set_relation(self.eval_node(v['value'],env))
                arg_list.append(a)
        if (not(env.get_env_type() == FlowBranch and env.get_flow_type() == "catch") and
            dependency == None):
            # No dependency if the object is the variable iterating
            # over the foreach loop because the dependency is taken
            # care of by the encompassing environment
            if ((type(env) == FlowBranch and env.flow_type == "foreach")
                and env.foreach_var == obj_name):
                dependency == None
            else:
                dependency = self.master.get_object(obj_name)
        # some dependencies are protected relationships, in this case,
        # the method call relies on the preceding object declaration. So, this is 
        # not added to the relational database as the method call will break without
        # the declaration
        ret_node = MethodSeqNode(self.master.get_rel_db(),
                                  obj_name,
                                  method_name,
                                  arg_list,
                                  dependency)
        self.master.check_in(ret_node)
        if self.curr_obj_name == None:
            env.set_node(node=ret_node)
        return ret_node

    def eval_Stmt_While(self,node,env,class_id):
        cond = self.eval_node(node['cond'],env)
        tmp_env = env.spawn_env("Flow",flow_type="while")
        tmp_env.set_condition(cond,cond.get_name())
        for stmt in node['stmts']:
            self.eval_node(stmt,tmp_env)
        return

    def eval_Stmt_If(self,node,env,class_id):
        cond = self.eval_node(node['cond'], env)
        tmp_env = env.spawn_env("Flow",flow_type="if")
        tmp_env.set_condition(cond,cond.get_name())
        if node['elseifs'] != None:
            for stmt in node['elseifs']:
                #this is env instead of tmp_env because the 
                #variables local to the if are not available to the elses
                self.eval_node(stmt,env)
        if node['else'] != None:
            #for stmt in node['else']['stmts']:
                #print(json.dumps(node['else'],indent=4))
            self.eval_node(node['else'],env)
        for stmt in node['stmts']:
            self.eval_node(stmt,tmp_env)
        return

    def eval_Stmt_ElseIf(self,node,env,class_id):
        cond = self.eval_node(node['cond'],env)
        tmp_env = env.spawn_env("Flow",flow_type="elseif")
        tmp_env.set_condition(cond,cond.get_name())
        for stmt in node['stmts']:
            self.eval_node(stmt,tmp_env)
        return

    def eval_Stmt_Else(self,node,env,class_id):
        tmp_env = env.spawn_env("Flow",flow_type="else")
        for stmt in node['stmts']:
            self.eval_node(stmt,tmp_env)
        return

    def eval_Stmt_TryCatch(self,node,env,class_id):
        tmp_env = env.spawn_env("Flow",flow_type="try")
        for stmt in node['catches']:
            self.eval_node(stmt,env)
        if node['finally'] != None:
            self.eval_node(node['finally'],env)
        for stmt in node['stmts']:
            self.eval_node(stmt,tmp_env)
        return

    def eval_Stmt_Catch(self,node,env,class_id):
        types = [i['name'] for i in node['types']]
        var = node['var']['name']
        tmp_env = env.spawn_env("Flow",flow_type="catch")
        tmp_env.set_catch_types_var(types,var)
        for stmt in node['stmts']:
            self.eval_node(stmt,tmp_env)
        return

    # this is a unique case, 
    # in PHP, a switch statement does not create a new scope UNLESS {} are used
    # let's put a pin in that for now though
    def eval_Stmt_Switch(self,node,env,class_id):
        conditions = []
        cond = self.eval_node(node['cond'],env)
        conditions.append(cond)
        switch_env = env.spawn_env("Flow",flow_type="switch")
        #wait to do this until the end when all conditions are collected
        #switch_env.set_condition(cond,cond.get_name())
        for case in node['cases']:
            case_env = None
            if case['cond'] != None:
                case_env = switch_env.spawn_env("Flow",flow_type="case")
                case_cond = self.eval_node(case['cond'],env)
                conditions.append(case_cond)
            else:
                case_env = switch_env.spawn_env("Flow",flow_type="case_default")
            case_env.set_condition(None,case_cond.get_name())
            for stmt in case['stmts']:
                #we should evaluate statements in parent environment scope but whateever
                self.eval_node(stmt,case_env)
        switch_env.set_condition(conditions,cond.get_name())

    # This is gonna suck
    # this needs a lot more work
    def eval_Stmt_Foreach(self,node,env,class_id):
        # This covers the basics
        cond = self.eval_node(node["expr"],env)
        tmp_env = env.spawn_env("Flow",flow_type="foreach")
        tmp_env.set_condition(cond,cond.get_name())
        for_var = node['valueVar']['name']
        by_ref = node['byRef']
        # This is a weird quirk of PHP where teh for_var is still in scope
        # after the loop's completion
        env.set_node(for_var,GenericSeqNode("$"+for_var))

        tmp_env.set_node(for_var,GenericSeqNode("$"+for_var))
        tmp_env.set_by_ref_foreach_var(by_ref)
        tmp_env.set_foreach_var(for_var)
        for stmt in node['stmts']:
            self.eval_node(stmt,tmp_env)
        return

        # This is dirty and should be done correctly, but we're kinda vibing too hard rn
        # to quit.
    def eval_Stmt_For(self,node,env,class_id):
        tmp_env = env.spawn_env("Flow",flow_type="for")
        self.eval_node(node["init"][0],tmp_env) # we at least need to evaluate the var
        for_var_name = node['init'][0]['var']['name']
        tmp_env.set_for_var(for_var_name)
        tmp_env.get_node(for_var_name).set_name("$"+for_var_name)
        init = tmp_env.get_node(for_var_name)
        #cond = self.eval_node(node["cond"][0],tmp_env)
        #loop = self.eval_node(node["loop"][0],tmp_env)
        #init = self.parse_file_string(node["init"][0]['attributes']['startFilePos'],
        #                              node["init"][0]['attributes']['endFilePos'],)
        cond = self.parse_file_string(node["cond"][0]['attributes']['startFilePos'],
                                      node["cond"][0]['attributes']['endFilePos'],)
        loop = self.parse_file_string(node["loop"][0]['attributes']['startFilePos'],
                                      node["loop"][0]['attributes']['endFilePos'],)
        tmp_env.set_condition([init,cond,loop],for_var_name)
        for stmt in node['stmts']:
            self.eval_node(stmt,tmp_env)
        return

    def eval_Expr_Throw(self,node,env,class_id):
        #figure this out later
        return

    def eval_Stmt_Finally(self,node,env,class_id):
        tmp_env = env.spawn_env("Flow",flow_type="finally")
        for stmt in node['stmts']:
            self.eval_node(stmt,tmp_env)
        return

    def eval_Expr_Array(self,node,env,class_id):
        ret_node = ArraySeqNode(self.master.get_rel_db())
        if len(node['items']) != 0:
            for item in node['items']:
              key, value = self.eval_node(item,env)  
              if key == None:
                  ret_node.add_value(key,
                                     self.master.set_relation(value))
              else:  
                  ret_node.add_value(self.master.set_relation(key),
                                     self.master.set_relation(value))
        self.master.check_in(ret_node)
        return ret_node

    # This is gonna be the only one that returns something different
    # This is gonna return a key:str and a value:ASNode
    def eval_ArrayItem(self,node,env,class_id):
        key = None
        if node['key'] != None:
            key = self.eval_node(node['key'],env)
        value = self.eval_node(node['value'],env)
        return (key,value)

    def eval_Expr_ConstFetch(self,node,env,class_id):
        node_type = self.get_node_type(node)
        ret_node = None
        #really weird edge case
        if self.get_node_type(node['name']) == "name_Relative":
            value = node['name']['name']
            ret_node = GenericSeqNode("namespace\\{}".format(value),node_type)
        else:
            value = node['name']['name']
            if value == "true":
                ret_node = GenericSeqNode(True,node_type)
            elif value == "false":
                ret_node = GenericSeqNode(False,node_type)
            else:
                ret_node = GenericSeqNode(value,node_type)
        self.master.check_in(ret_node)
        return ret_node

    def eval_Expr_ClassConstFetch(self,node,env,class_id):
        node_type = self.get_node_type(node)
        name = node['class']['name']+"::"+node['name']['name']
        ret_node = GenericSeqNode(name,node_type)
        self.master.check_in(ret_node)
        return ret_node

    def eval_Expr_ArrayDimFetch(self,node,env,class_id):
        var = self.eval_node(node['var'],env)
        dim = self.eval_node(node['dim'],env)
        if not isinstance(var,ArraySeqNode):
            #node_relation_key = self.master.set_relation(node_value)
            ret_node = FlexArrayDimFetchNode(self.master.get_rel_db(),
                                             var,
                                             dim)
            self.master.check_in(ret_node)
            return ret_node
        else:
            ret_node = var.get_value_by_key(dim.get_value())
            self.master.check_in(ret_node)
            return ret_node

    def eval_Stmt_Return(self,node,env,class_id):
        ret_val = self.eval_node(node['expr'],env)
        env.set_node(node=GenericCallSeqNode(self.master.get_rel_db,
                                             'return',ret_val))
        #ret_val = self.eval_node(node['expr'],env)
        #env.set_node("RETURN",ret_val)
        return 
    
    def eval_Expr_Exit(self,node,env,class_id):
        ret_val = self.eval_node(node['expr'],env)
        env.set_node(node=GenericCallSeqNode(self.master.get_rel_db,
                                             '',ret_val))
        return

    def eval_Expr_Clone(self,node,env,class_id):
        ret_val = self.eval_node(node['expr'],env)
        env.set_node(node=GenericCallSeqNode(self.master.get_rel_db,
                                             'clone',ret_val))
        return

    def eval_Expr_Yield(self,node,env,class_id):
        ret_val = self.eval_node(node['value'],env)
        env.set_node(node=GenericCallSeqNode(self.master.get_rel_db,
                                             'yield',ret_val))
        return

    def eval_Expr_YieldFrom(self,node,env,class_id):
        ret_val = self.eval_node(node['value'],env)
        env.set_node(node=GenericCallSeqNode(self.master.get_rel_db,
                                             'yield from',ret_val))
        return


    def eval_Expr_Closure(self,node,env,class_id):
        result = {}
        start_token = node['attributes']['startFilePos']
        end_token = node['attributes']['endFilePos']
        expression_string = self.parse_file_string(start_token,end_token)
        closure_node = GenericSeqNode(expression_string)
        self.master.check_in(closure_node)
        return closure_node

    def eval_Expr_Ternary(self,node,env,class_id):
        result = {}
        start_token = node['attributes']['startFilePos']
        end_token = node['attributes']['endFilePos']
        expression_string = self.parse_file_string(start_token,end_token)
        closure_node = GenericSeqNode(expression_string)
        self.master.check_in(closure_node)
        return closure_node

    def eval_VariadicPlaceholder(self,node,env,class_id):
        variadic_node = GenericSeqNode("...")
        self.master.check_in(variadic_node)
        return variadic_node

    def eval_Stmt_Nop(self,node,env,class_id):
        return

    def eval_Stmt_InlineHTML(self,node,env,class_id):
        return

    def eval_Expr_Include(self,node,env,class_id): #idk maybe include this later
        return

    def eval_Stmt_Break(self,node,env,class_id): 
        env.set_node(node=NonAssignmentNode("break;"))
        return

    def eval_Stmt_Echo(self,node,env,class_id): 
        for stmt in node['exprs']:
            result_node = self.eval_node(stmt,env)
            env.set_node(node=result_node)
        return 

    def eval_Expr_Print(self,node,env,class_id): 
        result_node = self.eval_node(node['expr'],env)
        env.set_node(node=result_node)
        return

    def eval_parse(self,node,env,class_id):
        start_token = node['attributes']['startFilePos']
        end_token = node['attributes']['endFilePos']
        expression_string = self.parse_file_string(start_token,end_token)
        env.set_node(node=NonAssignmentNode(expression_string))
        return

    def eval_default(self,node,_x,_y):
        print(json.dumps(node,indent=4))
        raise NotImplementedError(f"No handler for {node['nodeType']}")



    def analyze(self,target_file):
        with codecs.open(target_file,"r",encoding='utf-8',errors='ignore') as f:
            self.file_string = f.read()

        stmts = None
        try:
            #Convert to AST
            command = ["bash","./php_to_ast.sh",target_file]
            child = subprocess.Popen(command,
                             stdout=subprocess.PIPE,
                             stderr=subprocess.PIPE,
                             text=True)

            stdout,stderr = child.communicate(timeout=120)
            child.kill()
            stmts = json.loads(stdout)
        except Exception as e:
            return (False,self.master)

        if self.debug:
            for node in stmts:
                self.eval_node(node, self.master)
            return (False,self.master)
        else:
            try:
                for node in stmts:
                    self.eval_node(node, self.master)
                return (True,self.master)
            except Exception as e:
                return (False,self.master)

