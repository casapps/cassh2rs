use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct AST {
    pub root: ASTNode,
    pub metadata: ScriptMetadata,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ScriptMetadata {
    pub shebang: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub dependencies: Vec<String>,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ASTNode {
    // Top-level
    Script(Vec<Box<ASTNode>>),
    
    // Commands
    Command {
        name: String,
        args: Vec<Box<ASTNode>>,
        redirections: Vec<Redirection>,
        background: bool,
    },
    Pipeline(Vec<Box<ASTNode>>),
    
    // Control structures
    If {
        condition: Box<ASTNode>,
        then_block: Box<ASTNode>,
        elif_blocks: Vec<(Box<ASTNode>, Box<ASTNode>)>,
        else_block: Option<Box<ASTNode>>,
    },
    While {
        condition: Box<ASTNode>,
        body: Box<ASTNode>,
    },
    Until {
        condition: Box<ASTNode>,
        body: Box<ASTNode>,
    },
    For {
        variable: String,
        items: ForItems,
        body: Box<ASTNode>,
    },
    Case {
        expr: Box<ASTNode>,
        cases: Vec<CaseItem>,
    },
    
    // Functions
    Function {
        name: String,
        body: Box<ASTNode>,
    },
    
    // Variables and expansion
    Assignment {
        name: String,
        value: Box<ASTNode>,
        export: bool,
        readonly: bool,
        local: bool,
    },
    Variable(String),
    ParameterExpansion {
        name: String,
        expansion_type: ExpansionType,
    },
    CommandSubstitution(Box<ASTNode>),
    ArithmeticExpansion(Box<ASTNode>),
    
    // Expressions
    BinaryOp {
        left: Box<ASTNode>,
        op: BinaryOperator,
        right: Box<ASTNode>,
    },
    UnaryOp {
        op: UnaryOperator,
        operand: Box<ASTNode>,
    },
    
    // Literals
    String(String, StringType),
    Number(f64),
    Array(Vec<Box<ASTNode>>),
    
    // Special
    Glob(String),
    Heredoc {
        delimiter: String,
        content: String,
        strip_tabs: bool,
    },
    Return(Option<Box<ASTNode>>),
    Break,
    Continue,
    Exit(Option<Box<ASTNode>>),
    
    // Compound
    Block(Vec<Box<ASTNode>>),
    Subshell(Box<ASTNode>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForItems {
    List(Vec<Box<ASTNode>>),
    Command(Box<ASTNode>),
    CStyle {
        init: Box<ASTNode>,
        condition: Box<ASTNode>,
        update: Box<ASTNode>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaseItem {
    pub patterns: Vec<String>,
    pub body: Box<ASTNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExpansionType {
    Default(Box<ASTNode>),      // ${var:-default}
    Assign(Box<ASTNode>),       // ${var:=default}
    Error(String),              // ${var:?error}
    Alternative(Box<ASTNode>),  // ${var:+alt}
    Substring {
        offset: Box<ASTNode>,
        length: Option<Box<ASTNode>>,
    },                          // ${var:offset:length}
    RemovePrefix(String),       // ${var#pattern}
    RemovePrefixLong(String),   // ${var##pattern}
    RemoveSuffix(String),       // ${var%pattern}
    RemoveSuffixLong(String),   // ${var%%pattern}
    Replace {
        pattern: String,
        replacement: String,
        global: bool,
    },                          // ${var/pattern/replacement}
    Length,                     // ${#var}
    Indirect,                   // ${!var}
    Keys,                       // ${!var[@]}
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringType {
    SingleQuoted,
    DoubleQuoted,
    Unquoted,
    AnsiC,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    
    // Comparison
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    
    // String comparison
    StringEqual,
    StringNotEqual,
    Match,      // =~
    
    // Logical
    And,
    Or,
    
    // File test
    FileNewer,  // -nt
    FileOlder,  // -ot
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Not,
    Negate,
    
    // File tests
    FileExists,         // -e
    FileRegular,        // -f
    FileDirectory,      // -d
    FileSymlink,        // -L
    FileReadable,       // -r
    FileWritable,       // -w
    FileExecutable,     // -x
    FileNotEmpty,       // -s
    
    // String tests
    StringNotEmpty,     // -n
    StringEmpty,        // -z
}

#[derive(Debug, Clone, PartialEq)]
pub struct Redirection {
    pub fd: Option<i32>,
    pub target: RedirectionTarget,
    pub append: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RedirectionTarget {
    File(String),
    Fd(i32),
    Heredoc {
        delimiter: String,
        content: String,
        strip_tabs: bool,
    },
    HereString(String),
}

impl ASTNode {
    pub fn is_empty_block(&self) -> bool {
        match self {
            ASTNode::Block(nodes) => nodes.is_empty(),
            _ => false,
        }
    }
    
    pub fn get_dependencies(&self) -> Vec<String> {
        let mut deps = Vec::new();
        self.collect_dependencies(&mut deps);
        deps.sort();
        deps.dedup();
        deps
    }
    
    fn collect_dependencies(&self, deps: &mut Vec<String>) {
        match self {
            ASTNode::Script(nodes) | ASTNode::Block(nodes) => {
                for node in nodes {
                    node.collect_dependencies(deps);
                }
            }
            ASTNode::Command { name, args, .. } => {
                // Check if it's an external command
                if !is_builtin(name) {
                    deps.push(name.clone());
                }
                for arg in args {
                    arg.collect_dependencies(deps);
                }
            }
            ASTNode::Pipeline(commands) => {
                for cmd in commands {
                    cmd.collect_dependencies(deps);
                }
            }
            ASTNode::If { condition, then_block, elif_blocks, else_block } => {
                condition.collect_dependencies(deps);
                then_block.collect_dependencies(deps);
                for (cond, block) in elif_blocks {
                    cond.collect_dependencies(deps);
                    block.collect_dependencies(deps);
                }
                if let Some(block) = else_block {
                    block.collect_dependencies(deps);
                }
            }
            ASTNode::While { condition, body } | ASTNode::Until { condition, body } => {
                condition.collect_dependencies(deps);
                body.collect_dependencies(deps);
            }
            ASTNode::For { items, body, .. } => {
                match items {
                    ForItems::List(list) => {
                        for item in list {
                            item.collect_dependencies(deps);
                        }
                    }
                    ForItems::Command(cmd) => cmd.collect_dependencies(deps),
                    ForItems::CStyle { init, condition, update } => {
                        init.collect_dependencies(deps);
                        condition.collect_dependencies(deps);
                        update.collect_dependencies(deps);
                    }
                }
                body.collect_dependencies(deps);
            }
            ASTNode::Case { expr, cases } => {
                expr.collect_dependencies(deps);
                for case in cases {
                    case.body.collect_dependencies(deps);
                }
            }
            ASTNode::Function { body, .. } => {
                body.collect_dependencies(deps);
            }
            ASTNode::CommandSubstitution(cmd) => {
                cmd.collect_dependencies(deps);
            }
            ASTNode::Subshell(cmd) => {
                cmd.collect_dependencies(deps);
            }
            _ => {}
        }
    }
}

fn is_builtin(command: &str) -> bool {
    matches!(command,
        "echo" | "printf" | "read" | "cd" | "pwd" | "exit" |
        "source" | "." | "exec" | "eval" | "export" | "unset" |
        "set" | "shift" | "return" | "break" | "continue" |
        "trap" | "wait" | "jobs" | "fg" | "bg" | "kill" |
        "test" | "[" | "[[" | "true" | "false" | ":" |
        "alias" | "unalias" | "type" | "which" | "command" |
        "builtin" | "declare" | "typeset" | "local" | "readonly"
    )
}