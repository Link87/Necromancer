use super::expression::Expression;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Animate,
    AnimateNamed(String),
    Banish,
    BanishNamed(String),
    Disturb,
    DisturbNamed(String),
    Forget,
    ForgetNamed(String),
    Invoke,
    InvokeNamed(String),
    Remember(Vec<Expression>),
    RememberNamed(String, Vec<Expression>),
    Say(Vec<Expression>),
    SayNamed(String, Vec<Expression>),

    // Control flow
    ShambleUntil(Expression, Vec<Statement>),
    ShambleAround(Vec<Statement>),
    Stumble,
    Taste(Expression, Vec<Statement>, Vec<Statement>),
}
