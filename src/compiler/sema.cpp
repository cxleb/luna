#include "sema.h"
#include "compiler/ast.h"
#include "shared/environment.h"
#include "shared/stack.h"
#include "shared/type.h"
#include "shared/utils.h"
#include <optional>
#include <unordered_map>

#define STD_OPT_CHECK(e) if(auto&& err = e; err.has_value()) { return err.value(); }

namespace luna::compiler {

Error sema_error(ref<Node> node, const char* message, ...) {
    //std::string name = node->name();
    fprintf(stderr, "%llu:%llu: ", node->loc.line + 1, node->loc.col + 1);
    va_list args;
    va_start(args, message);
    auto err = verror(message, args);
    va_end(args);
    return err;  
}

class Inference {
    friend class Sema;

    Environment* env;
    ref<Module> module;
    ref<Func> func;
    luna::Stack<std::unordered_map<std::string, ref<Type>>> locals; 

    std::optional<Error> visit(ref<Expr> expr) {
        switch(expr->kind) {
#define VISITOR_SWITCH(name) \
        case Expr::Kind##name: \
            return this->accept( \
                static_ref_cast<name>(expr));
        EXPR_NODES(VISITOR_SWITCH)
#undef VISITOR_SWITCH
        }
        return std::nullopt;
    }

    std::optional<Error> visit(ref<Stmt> stmt) {
        switch(stmt->kind) {
#define VISITOR_SWITCH(name) \
        case Stmt::Kind##name: \
            return this->accept(static_ref_cast<name>(stmt)); \
            break;
        STMT_NODES(VISITOR_SWITCH)
#undef VISITOR_SWITCH
        }
        return std::nullopt;
    }

    void push_scope() {
        locals.push(std::unordered_map<std::string, ref<Type>>());
    }

    void pop_scope() {
        locals.pop();
    }

    void create_variable(const std::string& name, ref<Type> type) {
        locals.peek().insert({
            name,
            type
        });
    }

    std::optional<ref<Type>> get_variable(const std::string& name) {
        for(auto it = locals.rbegin(); it != locals.rend(); it+=1) {
            if ((*it).contains(name)) {
                return (*it)[name];
            }
        }
        return std::nullopt;
    }

    // Statements
    std::optional<Error> accept(ref<Stmt> stmt) {
        printf("Oh fuck, we should not be here! (Stmt)\n");
        return std::nullopt;
    }

    std::optional<Error> accept(ref<If> stmt) {
        // todo do some checking here?
        STD_OPT_CHECK(visit(stmt->condition));
        STD_OPT_CHECK(visit(stmt->then_stmt));
        if (stmt->else_stmt != nullptr) {
            STD_OPT_CHECK(visit(stmt->else_stmt));
        }
        return std::nullopt;
    }
    
    std::optional<Error> accept(ref<Return> ret) {
        if (func->return_type.has_value()) {
            if (!ret->value.has_value())
                return sema_error(ret, "Expecting return value");
            auto return_value = *ret->value;
            STD_OPT_CHECK(visit(*ret->value));
            if (!return_value->type->compare(*func->return_type)) {
                return sema_error(ret, "Return type is incompatiable");
            }
        } else {
            if(ret->value.has_value()) {
                return sema_error(ret, "%s should not return value", func->name.c_str());
            }
        }
        return std::nullopt;
    }
    
    std::optional<Error> accept(ref<VarDecl> decl) {
        luna_assert(locals.count() != 0);
        STD_OPT_CHECK(visit(decl->value));
        
        auto type = decl->value->type;
        if (decl->type.has_value())
            if (!type->compare(*decl->type))
                return sema_error(decl, "Type is not compatible to assignment");
        
        if (get_variable(decl->name).has_value()) {
            return sema_error(decl, "%s already defined", decl->name.c_str());
        }
        create_variable(decl->name, type);

        return std::nullopt;
    }
    
    std::optional<Error> accept(ref<While> stmt) {
        // todo do some checking here?
        STD_OPT_CHECK(visit(stmt->condition));
        STD_OPT_CHECK(visit(stmt->loop));
        return std::nullopt;
    }
    
    std::optional<Error> accept(ref<For> stmt) {
        return std::nullopt;
    }
    
    std::optional<Error> accept(ref<Block> block) {
        push_scope();
        for(auto stmt: block->stmts) {
            STD_OPT_CHECK(visit(stmt));
        }
        pop_scope();
        return std::nullopt;
    }

    std::optional<Error> accept(ref<ExprStmt> expr_stmt) {
        STD_OPT_CHECK(visit(expr_stmt->expr));
        return std::nullopt;
    }

    // Expressions
    std::optional<Error> accept(ref<Expr> expr) {
        printf("Oh fuck, we should not be here! (Expr)\n");
        return std::nullopt;
    }

    std::optional<Error> accept(ref<BinaryExpr> expr) {
        STD_OPT_CHECK(visit(expr->lhs));
        STD_OPT_CHECK(visit(expr->rhs));
        switch(expr->bin_kind) {
            case BinaryExpr::KindAdd:
            case BinaryExpr::KindDivide:
            case BinaryExpr::KindMultiply:
            case BinaryExpr::KindSubtract:
                if (!expr->lhs->type->is_numeric()) {
                    return sema_error(expr, "Trying to do a binary operation on a non-numeric number");
                }
                if (!expr->rhs->type->is_numeric()) {
                    return sema_error(expr, "Trying to do a binary operation on a non-numeric number");
                }
                if (!expr->lhs->type->compare(expr->rhs->type)) {
                    return sema_error(expr, "Trying to do a binary expression on indifferent types");
                }
                if (expr->lhs->type->compare(number_type()) || expr->rhs->type->compare(number_type())) {
                    expr->type = number_type();
                } else {
                    expr->type = int_type();
                }
                break;
            case BinaryExpr::KindEqual:
            case BinaryExpr::KindNotEqual:
            case BinaryExpr::KindLessThan:
            case BinaryExpr::KindGreaterThan:
            case BinaryExpr::KindLessThanEqual:
            case BinaryExpr::KindGreaterThanEqual:
                if (!expr->lhs->type->compare(expr->rhs->type)) {
                    return sema_error(expr, "Trying to do a comparison on indifferent types");
                }
                expr->type = bool_type();
                break;
        }
        return std::nullopt;
    }
    
    std::optional<Error> accept(ref<Unary> expr) {
        return std::nullopt;
    }

    std::optional<Error> accept(ref<Assign> assign) {
        STD_OPT_CHECK(visit(assign->local));
        STD_OPT_CHECK(visit(assign->value));
        if (!assign->local->type->compare(assign->value->type)) {
            return sema_error(assign, "Incompatible types in assignment");
        }
        assign->type = assign->local->type;
        return std::nullopt;
    }
    
    std::optional<Error> accept(ref<Call> call) {
        for (auto f: module->funcs) {
            if(f->name == call->name) {
                if (f->params.size() != call->args.size()) {
                    return sema_error(call, "Not enough arguements for function call");
                }
                uint32_t i = 0;
                for (auto arg: call->args) {
                    STD_OPT_CHECK(visit(arg));
                    if(!arg->type->compare(f->params[i].type)) {
                        return sema_error(call, "Invalid type for param %d", i);
                    }
                    i+=1;
                }
                if (f->return_type.has_value()) {
                    call->type = *f->return_type;
                } else {
                    call->type = nullptr;
                }                
                return std::nullopt;
            }
        }
        auto host_func_id = env->get_func_id(call->name);
        if (host_func_id.has_value()) {
            for (auto arg: call->args) {
                STD_OPT_CHECK(visit(arg));
            }
            // no return type
            call->type = nullptr;
            return std::nullopt;
        }
        return sema_error(call, "Attempting to call unknown function");
    }
    
    std::optional<Error> accept(ref<Integer> expr) {
        // type is assigned in constructor
        return std::nullopt;
    }
    
    std::optional<Error> accept(ref<Float> expr) {
        // type is assigned in constructor
        return std::nullopt;
    }
    
    std::optional<Error> accept(ref<String> str) {
        // type is assigned in constructor
        return std::nullopt;
    }
    
    std::optional<Error> accept(ref<Identifier> ident) {
        auto type = get_variable(ident->name);
        if (!type.has_value()) {
            return sema_error(ident, "%s not defined", ident->name.c_str());
        }
        ident->type = *type;
        return std::nullopt;
    }

    std::optional<Error> accept(ref<Lookup> lookup) {
        STD_OPT_CHECK(visit(lookup->expr));
        if (!lookup->expr->type->is_array()) {
            return sema_error(lookup, "Attempting to index non-array");
        }
        STD_OPT_CHECK(visit(lookup->index));
        if (!lookup->index->type->is_numeric()) {
            return sema_error(lookup, "Attempting to index array with non-integer index");
        }
        auto array_type = static_ref_cast<ArrayType>(lookup->expr->type);
        lookup->type = array_type->element_type;
        return std::nullopt;
    }

    std::optional<Error> accept(ref<ObjectLiteral> literal) {
        return std::nullopt;
    }

    std::optional<Error> accept(ref<ArrayLiteral> literal) {
        ref<Type> prev_type = unknown_type();
        for(auto expr: literal->elements) {
            STD_OPT_CHECK(visit(expr));
            if (prev_type->is_unknown()) {
                prev_type = expr->type;
            }
            if (!prev_type->compare(expr->type)) {
                return sema_error(literal, "Incompatible types in array literal");
            }
        }
        if (prev_type->is_unknown()) {
            return sema_error(literal, "Cannot determine array literal type, specify a value so it"
                " can be determined. TODO: Fix this by using the var type or further statements");
        }
        literal->type = array_type(prev_type);
        return std::nullopt;
    }
};

std::optional<Error> Sema::check(ref<Module> module, Environment* env) {
    for(auto func: module->funcs) {
        Inference inference;
        inference.env = env;    
        inference.func = func;
        inference.module = module;
        inference.push_scope();
        for(auto& param : func->params) {
            inference.create_variable(param.name, param.type);
        }
        STD_OPT_CHECK(inference.visit(func->root));
        inference.pop_scope();
    }
    return std::nullopt;
}

}