# capnpc-ocaml-decoder

"Decodes" [Cap'n Proto](https://capnproto.org) messages into native [OCaml](https://ocaml.org) variant and record types. Uses [capnp-ocaml](https://github.com/capnproto/capnp-ocaml) for parsing the messages.

## How to install

This program is itself written in Rust. To install it, first [install Rust](https://rustup.rs) and then run
```
cargo install --locked --git https://github.com/btj/capnpc-ocaml-decoder
```

## Example

Assume the file `example.capnp` contains the following Cap'n Proto schema:
```capnp
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
```
Running
```
capnp compile -o ocaml example.capnp
```
produces files `example.mli` and `example.ml`. Then running
```
capnp compile -o ocaml-decoder example.capnp > example_decoder.ml
```
produces file `example_decoder.ml` with the following contents:
```ocaml
type 't option =
  | Nothing
  | Something of 't
and bin_op =
  | Add
  | Sub
  | Mul
  | Div
and expr_bin_expr =
  {
    lhs: expr;
    op: bin_op;
    rhs: expr
  }
and expr =
  | Literal of int64
  | VarName of string
  | FunCall of {target_expr: expr option; fun_name: string; arg_exprs: expr list}
  | BinExpr of expr_bin_expr

module S = Example.Make (Capnp.BytesMessage)
module R = S.Reader

let rec decode_option: 'rt 't. ('rt S.reader_t -> 't) -> R.Option.t -> 't option = fun decode_t r ->
  match R.Option.get r with
  | Nothing -> Nothing
  | Something r' -> Something (decode_t (R.of_pointer r'))
  | Undefined _ -> failwith "Undefined discriminant"
and decode_bin_op (r: R.BinOp.t): bin_op = match r with
  | Add -> Add
  | Sub -> Sub
  | Mul -> Mul
  | Div -> Div
  | Undefined _ -> failwith "Undefined enumerant"
and decode_expr_bin_expr r: expr_bin_expr =
  {
    lhs = decode_expr (R.Expr.BinExpr.lhs_get r);
    op = decode_bin_op (R.Expr.BinExpr.op_get r);
    rhs = decode_expr (R.Expr.BinExpr.rhs_get r)
  }
and decode_expr r: expr =
  match R.Expr.get r with
  | Literal r' -> Literal (r')
  | VarName r' -> VarName (r')
  | FunCall r' -> FunCall {target_expr = decode_option decode_expr (R.Expr.FunCall.target_expr_get r'); fun_name = (R.Expr.FunCall.fun_name_get r'); arg_exprs = Capnp.Array.map_list (R.Expr.FunCall.arg_exprs_get r') ~f:decode_expr}
  | BinExpr r' -> BinExpr (decode_expr_bin_expr r')
  | Undefined _ -> failwith "Undefined discriminant"
```
Use the following incantation in your `dune` file:
```dune
(library
 (name example_decoder)
 (flags (:standard -w -55)) ; -55: inlining impossible
 (libraries stdint capnp))

(rule
 (targets example.mli example.ml)
 (deps
  (:schema example.capnp))
 (action
  (run capnp compile -I %{env:CAPNP_INC_DIR=} -o ocaml %{schema})))

(rule
 (targets example_decoder.ml)
 (deps
  (:schema example.capnp))
 (action
  (with-stdout-to example_decoder.ml
   (run capnp compile -I %{env:CAPNP_INC_DIR=} -o ocaml-decoder %{schema}))))
```