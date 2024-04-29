use either::Either;
use log::{debug, trace};
use malachite::num::conversion::traits::FromSciString;
use malachite::Integer;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_till, take_until};
use nom::character::complete::{
    alpha1, alphanumeric0, anychar, char, digit1, multispace0, multispace1,
};
use nom::combinator::{
    all_consuming, complete, consumed, cut, eof, into, map, map_opt, map_parser, not, peek,
    recognize, rest_len, value,
};
use nom::error::Error;
use nom::multi::{many0, many1, many_till, separated_list1};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
use nom::{Finish, IResult};

use crate::scroll::entity::{Entity, Species, TaskList};
use crate::scroll::expression::Expr;
use crate::scroll::statement::Stmt;
use crate::scroll::task::Task;
use crate::scroll::Scroll;
use crate::value::Value;

#[cfg(test)]
mod tests;

trait Parse<'a> {
    fn parse(code: &'a str) -> IResult<&'a str, Self>
    where
        Self: Sized;
}

impl<'a> Parse<'a> for Scroll {
    fn parse(code: &'a str) -> IResult<&'a str, Scroll> {
        trace!("Code (syntax tree): {}", code);
        multispace0(code)?;
        into(complete(many1(terminated(
            Entity::parse,
            alt((recognize(pair(multispace0, eof)), recognize(multispace1))),
        ))))(code)
    }
}

impl<'a> Parse<'a> for Entity {
    fn parse(code: &'a str) -> IResult<&'a str, Entity> {
        // Leave any whitespace after the entity definition in the input.
        trace!("Code (entity): {}", code);
        let (code, (name, species)) = parse_entity_header(code)?;

        // Find the end of the entity definition and collect any code in between. Expect EOF or a new entity definition after this one.
        // End of entity definition is still in input after this.
        let (code, contents) = recognize(many_till(
            anychar,
            peek(tuple((
                multispace1,
                alt((tag("animate"), tag("bind"), tag("disturb"))),
                alt((
                    recognize(pair(multispace0, eof)),
                    recognize(pair(multispace1, parse_entity_header)),
                )),
            ))),
        ))(code)?;

        // Now actually parse the end of the entity definition.
        let (code, spell) = preceded(
            multispace1,
            alt((tag("animate"), tag("bind"), tag("disturb"))),
        )(code)?;

        trace!("Code (entity): content is {}", contents);

        // Parse the contents of the entity definition.
        let (_, statements) = many0(preceded(
            multispace1,
            alt((
                map(Task::parse, Either::Left),
                map(
                    preceded(pair(tag("remember"), multispace1), Value::parse),
                    Either::Right,
                ),
            )),
        ))(contents)?;

        let active = matches!(
            (species, spell),
            (Species::Zombie, "animate")
                | (Species::Ghost, "disturb")
                | (Species::Vampire, "bind")
                | (Species::Demon, "bind")
                | (Species::Djinn, "bind")
        );

        // Separate values and tasks into different Vecs.
        let statements = statements
            .into_iter()
            .partition::<Vec<Either<Task, Value>>, _>(Either::is_left);
        let tasks = statements
            .0
            .into_iter()
            .map(Either::unwrap_left)
            .map(|task| (task.name(), task))
            .collect::<TaskList>();
        let memory = statements
            .1
            .into_iter()
            .next()
            .map(Either::unwrap_right)
            .unwrap_or(Value::Void);

        debug!(
            "Summoning creature {} of species {:?} with {} tasks, using {}.",
            name,
            species,
            tasks.len(),
            spell
        );

        Ok((code, Entity::summon(name, species, active, memory, tasks)))
    }
}

fn parse_entity_header(code: &str) -> IResult<&str, (&str, Species)> {
    trace!("Code (entity header): {}", code);
    terminated(
        separated_pair(
            parse_identifier,
            tuple((multispace1, tag("is"), multispace1)),
            Species::parse,
        ),
        pair(multispace1, tag("summon")),
    )(code)
}

impl<'a> Parse<'a> for Species {
    fn parse(code: &'a str) -> IResult<&'a str, Species> {
        trace!("Code (species): {}", code);
        alt((
            map(tuple((tag("a"), multispace1, tag("zombie"))), |_| {
                Species::Zombie
            }),
            map(
                tuple((tag("an"), multispace1, tag("enslaved undead"))),
                |_| Species::Zombie,
            ),
            map(tuple((tag("a"), multispace1, tag("ghost"))), |_| {
                Species::Ghost
            }),
            map(
                tuple((tag("a"), multispace1, tag("restless undead"))),
                |_| Species::Ghost,
            ),
            map(tuple((tag("a"), multispace1, tag("vampire"))), |_| {
                Species::Vampire
            }),
            map(
                tuple((tag("a"), multispace1, tag("free-willed undead"))),
                |_| Species::Vampire,
            ),
            map(tuple((tag("a"), multispace1, tag("demon"))), |_| {
                Species::Demon
            }),
            map(tuple((tag("a"), multispace1, tag("djinn"))), |_| {
                Species::Djinn
            }),
        ))(code)
    }
}

impl<'a> Parse<'a> for Task {
    fn parse(code: &'a str) -> IResult<&'a str, Task> {
        // Parse anything until the next task defintion. Take the last animate or bind as the end of the task.
        trace!("Code (task): {}", code);

        let (code, name) = parse_task_header(code)?;

        // Find the beginning of the next task definition or the end of the input.
        // May include some remembers after the end of the task though.
        let (next, contents) = cut(recognize(many_till(
            anychar,
            peek(alt((
                recognize(pair(multispace0, eof)),
                recognize(pair(multispace1, parse_task_header)),
            ))),
        )))(code)?;

        // Now find the last animate or bind in the contents. Everything after that is remember statements outside the task.
        let (remembers, contents) = cut(recognize(many1(many_till(
            anychar,
            alt((tag("animate"), tag("bind"))),
        ))))(contents)?;

        // Remove the animate or bind at the end of the task.
        let (_, (contents, (_, active))) = cut(consumed(many_till(
            many_till(anychar, multispace1),
            peek(terminated(
                alt((value(true, tag("animate")), value(false, tag("bind")))),
                pair(multispace0, eof),
            )),
        )))(contents)?;
        trace!("Code (task): content is {}", contents);

        // Parse statements in the task.
        let (_, stmts) = many0(preceded(multispace1, Stmt::parse))(contents)?;

        let rest = &code[rest_len(code)?.1 - next.len() - remembers.len()..];
        Ok((rest, Task::new(name, active, stmts)))
    }
}

/// Parse the header of a task definition and return the task's name.
///
/// A task header is defined as the keyword `task` followed by a single identifier.
fn parse_task_header(code: &str) -> IResult<&str, &str> {
    trace!("Code (task header): {}", code);
    preceded(pair(tag("task"), multispace1), parse_identifier)(code)
}

impl<'a> Parse<'a> for Stmt {
    fn parse(code: &'a str) -> IResult<&'a str, Stmt> {
        trace!("Code (statement): {}", code);
        alt((
            map(
                separated_pair(tag("animate"), multispace1, parse_identifier),
                |(_, name)| Stmt::Animate(Some(name.into())),
            ),
            map(tag("animate"), |_| Stmt::Animate(None)),
            map(
                separated_pair(tag("banish"), multispace1, parse_identifier),
                |(_, name)| Stmt::Banish(Some(name.into())),
            ),
            map(tag("banish"), |_| Stmt::Banish(None)),
            map(
                separated_pair(tag("disturb"), multispace1, parse_identifier),
                |(_, name)| Stmt::Disturb(Some(name.into())),
            ),
            map(tag("disturb"), |_| Stmt::Disturb(None)),
            map(
                separated_pair(tag("forget"), multispace1, parse_identifier),
                |(_, name)| Stmt::Forget(Some(name.into())),
            ),
            map(tag("forget"), |_| Stmt::Forget(None)),
            map(
                separated_pair(tag("invoke"), multispace1, parse_identifier),
                |(_, name)| Stmt::Invoke(Some(name.into())),
            ),
            map(tag("invoke"), |_| Stmt::Invoke(None)),
            map(
                separated_pair(tag("remember"), multispace1, Vec::<Expr>::parse),
                |(_, exprs)| Stmt::Remember(None, exprs),
            ),
            map(
                tuple((
                    tag("remember"),
                    multispace1,
                    parse_identifier,
                    multispace1,
                    Vec::<Expr>::parse,
                )),
                |(_, _, name, _, exprs)| Stmt::Remember(Some(name.into()), exprs),
            ),
            map(
                separated_pair(tag("say"), multispace1, Vec::<Expr>::parse),
                |(_, exprs)| Stmt::Say(None, exprs),
            ),
            map(
                tuple((
                    tag("say"),
                    multispace1,
                    parse_identifier,
                    multispace1,
                    Vec::<Expr>::parse,
                )),
                |(_, _, name, _, exprs)| Stmt::Say(Some(name.into()), exprs),
            ),
            map(
                delimited(
                    pair(tag("shamble"), multispace1),
                    map_parser(
                        take_until("around"),
                        all_consuming(many0(terminated(Stmt::parse, multispace1))),
                    ),
                    tag("around"),
                ),
                Stmt::ShambleAround,
            ),
            map(
                tuple((
                    pair(tag("shamble"), multispace1),
                    map_parser(
                        take_until("until"),
                        all_consuming(many0(terminated(Stmt::parse, multispace1))),
                    ),
                    preceded(pair(tag("until"), multispace1), Expr::parse),
                )),
                |(_, statements, expr)| Stmt::ShambleUntil(expr, statements),
            ),
            map(tag("stumble"), |_| Stmt::Stumble),
            map(
                tuple((
                    preceded(pair(tag("taste"), multispace1), Expr::parse),
                    preceded(
                        tuple((multispace1, tag("good"), multispace1)),
                        map_parser(
                            take_until("bad"),
                            all_consuming(many0(terminated(Stmt::parse, multispace1))),
                        ),
                    ),
                    delimited(
                        pair(tag("bad"), multispace1),
                        map_parser(
                            take_until("spit"),
                            all_consuming(many0(terminated(Stmt::parse, multispace1))),
                        ),
                        tag("spit"),
                    ),
                )),
                |(condition, good, bad)| Stmt::Taste(condition, good, bad),
            ),
        ))(code)
    }
}

impl<'a> Parse<'a> for Vec<Expr> {
    fn parse(code: &'a str) -> IResult<&'a str, Vec<Expr>> {
        trace!("Code (expression vec): {}", code);
        separated_list1(multispace1, Expr::parse)(code)
    }
}

impl<'a> Parse<'a> for Expr {
    fn parse(code: &'a str) -> IResult<&'a str, Expr> {
        trace!("Code (expression): {}", code);
        alt((
            map(
                separated_pair(tag("moan"), multispace1, parse_identifier),
                |(_, name)| Expr::Moan(Some(name.into())),
            ),
            map(tag("moan"), |_| Expr::Moan(None)),
            map(
                tuple((
                    tag("remembering"),
                    multispace1,
                    parse_identifier,
                    multispace1,
                    Value::parse,
                )),
                |(_, _, name, _, value)| Expr::Remembering(Some(name.into()), value),
            ),
            map(
                separated_pair(tag("remembering"), multispace1, Value::parse),
                |(_, value)| Expr::Remembering(None, value),
            ),
            map(tag("rend"), |_| Expr::Rend),
            map(tag("turn"), |_| Expr::Turn),
            map(Value::parse, Expr::Value),
        ))(code)
    }
}

impl<'a> Parse<'a> for Value {
    fn parse(code: &'a str) -> IResult<&'a str, Value> {
        trace!("Code (value): {}", code);
        alt((
            map(parse_integer, Value::Integer),
            map(parse_string, |s| Value::String(String::from(s))),
        ))(code)
    }
}

/// Parse an integer.
///
/// Supports positive and negative integers.
fn parse_integer(code: &str) -> IResult<&str, Integer> {
    trace!("Code (int): {}", code);
    map_opt(
        alt((digit1, recognize(pair(char('-'), digit1)))),
        FromSciString::from_sci_string,
    )(code)
}

/// Parse a string.
///
/// Strings are delimited by double quotes ("").
fn parse_string(code: &str) -> IResult<&str, &str> {
    trace!("Code (string): {}", code);
    delimited(char('"'), take_till(|c| c == '\"'), char('"'))(code)
}

/// Parse an identifier.
///
/// An identifier is a string of alphanumeric characters starting with a letter. Keywords are not allowed as identifiers.
fn parse_identifier(code: &str) -> IResult<&str, &str> {
    trace!("Code (identifier): {}", code);
    peek(not(keyword))(code)?;
    recognize(pair(alpha1, alphanumeric0))(code)
}

/// Recognize a keyword.
///
/// Returns `Ok` if the input starts with a keyword, otherwise `Err`.
fn keyword(code: &str) -> IResult<&str, &str> {
    recognize(alt((
        alt((
            tag("zombie"),
            tag("enslaved undead"),
            tag("ghost"),
            tag("restless undead"),
            tag("vampire"),
            tag("free-willed undead"),
            tag("demon"),
            tag("djin"),
            tag("summon"),
            tag("animate"),
            tag("disturb"),
            tag("bind"),
            tag("task"),
            tag("remember"),
            tag("moan"),
            tag("banish"),
            tag("forget"),
            tag("invoke"),
            tag("say"),
            tag("shamble"),
            tag("until"),
        )),
        alt((
            tag("around"),
            tag("stumble"),
            tag("taste"),
            tag("good"),
            tag("spit"),
            tag("remembering"),
            tag("rend"),
            tag("turn"),
        )),
    )))(code)
}

pub fn parse(code: &str) -> Result<Scroll, Error<&str>> {
    match Finish::finish(terminated(Scroll::parse, pair(multispace0, eof))(code)) {
        Ok((_, tree)) => Ok(tree),
        Err(error) => Err(error),
    }
}
