#include "gen.h"
#include "compiler/ast.h"
#include "runtime/value.h"
#include "shared/builder.h"
#include "shared/environment.h"
#include <optional>
#include <sys/types.h>

namespace luna::compiler {

class GenVisitor {
    friend class Gen;
    bool is_assign;

    uint8_t visit(ref<Expr> expr, std::optional<uint8_t> into) {
        switch(expr->kind) {
#define VISITOR_SWITCH(name) \
        case Expr::Kind##name: \
            return this->accept( \
                static_ref_cast<name>(expr), into);
        EXPR_NODES(VISITOR_SWITCH)
#undef VISITOR_SWITCH
        }
    }

    void visit(ref<Stmt> stmt) {
        switch(stmt->kind) {
#define VISITOR_SWITCH(name) \
        case Stmt::Kind##name: \
            this->accept(static_ref_cast<name>(stmt)); \
            break;
        STMT_NODES(VISITOR_SWITCH)
#undef VISITOR_SWITCH
        }
    }

    uint8_t maybe_alloc_temp(std::optional<uint8_t> into) {
        if (into.has_value()) {
            return *into;
        }
        return builder->alloc_temp();
    }

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
        
        auto temp = builder->alloc_temp();
        auto cond = visit(stmt->condition, temp);
        builder->condbr(cond, body_label);
        builder->free_temp(temp);

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
            auto local = visit(*ret->value, std::nullopt);
            builder->ret(local);
            builder->free_temp(local);
        } else {
            builder->ret();
        }
    }
    
    void accept(ref<VarDecl> decl) {
        //printf("Visiting var decl\n");
        auto local = builder->create_local(decl->name);
        visit(decl->value, local);
    }
    
    void accept(ref<While> stmt) {
        //printf("Visiting while\n");

        //printf("Visiting if\n");
        //if (stmt->)
        auto start_label = builder->new_label();
        auto body_label = builder->new_label();
        auto end_label = builder->new_label();
        
        builder->mark_label(start_label);
        auto temp = builder->alloc_temp();
        auto cond = visit(stmt->condition, temp);
        builder->condbr(cond, body_label);
        builder->free_temp(temp);
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
        auto temp = builder->alloc_temp();
        visit(expr_stmt->expr, temp);
        builder->free_temp(temp);
    }

    // Expressionss
    uint8_t accept(ref<Expr> expr, std::optional<uint8_t> into) {
        printf("Oh fuck, we should not be here! (Expr)\n");
        return 0;
    }

    uint8_t accept(ref<BinaryExpr> expr, std::optional<uint8_t> maybe_into) {
        auto into = maybe_alloc_temp(maybe_into);

        auto lhs = visit(expr->lhs, std::nullopt);
        auto rhs = visit(expr->rhs, std::nullopt);
        switch(expr->bin_kind) {
            case BinaryExpr::KindAdd:
                builder->add(lhs, rhs, into);
                break;
            case BinaryExpr::KindSubtract:
                builder->sub(lhs, rhs, into);
                break;
            case BinaryExpr::KindMultiply:
                builder->mul(lhs, rhs, into);
                break;
            case BinaryExpr::KindDivide:
                builder->div(lhs, rhs, into);
                break;
            case BinaryExpr::KindEqual:
                builder->eq(lhs, rhs, into);
                break;
            case BinaryExpr::KindNotEqual:
                builder->noteq(lhs, rhs, into);
                break;
            case BinaryExpr::KindLessThan:
                builder->less(lhs, rhs, into);
                break;
            case BinaryExpr::KindGreaterThan:
                builder->gr(lhs, rhs, into);
                break;
            case BinaryExpr::KindLessThanEqual:
                builder->less_eq(lhs, rhs, into);
                break;
            case BinaryExpr::KindGreaterThanEqual:
                builder->gr_eq(lhs, rhs, into);
                break;
        }
        builder->free_temp(lhs);
        builder->free_temp(rhs);

        return into;
    }
    
    uint8_t accept(ref<Unary> expr, std::optional<uint8_t> into) {
        printf("Visiting unary\n");
        return 0;
    }

    uint8_t accept(ref<Assign> assign, std::optional<uint8_t> maybe_into) {
        //printf("Visiting assign\n");
        //auto temp = builder->alloc_temp();
        auto into = maybe_alloc_temp(maybe_into);
        visit(assign->value, into);
        auto temp_is_assign = is_assign;
        is_assign = true;
        auto local = visit(assign->local, into);
        is_assign = temp_is_assign;
        builder->free_temp(local);
        return local;
    }
    
    uint8_t accept(ref<Call> call, std::optional<uint8_t> maybe_into) {
        //printf("Visiting call\n");
        uint8_t temp = builder->alloc_temp();
        uint8_t n = 0;
        for(auto arg: call->args) {
            // this might use the temp
            // which is why we need to make the copy using the returned index
            auto a = visit(arg, temp);
            builder->arg(n++, a);
        }
        builder->free_temp(temp);
        auto into = maybe_alloc_temp(maybe_into);
        builder->call(call->name, call->args.size(), into);
        return into;
    }
    
    uint8_t accept(ref<Integer> expr, std::optional<uint8_t> maybe_into) {
        //printf("Visiting int\n");
        // todo(caleb): optimize loading directly into variables
        auto into = maybe_alloc_temp(maybe_into);
        builder->load_const(into, expr->value);
        return into;
    }
    
    uint8_t accept(ref<Float> expr, std::optional<uint8_t> maybe_into) {
        //printf("Visiting float\n");
        // todo(caleb): optimize loading directly into variables
        auto into = maybe_alloc_temp(maybe_into);
        builder->load_const(into, expr->value);
        return into;
    }
    
    uint8_t accept(ref<String> str, std::optional<uint8_t> maybe_into) {
        auto into = maybe_alloc_temp(maybe_into);
        //printf("Visiting string\n");
        auto cell = env->heap.alloc_string(str->value);
        // todo(caleb): optimize loading directly into variables
        auto temp = builder->alloc_temp();
        builder->load_const(into, cell);
        return into;
    }
    
    uint8_t accept(ref<Identifier> ident, std::optional<uint8_t> into) {
        //printf("Visiting ident\n");
        if (is_assign) {
            builder->store(*into, ident->name);
            return *into;
        } else {
            auto id = builder->get_local_id(ident->name);
            return *id;
        }
    }

    uint8_t accept(ref<Lookup> lookup, std::optional<uint8_t> maybe_into) {
        bool temp_is_assign = is_assign;
        is_assign = false;
        auto expr = visit(lookup->expr, std::nullopt);
        is_assign = temp_is_assign;
        auto idx = visit(lookup->index, std::nullopt);
        uint8_t into;
        if(is_assign) {
            builder->object_set(expr, idx, *maybe_into);
            into =  *maybe_into;
        } else {
            into = maybe_alloc_temp(maybe_into);
            builder->object_get(expr, idx, into);
        }
        builder->free_temp(expr);
        builder->free_temp(idx);
        return into;
    }

    uint8_t accept(ref<ObjectLiteral> literal, std::optional<uint8_t> maybe_into) {
        auto into = maybe_alloc_temp(maybe_into);
        builder->object_new(into);
        return into;
    }

    uint8_t accept(ref<ArrayLiteral> literal, std::optional<uint8_t> maybe_into) {
        //builder->object_new();
        //uint64_t i = 0;
        //for(auto& expr : literal->elements) {
        //    builder->int_(i);
        //    builder->
        //    i++;
        //}
        auto into = maybe_alloc_temp(maybe_into);
        builder->object_new(into);
        return into;
    }

    FunctionBuilder* builder;
    Environment* env;
}; 

ref<runtime::Module> Gen::generate(ref<Module> module, Environment* env) {
    ModuleBuilder module_builder(env);

    for(auto func: module->funcs) {
        FunctionBuilder builder = module_builder.new_function(func->name);
        builder.push_scope();
        for(auto& param : func->params) {
            builder.create_local(param.name);
        }
        GenVisitor visitor(&builder, env);
        visitor.visit(func->root);
        builder.pop_scope();
        module_builder.add_function(builder.build());
    }

    return module_builder.get_module();
}

}