#include "gen.h"
#include "compiler/ast.h"
#include "shared/builder.h"
#include "shared/environment.h"

namespace luna::compiler {

class GenVisitor : public Visitor<GenVisitor> {
    bool is_assign;
public:
    GenVisitor(FunctionBuilder* b, Environment* e): builder(b), env(e), is_assign(false) {}

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
    
    void accept(ref<Block> block) {
        //printf("Visiting block\n");
        builder->push_scope();
        for(auto stmt: block->stmts) {
            visit(stmt);
        }
        builder->pop_scope();
    }

    void accept(ref<ExprStmt> expr_stmt) {
        visit(expr_stmt->expr);
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

    void accept(ref<Assign> assign) {
        //printf("Visiting assign\n");
        visit(assign->value);
        auto temp_is_assign = is_assign;
        is_assign = true;
        visit(assign->local);
        is_assign = temp_is_assign;
    }
    
    void accept(ref<Call> call) {
        //printf("Visiting call\n");
        for(auto arg: call->args) {
            visit(arg);
        }
        builder->call(call->name, call->args.size());
    }
    
    void accept(ref<Integer> expr) {
        //printf("Visiting int\n");
        builder->int_(expr->value);
    }
    
    void accept(ref<Float> expr) {
        //printf("Visiting float\n");
        builder->float_(expr->value);
    }
    
    void accept(ref<String> str) {
        //printf("Visiting string\n");
        auto cell = env->heap.alloc_string(str->value);
        builder->cell(cell);
    }
    
    void accept(ref<Identifier> ident) {
        //printf("Visiting ident\n");
        if (is_assign) {
            builder->store(ident->name);
        } else {
            builder->load(ident->name);
        }
    }

    void accept(ref<Lookup> lookup) {
        visit(lookup->expr);
        visit(lookup->index);
        if(is_assign) {
            builder->object_set();
        } else {
            builder->object_get();
        }
    }

    void accept(ref<ObjectLiteral> literal) {
        builder->object_new();
    }

    void accept(ref<ArrayLiteral> literal) {
        builder->object_new();
        uint64_t i = 0;
        for(auto& expr : literal->elements) {
            builder->int_(i);
            builder->
            i++;
        }
    }

    FunctionBuilder* builder;
    Environment* env;
}; 

ref<runtime::Module> Gen::generate(ref<Module> module, Environment* env) {
    ModuleBuilder module_builder(env);

    for(auto func: module->funcs) {
        FunctionBuilder builder = module_builder.new_function(func->name);
        GenVisitor visitor(&builder, env);
        visitor.visit(func->root);
        module_builder.add_function(builder.build());
    }

    return module_builder.get_module();
}

}