import json
### Language description ###
#create a mapping of ast nodes to tree points
#create leafNodes 
#create opNodes
#pretty print instructions
#custom tree walker
'''
It's gonna work like this:
we have a language description of PHP to our tree IR. this language description includes a mapping of the AST
Nodes to the tree points, leafnodes, specialnodes, and opnodes. the leafnodes and opnodes will contain instructions for 
pretty printing. Each additional language will need a custom tree walker. 
'''

class node():
    def __init__(self, name, TreePoint):
        self.name = name
        self.TreePoint = TreePoint

def php_lang_description_generator():
    TreePoints = []
    opNodes = []
    leafNodes = []
    specialNodes = []
    nodes = []

    nodes.append(node('Stmt_Expression','stmt_expr'))
    nodes.append(node('Scalar_LNumber','scalar_lnumber'))
    nodes.append(node('Stmt_Const','stmt_const'))
    nodes.append(node('Scalar_Int','scalar_int'))
    nodes.append(node('Scalar_Float','scalar_float'))
    nodes.append(node('Scalar_String','scalar_string'))
    nodes.append(node('Expr_Eval','expr_eval'))
    nodes.append(node('Expr_Assign','expr_assign'))
    nodes.append(node('Expr_Variable','expr_variable'))
    nodes.append(node('Expr_Closure','expr_closure'))
    nodes.append(node('Expr_New','expr_new'))
    nodes.append(node('Expr_Variable','expr_variable'))
    nodes.append(node('Expr_StaticCall','expr_staticcall'))
    nodes.append(node('Scalar_InterpolatedString','scalar_interpolatedstring'))
    nodes.append(node('InterpolatedStringPart','interpolatedstringpart'))
    nodes.append(node('Expr_PropertyFetch','expr_propertyfetch'))
    nodes.append(node('Expr_BinaryOp_Coalesce','expr_binaryop_coalesce'))
    nodes.append(node('Expr_BinaryOp_Plus','expr_binaryop_plus'))
    nodes.append(node('Expr_BinaryOp_LogicalAnd','expr_binaryop_logicaland'))
    nodes.append(node('Expr_BinaryOp_Greater','expr_binaryop_greater'))
    nodes.append(node('Expr_BinaryOp_Concat','expr_binaryop_concat'))
    nodes.append(node('Expr_BinaryOp_Mod','expr_binaryop_mod'))
    nodes.append(node('Expr_BinaryOp_Identical','expr_binaryop_identical'))
    nodes.append(node('Expr_BinaryOp_Minus','expr_binaryop_minus'))
    nodes.append(node('Expr_BinaryOp_Mul','expr_binaryop_mu'))
    nodes.append(node('Expr_BinaryOp_Div','expr_binaryop_div'))
    nodes.append(node('Expr_BinaryOp_GreaterOrEqual','expr_binaryop_greaterorequal'))
    nodes.append(node('Expr_BinaryOp_Smaller','expr_binaryop_smaller'))
    nodes.append(node('Expr_BinaryOp_SmallerOrEqual','expr_binaryop_smallerorequal'))
    nodes.append(node('Expr_BinaryOp_BitwiseAnd','expr_binaryop_bitwiseand'))
    nodes.append(node('Expr_BitwiseNot','expr_bitwisenot'))
    nodes.append(node('Expr_BinaryOp_LogicalOr','expr_binaryop_logicalor'))
    nodes.append(node('Expr_BinaryOp_BooleanOr','expr_binaryop_booleanor'))
    nodes.append(node('Expr_BinaryOp_BooleanAnd','expr_binaryop_booleanand'))
    nodes.append(node('Expr_BinaryOp_Equal','expr_binaryop_equal'))
    nodes.append(node('Expr_BinaryOp_NotEqual','expr_binaryop_notequal'))
    nodes.append(node('Expr_BinaryOp_NotIdentical','expr_binaryop_notidentical'))
    nodes.append(node('Expr_BinaryOp_BitwiseXor','expr_binaryop_bitwisexor'))
    nodes.append(node('Expr_BinaryOp_BitwiseOr','expr_binaryop_bitwiseor'))
    nodes.append(node('Expr_AssignRef','expr_assignref'))
    nodes.append(node('Expr_BooleanNot','expr_booleannot'))
    nodes.append(node('Expr_UnaryMinus','expr_unaryminus'))
    nodes.append(node('ArrayItem','arrayitem'))
    nodes.append(node('Expr_Array','expr_array'))
    nodes.append(node('Stmt_Function','stmt_function'))
    nodes.append(node('Stmt_If','stmt_if'))
    nodes.append(node('Stmt_Switch','stmt_switch'))
    nodes.append(node('Stmt_Break','stmt_break'))
    nodes.append(node('Stmt_TryCatch','stmt_trycatch'))
    nodes.append(node('Stmt_Catch','stmt_catch'))
    nodes.append(node('Stmt_Finally','stmt_finally'))
    nodes.append(node('Expr_Throw','expr_throw'))
    nodes.append(node('Stmt_Continue','stmt_continue'))
    nodes.append(node('Stmt_Foreach','stmt_foreach'))
    nodes.append(node('Stmt_For','stmt_for'))
    nodes.append(node('Stmt_While','stmt_while'))
    nodes.append(node('Stmt_Nop','stmt_nop'))
    nodes.append(node('Stmt_Echo','stmt_echo'))
    nodes.append(node('Stmt_ElseIf','stmt_elseif'))
    nodes.append(node('Stmt_Else','stmt_else'))
    nodes.append(node('Stmt_Return','stmt_return'))
    nodes.append(node('Expr_Print','expr_print'))
    nodes.append(node('Expr_Yield','expr_yield'))
    nodes.append(node('Expr_Include','expr_include'))
    nodes.append(node('Expr_MethodCall','expr_methodcall'))
    nodes.append(node('Expr_PropertyFetch','expr_propertyfetch'))
    nodes.append(node('Stmt_Use','stmt_use'))
    nodes.append(node('Stmt_ClassMethod','stmt_classmethod'))
    nodes.append(node('Expr_ConstFetch','expr_constfetch'))
    nodes.append(node('Expr_ClassConstFetch','expr_classconstfetch'))
    nodes.append(node('Expr_ArrayDimFetch','expr_arraydimfetch'))
    nodes.append(node('Expr_Isset','expr_isset'))
    nodes.append(node('Expr_FuncCall','expr_funccall'))
    nodes.append(node('Stmt_InlineHTML','stmt_inlinehtml'))
    nodes.append(node('Scalar_MagicConst_Dir','scalar_magicconst_dir'))
    nodes.append(node('Scalar_MagicConst_File','scalar_magicconst_file'))
    nodes.append(node('Stmt_Class','stmt_class'))
    nodes.append(node('Stmt_Interface','stmt_interface'))
    nodes.append(node('Stmt_Trait','stmt_trait'))
    nodes.append(node('Stmt_Namespace','stmt_namespace'))
    nodes.append(node('Expr_StaticCall','expr_staticcall'))
    nodes.append(node('Stmt_Declare','stmt_declare'))
    nodes.append(node('Stmt_Unset','stmt_unset'))
    nodes.append(node('Expr_ErrorSuppress','expr_errorsuppress'))
    nodes.append(node('Name','name'))
    nodes.append(node('Expr_AssignOp_Mul','expr_assignop_mul'))
    nodes.append(node('Expr_AssignOp_Minus','expr_assignop_minus'))
    nodes.append(node('Expr_AssignOp_Plus','expr_assignop_plus'))
    nodes.append(node('Expr_AssignOp_Div','expr_assignop_div'))
    nodes.append(node('Expr_PostInc','expr_postinc'))
    nodes.append(node('Expr_PostDec','expr_postdec'))
    nodes.append(node('Expr_Cast_Int','expr_cast_int'))
    nodes.append(node('Expr_Cast_String','expr_cast_string'))
    nodes.append(node('Expr_Cast_Double','expr_cast_double'))
    nodes.append(node('Expr_Cast_Bool','expr_cast_bool'))
    nodes.append(node('Expr_Empty','expr_empty'))
    nodes.append(node('Expr_Ternary','expr_ternary'))
    nodes.append(node('Expr_PreInc','expr_preinc'))
    nodes.append(node('Expr_PreDec','expr_predec'))
    nodes.append(node('Name_FullyQualified','name_fullyqualified'))
    nodes.append(node('Expr_List','expr_list'))

    opNodes = {"MiddleOpNodes":{},"FrontOpNodes":{},"RearOpNodes":{},"InnerOpNodes":{}}
    opNodes['MiddleOpNodes']['expr_binaryop_plus'] = "+"
    opNodes['MiddleOpNodes']['expr_binaryop_coalesce'] = "??"
    opNodes['MiddleOpNodes']['expr_binaryop_logicaland'] = "and"
    opNodes['MiddleOpNodes']['expr_binaryop_greater'] = ">"
    opNodes['MiddleOpNodes']['expr_binaryop_concat'] = "."
    opNodes['MiddleOpNodes']['expr_binaryop_mod'] = "%"
    opNodes['MiddleOpNodes']['expr_binaryop_identical'] = "==="
    opNodes['MiddleOpNodes']['expr_binaryop_notidentical'] = "!=="
    opNodes['MiddleOpNodes']['expr_binaryop_minus'] = "-"
    opNodes['MiddleOpNodes']['expr_binaryop_mu'] = "*"
    opNodes['MiddleOpNodes']['expr_binaryop_div'] = "/"
    opNodes['MiddleOpNodes']['expr_binaryop_greaterorequal'] = ">="
    opNodes['MiddleOpNodes']['expr_binaryop_smaller'] = "<"
    opNodes['MiddleOpNodes']['expr_binaryop_smallerorequal'] = "<="
    opNodes['MiddleOpNodes']['expr_binaryop_bitwiseand'] = "&"
    opNodes['MiddleOpNodes']['expr_bitwisenot'] = "~"
    opNodes['MiddleOpNodes']['expr_binaryop_logicalor'] = "||"
    opNodes['MiddleOpNodes']['expr_binaryop_booleanor'] = "or"
    opNodes['MiddleOpNodes']['expr_binaryop_booleanand'] = "and"
    opNodes['MiddleOpNodes']['expr_binaryop_equal'] = "=="
    opNodes['MiddleOpNodes']['expr_binaryop_notequal'] = "!="
    opNodes['MiddleOpNodes']['expr_binaryop_bitwisexor'] = "^"
    opNodes['MiddleOpNodes']['expr_binaryop_bitwiseor'] = "|"


    opNodes['FrontOpNodes']['expr_list'] = "list"
    opNodes['FrontOpNodes']['expr_booleannot'] = "!"
    opNodes['FrontOpNodes']['stmt_unset'] = "unset"
    opNodes['FrontOpNodes']['expr_errorsuppress'] = "@"
    opNodes['FrontOpNodes']['expr_unaryminus'] = "-"
    opNodes['FrontOpNodes']['expr_cast_int'] = "(int)"
    opNodes['FrontOpNodes']['expr_cast_string'] = "(string)"
    opNodes['FrontOpNodes']['expr_cast_double'] = "(double)"
    opNodes['FrontOpNodes']['expr_cast_bool'] = "(bool)"
    opNodes['FrontOpNodes']['expr_empty'] = "empty"
    opNodes['FrontOpNodes']['expr_preinc'] = "++"
    opNodes['FrontOpNodes']['expr_predec'] = "--"

    opNodes['RearOpNodes']['expr_postinc'] = "++"
    opNodes['RearOpNodes']['expr_postdec'] = "--"

    opNodes['InnerOpNodes']['expr_assignop_mul'] = "*="
    opNodes['InnerOpNodes']['expr_assignop_minus'] = "-="
    opNodes['InnerOpNodes']['expr_assignop_plus'] = "+="
    opNodes['InnerOpNodes']['expr_assignop_div'] = "/="
    opNodes['InnerOpNodes']['expr_assignref'] = "=&"

    leafNodes = {}



    php_lang_description = {'TreePoints':{},'OpNodes':opNodes,'leafNodes':{},'specialNodes':{}}
    for i in nodes:
        php_lang_description['TreePoints'][i.name] = i.TreePoint

    ## replace all occurrences of AST node names with TreePoint names in walker.py
    #with open('walker.py', 'r') as f:
    #    code = f.read()
    #for i in nodes:
    #    code = code.replace(i.name, i.TreePoint)
    #with open('walker.py', 'w') as f:
    #    f.write(code)

    # Write to JSON file
    with open('php_lang_description.json', 'w') as f:
        json.dump(php_lang_description, f, indent=4)

def main():
    php_lang_description_generator()

if __name__ == "__main__":
    main()
