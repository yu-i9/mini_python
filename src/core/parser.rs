use std::iter::Peekable;
use syntax::*;
use token::Token;

/*
program -> statement* EOF

block -> Indent statement+ Dedent | e

statement -> simple_stmt NewLine
           | compound_stmt

simple_stmt ->
  | expr
  | target = expr
  | Return expr?
  | Continue
  | Break
  | Raise expr
  | Assert expr

target ->
  | Ident
  | cexpr Dot Ident
  | cexpr LBrace expr RBrace

compound_stmt ->
  | If expr Colon NewLine block Else Colon NewLine block
  | While expr Colon NewLine block
  | For target In Expr Colon NewLine block
  | Try Colon NewLine block Except Colon NewLine block
  | Def Ident(s) LParen parm_list RParen Colon NewLine block
  | Class Ident(s) LParen (expr, ..., expr) RParen Colon NewLine block

parm_list ->
  | Ident(s)
  | Ident(s) Comma parm_list
  | e

expr -> eexpr Lt eexpr
      | eexpr

eexpr -> pexpr EqEq pexpr
       | pexpr

pexpr -> cexpr Plus pexpr
       | cexpr

cexpr -> aexpr successor*
       | aexpr

successor -> LParen comma_list RParen
           | Dot Ident
           | LBrace expr RBrace

aexpr -> LParen expr RParen
       | LBracket (expr (Comma expr)*)? RBracket
       | LBrace (expr Colon expr)* RBrace
       | True
       | False
       | Ident
       | Int
       | Str
       | None

comma_list ->
  | expr
  | expr Comma comma_list
  | e
 */

pub trait TokenStream {
    fn parse(&mut self) -> Program;
    fn program(&mut self) -> Program;
    fn block(&mut self) -> Program;
    fn statement(&mut self) -> Stmt;
    fn simple_stmt(&mut self) -> SimpleStmt;
    fn is_compound(&mut self) -> bool;
    fn compound_stmt(&mut self) -> CompoundStmt;
    fn parm_list(&mut self) -> Vec<Id>;
    fn expr(&mut self) -> Expr;
    fn eexpr(&mut self) -> Expr;
    fn pexpr(&mut self) -> Expr;
    fn cexpr(&mut self) -> Expr;
    fn comma_list(&mut self) -> Vec<Expr>;
    fn pair_list(&mut self) -> Vec<(Expr, Expr)>;
    fn is_expr(&mut self) -> bool;
    fn aexpr(&mut self) -> Expr;
    fn match_token(&mut self, token: Token) -> bool;
    fn consume(&mut self, token: Token) -> ();
    fn consume_ident(&mut self) -> String;
    fn consume_int(&mut self) -> i32;
    fn consume_str(&mut self) -> String;
}

impl<I: Iterator<Item = Token>> TokenStream for Peekable<I> {
    fn parse(&mut self) -> Program {
        self.program()
    }

    fn program(&mut self) -> Program {
        let mut prog: Program = vec![];
        loop {
            match self.peek() {
                Some(&Token::EOF) => break,
                Some(_) => prog.push(self.statement()),
                None => panic!("Parse Error: program"),
            }
        };
        prog
    }

    fn block(&mut self) -> Program {
        let mut prog: Program = vec![];
        match self.peek() {
            Some(&Token::Indent) => {
                self.consume(Token::Indent);
                prog.push(self.statement());
                loop {
                    match self.peek() {
                        Some(&Token::Dedent) => {
                            self.consume(Token::Dedent);
                            break
                        },
                        Some(_) => prog.push(self.statement()),
                        _ => panic!("Parse Error: block")
                    }
                }
            },
            _ => (),
        };
        prog
    }

    fn statement(&mut self) -> Stmt {
        if self.is_compound() {
            Stmt::StmtCompound(self.compound_stmt())
        } else {
            let stmt = Stmt::StmtSimple(self.simple_stmt());
            self.consume(Token::NewLine);
            stmt
        }
    }

    fn simple_stmt(&mut self) -> SimpleStmt {
        match self.peek() {
            Some(&Token::Break) => {
                self.consume(Token::Break);
                SimpleStmt::BreakStmt
            },
            Some(&Token::Continue) => {
                self.consume(Token::Continue);
                SimpleStmt::ContinueStmt
            },
            Some(&Token::Raise) => {
                self.consume(Token::Raise);
                let expr = self.expr();
                SimpleStmt::RaiseStmt(expr)
            },
            Some(&Token::Return) => {
                self.consume(Token::Return);
                let mut expr = Expr::NoneExpr;
                if !self.match_token(Token::NewLine) {
                    expr = self.expr();
                }
                SimpleStmt::ReturnStmt(expr)
            },
            Some(&Token::Assert) => {
                self.consume(Token::Assert);
                let expr = self.expr();
                SimpleStmt::AssertStmt(expr)
            },
            _ => {
                let expr = self.expr();
                match self.peek() {
                    Some(&Token::Eq) => {
                        let target = match expr {
                            Expr::VarExpr(id) => Target::IdentTarget(id),
                            Expr::AttrExpr(expr, id) => Target::AttrTarget(expr, id),
                            Expr::SubscrExpr(expr1, expr2) => Target::SubscrTarget(expr1, expr2),
                            _ => panic!("Parse Error: Assign Target")
                        };
                        self.consume(Token::Eq);
                        let expr = self.expr();
                        SimpleStmt::AssignStmt(target, expr)
                    },
                    Some(&Token::NewLine) => {
                        SimpleStmt::ExprStmt(expr)
                    },
                    _ => panic!("Parse Error: AssignStmt")
                }
            },
        }
    }

    fn is_compound(&mut self) -> bool {
        match self.peek() {
            Some(&Token::If) => true,
            Some(&Token::While) => true,
            Some(&Token::For) => true,
            Some(&Token::Try) => true,
            Some(&Token::Def) => true,
            Some(&Token::Class) => true,
            _ => false,
        }
    }

    fn compound_stmt(&mut self) -> CompoundStmt {
        match self.peek() {
            Some(&Token::If) => {
                self.consume(Token::If);
                let expr = self.expr();
                self.consume(Token::Colon);
                self.consume(Token::NewLine);
                let prog_then = self.block();
                self.consume(Token::Else);
                self.consume(Token::Colon);
                self.consume(Token::NewLine);
                let prog_else = self.block();
                CompoundStmt::IfStmt(expr, prog_then, prog_else)
            },
            Some(&Token::While) => {
                self.consume(Token::While);
                let expr = self.expr();
                self.consume(Token::Colon);
                self.consume(Token::NewLine);
                let prog = self.block();
                CompoundStmt::WhileStmt(expr, prog)
            },
            Some(&Token::For) => {
                self.consume(Token::For);
                let target = match self.expr() {
                    Expr::VarExpr(id) => Target::IdentTarget(id),
                    Expr::AttrExpr(expr, id) => Target::AttrTarget(expr, id),
                    Expr::SubscrExpr(expr1, expr2) => Target::SubscrTarget(expr1, expr2),
                    _ => panic!("Parse Error: Assign Target")
                };
                self.consume(Token::In);
                let expr = self.expr();
                self.consume(Token::Colon);
                self.consume(Token::NewLine);
                let prog = self.block();
                CompoundStmt::ForStmt(target, expr, prog)
            },
            Some(&Token::Try) => {
                self.consume(Token::Try);
                self.consume(Token::Colon);
                self.consume(Token::NewLine);
                let prog_try = self.block();
                self.consume(Token::Except);
                self.consume(Token::Colon);
                self.consume(Token::NewLine);
                let prog_except = self.block();
                CompoundStmt::TryStmt(prog_try, prog_except)
            },
            Some(&Token::Def) => {
                self.consume(Token::Def);
                let fun_name = self.consume_ident();
                self.consume(Token::LParen);
                let parm_list = self.parm_list();
                self.consume(Token::RParen);
                self.consume(Token::Colon);
                self.consume(Token::NewLine);
                let prog = self.block();
                CompoundStmt::DefStmt(fun_name, parm_list, prog)
            },
            Some(&Token::Class) => {
                let mut bases = vec![];
                self.consume(Token::Class);
                let class_name = self.consume_ident();

                if self.match_token(Token::LParen) {
                    self.consume(Token::LParen);
                    bases = self.comma_list();
                    self.consume(Token::RParen);
                }

                self.consume(Token::Colon);
                self.consume(Token::NewLine);
                let prog = self.block();
                CompoundStmt::ClassStmt(class_name, bases, prog)
            },
            _ => panic!("Parse Error: compound_stmt"),
        }
    }

    fn parm_list(&mut self) -> Vec<Id> {
        let mut pl: Vec<Id>  = vec![];
        match self.peek() {
            Some(&Token::Ident(_)) => pl.push(self.consume_ident()),
            Some(_) => return pl,
            _ => panic!("Parse Error: parm_list"),
        };
        loop {
            match self.peek() {
                Some(&Token::Comma) => {
                    self.consume(Token::Comma);
                    pl.push(self.consume_ident());
                },
                Some(_) => break,
                _ => panic!("Parse Error: parm_list"),
            }
        };
        pl
    }

    fn expr(&mut self) -> Expr {
        let expr1 = self.eexpr();
        match self.peek() {
            Some(&Token::Lt) => {
                self.consume(Token::Lt);
                let expr2 = self.eexpr();
                Expr::LtExpr(Box::new(expr1), Box::new(expr2))
            },
            Some(_) => expr1,
            None => panic!("Parse Error: expr"),
        }
    }

    fn eexpr(&mut self) -> Expr {
        let expr1 = self.pexpr();
        match self.peek() {
            Some(&Token::EqEq) => {
                self.consume(Token::EqEq);
                let expr2 = self.pexpr();
                Expr::EqEqExpr(Box::new(expr1), Box::new(expr2))
            },
            Some(_) => expr1,
            None => panic!("Parse Error: expr"),
        }
    }

    fn pexpr(&mut self) -> Expr {
        let expr1 = self.cexpr();
        match self.peek() {
            Some(&Token::Plus) => {
                self.consume(Token::Plus);
                let expr2 = self.pexpr();
                Expr::AddExpr(Box::new(expr1), Box::new(expr2))
            },
            Some(_) => expr1,
            None => panic!("Parse Error: pexpr"),
        }
    }

    fn cexpr(&mut self) -> Expr {
        let mut expr = self.aexpr();
        loop {
            match self.peek() {
                Some(&Token::LParen) => {
                    self.consume(Token::LParen);
                    let comma_list = self.comma_list();
                    self.consume(Token::RParen);
                    expr = Expr::CallExpr(Box::new(expr), comma_list)
                },
                Some(&Token::Dot) => {
                    self.consume(Token::Dot);
                    let ident = self.consume_ident();
                    expr = Expr::AttrExpr(Box::new(expr), ident)
                },
                Some(&Token::LBracket) => {
                    self.consume(Token::LBracket);
                    let key_expr = self.expr();
                    self.consume(Token::RBracket);
                    expr = Expr::SubscrExpr(Box::new(expr), Box::new(key_expr))
                },
                Some(_) => return expr,
                None => panic!("Parse Error: cexpr"),
            }
        }
    }

    fn aexpr(&mut self) -> Expr {
        match self.peek().unwrap() {
            &Token::LParen => {
                self.consume(Token::LParen);
                let expr = self.expr();
                self.consume(Token::RParen);
                expr
            },
            &Token::LBracket => {
                self.consume(Token::LBracket);
                let cl = self.comma_list();
                self.consume(Token::RBracket);
                Expr::ListExpr(cl)

            },
            &Token::LBrace => {
                self.consume(Token::LBrace);
                let pl = self.pair_list();
                self.consume(Token::RBrace);
                Expr::DictExpr(pl)
            },
            &Token::True => {
                self.consume(Token::True);
                Expr::BoolExpr(true)
            },
            &Token::False => {
                self.consume(Token::False);
                Expr::BoolExpr(false)
            },
            &Token::Ident(_) => {
                let ident = self.consume_ident();
                Expr::VarExpr(ident)
            },
            &Token::Int(_) => {
                let i = self.consume_int();
                Expr::IntExpr(i)
            },
            &Token::Str(_) => {
                let s = self.consume_str();
                Expr::StrExpr(s)
            },
            _ => panic!("Parse Error: aexpr"),
        }
    }

    fn is_expr(&mut self) -> bool {
        match self.peek() {
            Some(&Token::LParen) => true,
            Some(&Token::LBracket) => true,
            Some(&Token::LBrace) => true,
            Some(&Token::True) => true,
            Some(&Token::False) => true,
            Some(&Token::Ident(_)) => true,
            Some(&Token::Int(_)) => true,
            Some(&Token::Str(_)) => true,
            Some(&Token::None) => true,
            _ => false,
        }
    }

    fn comma_list(&mut self) -> Vec<Expr> {
        let mut al: Vec<Expr>  = vec![];

        if self.is_expr() {
            al.push(self.expr());
        } else {
            return al;
        }

        loop {
            match self.peek() {
                Some(&Token::Comma) => {
                    self.consume(Token::Comma);
                    al.push(self.expr());
                },
                Some(_) => break,
                _ => panic!("Parse Error: parm_list"),
            }
        };
        al
    }

    fn pair_list(&mut self) -> Vec<(Expr, Expr)> {
        let mut pl: Vec<(Expr, Expr)>  = vec![];

        if self.is_expr() {
            let e1 = self.expr();
            self.consume(Token::Colon);
            let e2 = self.expr();
            pl.push((e1, e2))
        } else {
            return pl;
        }

        loop {
            match self.peek() {
                Some(&Token::Comma) => {
                    self.consume(Token::Comma);
                    let e1 = self.expr();
                    self.consume(Token::Colon);
                    let e2 = self.expr();
                    pl.push((e1, e2))
                },
                Some(_) => break,
                _ => panic!("Parse Error: pair_list"),
            }
        };
        pl
    }

    fn match_token(&mut self, token: Token) -> bool {
        match self.peek() {
            Some(token_) if token == *token_ => true,
            _ => false,
        }
    }

    fn consume(&mut self, token: Token) -> () {
        if self.match_token(token.clone()) {
            self.next();
        } else {
            panic!("Parse Error: {:?} expected", token);
        }
    }

    fn consume_ident(&mut self) -> String {
        match self.next() {
            Some(Token::Ident(ref s)) => s.clone(),
            _ => panic!("Parse Error: Ident expected"),
        }
    }

    fn consume_int(&mut self) -> i32 {
        match self.next() {
            Some(Token::Int(ref i)) => i.clone(),
            _ => panic!("Parse Error: int expected"),
        }
    }

    fn consume_str(&mut self) -> String {
        match self.next() {
            Some(Token::Str(ref s)) => s.clone(),
            _ => panic!("Parse Error: str expected"),
        }
    }
}
