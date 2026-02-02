from utils import *

########
# Each node needs access to the overall tree
# NO NODE should point DIRECTLY at a node!
# Nodes should ONLY be accessible via the environment
class ASNode():
    def get_rel_val(self,key):
        return self.rel_db[key]

class SeqNode(ASNode):
    ...

class FuncSeqNode(SeqNode):
    def __init__(self,rel_db,func_name,arguments:list,dependency):
        self.rel_db = rel_db
        self.func_name = func_name
        self.arguments = arguments
        self.dependency = dependency 
        self.name = new_name()
        # We don't need to print nodes more than once, but the system typicall does that
        # because of its verbose, context-preserving nature. This just makes it cleaner.
        self.printed = False

    def get_name(self):
        return self.name

    def set_name(self,name):
        self.name = name

    def get_dependency(self):
        return self.dependency

    def get_func_name(self):
        return self.func_name

    def get_expression(self,pp_env):
        if not self.printed:
            self.printed = True
            ret_expr = "{v} = {f}(".format(v=self.name,
                                           f=self.func_name)
            #I can make list comprehensions much much worse
            #don't complain ㅅㅂㄴㅁ
            args = [self.get_rel_val(a).get_expression(pp_env) for a in self.arguments]
            for a in args:
                ret_expr += a +","
            ret_expr += ");"
            pp_env.append(ret_expr)
        return self.name

class GenericCallSeqNode(SeqNode):
    def __init__(self,rel_db,call,argument):
        self.rel_db = rel_db
        self.call = call
        self.argument = argument
        self.printed = False

    def get_expression(self,pp_env):
        if not self.printed:
            self.printed = True
            if self.argument != None:
                arg = self.argument.get_expression(pp_env)
                ret_expr = "{c}({a});".format(c=self.call,a=arg)
                pp_env.append(ret_expr)
            else:
                ret_expr = "{};".format(self.call)
                pp_env.append(ret_expr)

class MethodSeqNode(SeqNode):
    def __init__(self,rel_db,obj_name,method_name,arguments:list,dependency):
        self.rel_db = rel_db
        self.obj_name = obj_name
        self.method_name = method_name
        self.arguments = arguments
        self.dependency = dependency
        self.name = new_name()
        self.printed = False

    def get_name(self):
        return self.name

    def set_name(self,name):
        self.name = name

    def get_dependency(self):
        return self.dependency

    def get_method_name(self):
        return self.method_name

    def get_expression(self,pp_env):
        if not self.printed:
            self.printed = True
            tmp_v = []
            ret_expr = ""
            if self.dependency != None:
                self.dependency.get_expression(tmp_v)
                ret_expr = "\n".join(tmp_v)
                ret_expr += "{v} = {on}->{mn}(".format(v=self.name,
                                                      on=self.dependency.get_name(),
                                                      mn=self.method_name)
            else:
                # This is special for try catch statements
                ret_expr += "{v} = ${on}->{mn}(".format(v=self.name,
                                                      on=self.obj_name,
                                                      mn=self.method_name)
            args = [self.get_rel_val(a).get_expression(pp_env) for a in self.arguments]
            for a in args:
                ret_expr += a +","
            ret_expr += ");"
            pp_env.append(ret_expr)
        return self.name

class PropertyFetchNode(SeqNode):
    def __init__(self,rel_db,obj_name,property_name,dependency):
        self.rel_db = rel_db
        self.obj_name = obj_name
        self.property_name = property_name
        self.dependency = dependency
        self.name = new_name()
        self.printed = False

    def get_name(self):
        return self.name

    def set_name(self,name):
        self.name = name

    def get_dependency(self):
        return self.dependency

    def get_property_name(self):
        return self.property_name

    def get_expression(self,pp_env):
        if not self.printed:
            self.printed = True
            tmp_v = []
            ret_expr = ""
            if self.dependency != None:
                self.dependency.get_expression(tmp_v)
                #raise ValueError("PropertyFetch")
                ret_expr = "\n".join(tmp_v)
                ret_expr += "\n{v} = {on}->{mn};".format(v=self.name,
                                                      on=self.dependency.get_name(),
                                                      mn=self.property_name)
            else:
                # This is special for try catch statements
                ret_expr += "\n{v} = ${on}->{mn};".format(v=self.name,
                                                      on=self.obj_name,
                                                      mn=self.property_name)
            #args = [a.get_expression(pp_env) for a in self.arguments]
            #for a in args:
            #    ret_expr += a +","
            #ret_expr += ");"
            pp_env.append(ret_expr)
        return self.name

class ObjDecSeqNode(SeqNode):
    def __init__(self,rel_db,obj_name,class_name,arguments:list,dependency):
        self.rel_db = rel_db
        self.obj_name = obj_name
        self.class_name = class_name
        self.arguments = arguments
        self.dependency = dependency
        self.name = new_name()
        self.printed = False

    def get_name(self):
        return self.name

    def set_name(self,name):
        self.name = name

    def get_dependency(self):
        return self.dependency

    def get_obj_name(self):
        return self.obj_name

    def get_class_name(self):
        return self.class_name

    def get_expression(self,pp_env):
        if not self.printed:
            self.printed = True
            ret_expr = ""
            if self.dependency != None:
                ret_expr += self.dependency + "\n"
            ret_expr += "{v} = new {c}(".format(v=self.name,
                                               c=self.class_name)
            args = [self.get_rel_val(a).get_expression(pp_env) for a in self.arguments]
            for a in args:
                ret_expr += a + ","
            ret_expr += ");"
            pp_env.append(ret_expr)
        return self.name

class StaticCallSeqNode(ObjDecSeqNode):
    def get_expression(self,pp_env):
        if not self.printed:
            self.printed = True
            ret_expr = ""
            if self.dependency != None:
                ret_expr += self.dependency + "\n"
            ret_expr += "{v} = {cn}(".format(v=self.name,
                                                   cn=self.class_name)
            args = [self.get_rel_val(a).get_expression(pp_env) for a in self.arguments]
            for a in args:
                ret_expr += a + ","
            ret_expr += ");"
            pp_env.append(ret_expr)
        return self.name

# GenericSeqNodes have NO DEPENDENCIES! and need no access to the relational database
class GenericSeqNode(SeqNode):
    def __init__(self,value,node_type=None,dependency=None):
        self.value = value
        self.type = node_type
        self.name = new_name()
        self.printed = False
        self.dependency = dependency
        #if self.type == "scalar_string":
        #    self.value = "\""+self.value+ "\"" 

    #def __str__(self):
    #    return str(self.value)

    def get_copy(self):
        copy = GenericSeqNode(self.value,node_type)
        copy.var = self.name
        return copy

    def set_type(self,node_type):
        self.type = node_type

    def get_type(self):
        return(self.type)

    def get_name(self):
        return self.name

    def set_name(self,name):
        self.name = name

    def get_value(self):
        return self.value

    def set_value(self,value):
        self.value = value

    def get_expression(self,pp_env=[]):
        if self.value == None:
            self.value = self.name
        if not self.printed:
            self.printed = True
            ret_expr = "{v} = {val};".format(v=self.name,
                                             val=self.value)
            pp_env.append(ret_expr)
        return self.name

class NonAssignmentNode(SeqNode):
    def __init__(self,value):
        self.value = value
    def get_expression(self,pp_env):
        pp_env.append(self.value)

class InterpolatedStringNode(GenericSeqNode):
    def __init__(self,rel_db,value:list,node_type=None):
        self.rel_db = rel_db
        self.value = value
        self.printed = False
        self.name = new_name()
        self.type = node_type

    def get_expression(self,pp_env):
        if not self.printed:
            self.printed = True
            ret_expr = "{v} = \"".format(v=self.name)
            for p in self.value:
                if isinstance(p,int):
                    p_val = self.get_rel_val(p)
                    ret_expr += p_val.get_expression(pp_env).replace("\'","")
                else:
                    ret_expr += p.replace("\'","")
            ret_expr += "\";"

            pp_env.append(ret_expr)
        return self.name

class ArraySeqNode(SeqNode):
    def __init__(self,rel_db):
        self.rel_db = rel_db
        self.value = {}
        self.name = new_name()
        self.printed = False

    #def __str__(self):
    #    return str(self.value)

    def add_value(self, key, value):
        if key == None:
            array_push(self.value,value)
        else:
            self.value[str(self.get_rel_val(key).get_value())] = value

    def get_value_by_key(self,key):
        assert(key in self.value)
        return ArrayValueNode(key,self.name,dependency=self)
        #return self.get_rel_val(self.value[key])

    def set_type(self,node_type):
        self.type = node_type

    def get_type(self):
        return(self.type)

    def get_name(self):
        return self.name

    def set_name(self,name):
        self.name = name

    def get_value(self):
        return self.value

    def set_value(self,value):
        self.value = value

    def get_expression(self,pp_env):
        if not self.printed:
            self.printed = True
            ret_expr = "{v} = array(".format(v=self.name)
            for k,v in self.value.items():
                ret_expr += "{key} => {value},".format(key=k,
                                                       value=self.get_rel_val(v).get_expression(
                                                           pp_env))
            ret_expr += ");"
            pp_env.append(ret_expr)
        return self.name

# This might be a little dirty idk it's late
class ArrayValueNode(GenericSeqNode):
    def __init__(self,value,arr_name,node_type=None,dependency=None):
        self.value = value
        self.arr_name = arr_name
        self.type = node_type
        self.name = new_name()
        self.printed = False
        self.dependency = dependency

    def get_expression(self,pp_env=[]):
        self.dependency.get_expression(pp_env)
        if self.value == None:
            self.value = self.name
        if not self.printed:
            self.printed = True
            ret_expr = "{v} = {a}[{k}];".format(v=self.name,
                                                a=self.arr_name,
                                                k=self.value)
            pp_env.append(ret_expr)
        return self.name



class FlexArrayDimFetchNode(SeqNode):
    def __init__(self,rel_db,var,dim):
        self.rel_db = rel_db
        self.var = var
        self.dim = dim
        self.name = new_name()
        self.printed = False
        
    def get_expression(self,pp_env): # this is just experimental. ykwtd
        if not self.printed:
            self.printed = True
            #node = self.get_rel_val(self.node_key)
            ret_expr = "{v} = {var}[{dim}];".format(v=self.name,
                                                    var=self.var.get_expression(pp_env),
                                                    dim=self.dim.get_expression(pp_env))
            pp_env.append(ret_expr)
        return self.name

class OPNode(ASNode):
    def getExpression(self):
        ...
    def getValue(self):
        ...
    def getNodeExpression(self):
        ...
    def addDependency(self,node:ASNode):
        if isinstance(node,ASNode):
            self.dependencies.append(node)

class MiddleOpNode(OPNode):
    def __init__(self,rel_db,left_key,right_key,value,node_type=None):
        self.rel_db = rel_db
        self.left_key = left_key
        self.right_key = right_key
        self.value = value
        self.dependencies = []
        self.name = new_name()
        self.type = node_type
        self.printed = False

    def get_name(self):
        return self.name

    def set_name(self,name):
        self.name = name

    def get_value(self):
        return self.value

    def set_value(self,value):
        self.value = value

    def set_type(self,node_type):
        self.type = node_type

    def get_type(self):
        return(self.type)

    def get_node_expression(self):
        return [self, self.left.get_node_expression(),self.right.get_node_expression()]

    def get_expression(self,pp_env): # this is just experimental. ykwtd
        if not self.printed:
            self.printed = True
            left = self.get_rel_val(self.left_key)
            right = self.get_rel_val(self.right_key)
            ret_expr = "{v} = {l} {op} {r};".format(v=self.name,
                                                    l=left.get_expression(pp_env),
                                                    op=self.value,
                                                    r=right.get_expression(pp_env))
            pp_env.append(ret_expr)
        return self.name

class FrontOpNode(OPNode):
    def __init__(self,rel_db,node_key,value):
        self.rel_db = rel_db
        self.node_key = node_key
        self.pos = 'f'
        self.value = value
        self.dependencies = []
        self.name = new_name()
        self.printed = False

    def get_name(self):
        return self.name

    def set_name(self,name):
        self.name = name

    def get_value(self):
        return self.value

    def set_value(self,value):
        self.value = value

    def set_type(self,node_type):
        self.type = node_type

    def get_type(self,node_type):
        return(self.type)

    def getNodeExpression(self):
        return [self, self.node.getNodeExpression()]

    def get_expression(self,pp_env): # this is just experimental. ykwtd
        if not self.printed:
            self.printed = True
            node = self.get_rel_val(self.node_key)
            ret_expr = "{v} = {op}({val});".format(v=self.name,
                                                   op=self.value,
                                                   val=node.get_expression(pp_env))
            pp_env.append(ret_expr)
        return self.name

# this is not correct
class InnerOpNode(FrontOpNode):
    def get_expression(self,pp_env):
        if not self.printed:
            self.printed = True
            node = self.get_rel_val(self.node_key)
            ret_expr = "{v} {op} ({val});".format(v=self.name,
                                                   op=self.value,
                                                   val=node.get_expression(pp_env))
            pp_env.append(ret_expr)
        return self.name

class RearOpNode(FrontOpNode):
    def get_expression(self,pp_env):
        if not self.printed:
            self.printed = True
            node = self.get_rel_val(self.node_key)
            ret_expr = "{v} = {val};".format(v=self.name,
                                                   val = node.get_expression(pp_env))
            ret_expr += "\n{v}{op};".format(v=self.name,
                                                  op=self.value)
            pp_env.append(ret_expr)
        return self.name


