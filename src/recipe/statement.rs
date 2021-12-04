use super::expression::Expression;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement<'a> {
    Animate,
    AnimateNamed(&'a str),
    Banish,
    BanishNamed(&'a str),
    Disturb,
    DisturbNamed(&'a str),
    Forget,
    ForgetNamed(&'a str),
    Invoke,
    InvokeNamed(&'a str),
    Remember(Vec<Expression<'a>>),
    RememberNamed(&'a str, Vec<Expression<'a>>),
    Say(Vec<Expression<'a>>),
    SayNamed(&'a str, Vec<Expression<'a>>),

    // Control flow
    ShambleUntil(Expression<'a>, Vec<Statement<'a>>),
    ShambleAround(Vec<Statement<'a>>),
    Stumble,
    Taste(Expression<'a>, Vec<Statement<'a>>, Vec<Statement<'a>>),
}
