use smol_str::SmolStr;

use super::expression::Expr;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// Activates a new copy of the named entity, if it is an inactive zombie.
    Animate(Option<SmolStr>),
    /// Immediately deactivates the entity.
    Banish(Option<SmolStr>),
    /// Activates a new copy of the named entity, if it is an inactive ghost.
    Disturb(Option<SmolStr>),
    /// Instructs the entity to forget its remembered data value.
    Forget(Option<SmolStr>),
    /// Invokes a new copy of the named entity.
    Invoke(Option<SmolStr>),
    /// Instructs the entity to remember the sum of the values in the statement stack.
    /// Since a zombie can only remember one thing at a time, this causes it
    /// to forget any previously remembered value.
    Remember(Option<SmolStr>, Vec<Expr>),
    /// Print the text to the standard output.
    /// (It doesn't matter what entity does this, as the result is the same.)
    Say(Option<SmolStr>, Vec<Expr>),

    // Control flow
    /// Causes the entity to repeat the statements between shamble and until until the variable evaluates to true.
    ShambleUntil(Expr, Vec<Stmt>),
    /// Causes the entity to repeat the statements between shamble and around in an infinite loop.
    ShambleAround(Vec<Stmt>),
    /// Causes the current task to become inactive immediately.
    Stumble,
    /// If the variable evaluates to true, causes the entity to perform the statements between good and bad, otherwise perform the statements between bad and spit.
    Taste(Expr, Vec<Stmt>, Vec<Stmt>),
}
