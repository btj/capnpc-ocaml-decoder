@0xdcc5beb4d6f58a04;

struct Option(T) {
    union {
        nothing @0: Void;
        something @1: T;
    }
}

enum BinOp {
    add @0;
    sub @1;
    mul @2;
    div @3;
}

struct Expr {
    struct BinExpr {
        lhs @0: Expr;
        op @1: BinOp;
        rhs @2: Expr;
    }

    union {
        literal @0: Int64;
        varName @1: Text;
        funCall :group {
            targetExpr @2: Option(Expr);
            funName @3: Text;
            argExprs @4: List(Expr);
        }
        binExpr @5: BinExpr;
    }
}
