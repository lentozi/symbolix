use tree_drawer::egui_viewer::TreeViewer;
use tree_drawer::layout::{build_layout_tree, TidyLayout};
use symbolix::{context, var};
use symbolix::lexer::symbol::Precedence;
use symbolix::lexer::Lexer;
use symbolix::parser::expression::Expression;
use symbolix::parser::pratt_parsing;
use symbolix::semantic::ast_to_semantic;
use symbolix::semantic::variable::VariableType;

fn main() {
    context! {
        let _a = var!("a", VariableType::Integer, None);
        let _b = var!("b", VariableType::Integer, None);
        let _c = var!("c", VariableType::Integer, None);
        let _d = var!("d", VariableType::Integer, None);
        let _e = var!("e", VariableType::Integer, None);
        let _x = var!("x", VariableType::Integer, None);

        let input = "-x + 123 + 45.67 * (89 - 0.1) ^ x";
        // let input = "(x > 100 ? x * (2 + 3) : x) / 2";
        // let input = "1 * (2 + 3) * 4";
        // let input = "a + b * c - d / e";
        // let input = "x > 0 ? x : -x";
        let mut lexer: Lexer = Lexer::new(input);
        let expression: Expression = pratt_parsing(&mut lexer, Precedence::Lowest);
        let _expr_tree = expression.to_owned_tree();
        let mut semantic_expression = ast_to_semantic(&expression);
        semantic_expression.normalize();
        let _semantic_tree = semantic_expression.to_owned_tree();

        let mut layout = build_layout_tree(&_expr_tree);
        TidyLayout::default().apply(&mut layout);
        TreeViewer::new(layout)
            .with_title("Expression Tree")
            .run().expect("Failed to run TreeViewer");
    }
}
