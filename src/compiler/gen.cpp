#include "gen.h"
#include "compiler/ast.h"
#include "shared/builder.h"
#include "shared/environment.h"

namespace luna::compiler {

class GenVisitor : public Visitor<GenVisitor> {
public:
    GenVisitor(FunctionBuilder* b): builder(b) {}

    // Statements
    void accept(ref<Stmt> stmt) {
        printf("Oh fuck, we should not be here! (Stmt)\n");
    }

    void accept(ref<If> stmt) {
        //printf("Visiting if\n");
        //if (stmt->)
        auto end_label = builder->new_label();
        auto body_label = builder->new_label();
        
        visit(stmt->condition);
        builder->condbr(body_label);
        if (stmt->else_stmt != nullptr) {
            visit(stmt->else_stmt);
        }
        builder->br(end_label);
        builder->mark_label(body_label);
        visit(stmt->then_stmt);
        builder->mark_label(end_label);
    }
    
    void accept(ref<Return> ret) {
        //printf("Visiting return\n");
        if(ret->value.has_value()) {
            visit(*ret->value);
        }
        builder->ret();
    }
    
    void accept(ref<VarDecl> decl) {
        //printf("Visiting var decl\n");
        builder->create_local(decl->name);
        visit(decl->value);
        builder->store(decl->name);
    }
    
    void accept(ref<Assign> assign) {
        //printf("Visiting assign\n");
        visit(assign->value);
        builder->store(assign->name);
    }
    
    void accept(ref<While> stmt) {
        //printf("Visiting while\n");

        //printf("Visiting if\n");
        //if (stmt->)
        auto start_label = builder->new_label();
        auto body_label = builder->new_label();
        auto end_label = builder->new_label();
        
        builder->mark_label(start_label);
        visit(stmt->condition);
        builder->condbr(body_label);
        builder->br(end_label);
        builder->mark_label(body_label);
        visit(stmt->loop);
        builder->br(start_label);
        builder->mark_label(end_label);
    }
    
    void accept(ref<For> stmt) {
        //printf("Visiting for\n");
    }
    
    void accept(ref<Call> call) {
        //printf("Visiting call\n");
        for(auto arg: call->args) {
            visit(arg);
        }
        builder->call(call->name, call->args.size());
    }
    
    void accept(ref<Block> block) {
        //printf("Visiting block\n");
        builder->push_scope();
        for(auto stmt: block->stmts) {
            visit(stmt);
        }
        builder->pop_scope();
    }

    // Expressionss
    void accept(ref<Expr> expr) {
        printf("Oh fuck, we should not be here! (Expr)\n");
    }

    void accept(ref<BinaryExpr> expr) {
        //printf("Visiting BinaryExpr\n");
        visit(expr->lhs);
        visit(expr->rhs);
        switch(expr->bin_kind) {
            case BinaryExpr::KindAdd:
                builder->add();
                break;
            case BinaryExpr::KindSubtract:
                builder->sub();
                break;
            case BinaryExpr::KindMultiply:
                builder->mul();
                break;
            case BinaryExpr::KindDivide:
                builder->div();
                break;
            case BinaryExpr::KindEqual:
                builder->eq();
                break;
            case BinaryExpr::KindNotEqual:
                builder->noteq();
                break;
            case BinaryExpr::KindLessThan:
                builder->less();
                break;
            case BinaryExpr::KindGreaterThan:
                builder->gr();
                break;
            case BinaryExpr::KindLessThanEqual:
                builder->less_eq();
                break;
            case BinaryExpr::KindGreaterThanEqual:
                builder->gr_eq();
                break;
        }
    }
    
    void accept(ref<Unary> expr) {
        //printf("Visiting unary\n");
    }
    
    void accept(ref<CallExpr> call) {
        //printf("Visiting call\n");
        for(auto arg: call->call.args) {
            visit(arg);
        }
        builder->call(call->call.name, call->call.args.size());
    }
    
    void accept(ref<Integer> expr) {
        //printf("Visiting int\n");
        builder->int_(expr->value);
    }
    
    void accept(ref<Float> expr) {
        //printf("Visiting float\n");
        builder->float_(expr->value);
    }
    
    void accept(ref<String> expr) {
        //printf("Visiting string\n");
    }
    
    void accept(ref<Identifier> ident) {
        //printf("Visiting ident\n");
        builder->load(ident->name);
    }

    FunctionBuilder* builder;
}; 

ref<runtime::Module> Gen::generate(ref<Module> module, Environment* env) {
    ModuleBuilder module_builder(env);

    for(auto func: module->funcs) {
        FunctionBuilder builder = module_builder.new_function(func->name);
        GenVisitor visitor(&builder);
        visitor.visit(func->root);
        module_builder.add_function(builder.build());
    }

    return module_builder.get_module();
}

}