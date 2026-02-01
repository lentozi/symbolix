use symbolix::lexer::symbol::Precedence;
use symbolix::lexer::Lexer;
use symbolix::optimizer::{normalize, optimize_d1};
use symbolix::parser::expression::Expression;
use symbolix::parser::pratt_parsing;
use symbolix::semantic::ast_to_semantic;
use symbolix::semantic::variable::VariableType;
use symbolix::{context, var};
use tree_drawer::egui_viewer::TreeViewer;
use tree_drawer::layout::{build_layout_tree, TidyLayout};

fn main() {
    context! {
        let _a = var!("a", VariableType::Integer, None);
        let _b = var!("b", VariableType::Integer, None);
        let _c = var!("c", VariableType::Integer, None);
        let _d = var!("d", VariableType::Integer, None);
        let _e = var!("e", VariableType::Integer, None);
        let _x = var!("x", VariableType::Integer, None);

        let input = "-x + x + 123 + 45.67 * ((89 - 0.1) ^ x) ^ x + 0";
        // let input = "(x > 100 ? x * (2 + 3) : x) / 2";
        // let input = "1 * (2 + 3) * 4";
        // let input = "a + b * c - d / e";
        // let input = "x > 0 ? x : -x";
        let mut lexer: Lexer = Lexer::new(input);
        let expression: Expression = pratt_parsing(&mut lexer, Precedence::Lowest);
        let _expr_tree = expression.to_owned_tree();
        let mut semantic_expression = ast_to_semantic(&expression);
        // semantic_expression.normalize();
        normalize(&mut semantic_expression);
        optimize_d1(&mut semantic_expression);
        // semantic_expression.normalize();
        normalize(&mut semantic_expression);
        println!("{}", semantic_expression);
        let _semantic_tree = semantic_expression.to_owned_tree();

        let mut layout1 = build_layout_tree(&_expr_tree);
        TidyLayout::default().apply(&mut layout1);
        TreeViewer::new(layout1)
            .with_title("Expression Tree")
            .run().expect("Failed to run TreeViewer");
        let mut layout2 = build_layout_tree(&_semantic_tree);
        TidyLayout::default().apply(&mut layout2);
        TreeViewer::new(layout2)
            .with_title("Semantic Tree")
            .run().expect("Failed to run TreeViewer");
    }
}
