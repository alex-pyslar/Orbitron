Add a new keyword to the Orbitron language end-to-end.

Usage: /add-keyword <keyword> <description>

Example: /add-keyword yield "yield a value from a generator"

Steps:
1. Read `src/lexer/token.rs` to understand the Token enum structure.
2. Add the new `Token::` variant to the enum with a comment explaining origin language.
3. Read `src/lexer/mod.rs` and add the keyword to the keyword map (the `match` or `HashMap` that maps string → Token).
4. Read `src/parser/ast.rs` — determine if a new AST node is needed.
5. If yes, add the AST node (Stmt or Expr variant).
6. Read `src/parser/mod.rs` — add parsing logic for the keyword.
7. Read `src/codegen/stmt.rs` or `src/codegen/expr.rs` — add codegen for the new construct.
8. Read `src/fmt/mod.rs` — add formatter support so `orbitron fmt` handles the new syntax.
9. Read `src/jvm/mod.rs` — add JVM backend support if applicable.
10. Create `examples/07_advanced/<keyword>_demo.ot` showing usage.
11. Update `docs/reference.md` — add entry to the keywords table.
12. Run `/test` to verify nothing is broken.
