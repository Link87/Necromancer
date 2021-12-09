use either::Either;
use indexmap::IndexSet;
use log::{debug, trace};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_till, take_until};
use nom::character::complete::{alpha1, alphanumeric0, char, digit1, multispace0, multispace1};
use nom::combinator::{all_consuming, eof, into, map, map_parser, map_res, not, peek, recognize};
use nom::error::Error;
use nom::multi::{many0, many1, many_till, separated_list1};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
use nom::{Finish, IResult};

use crate::scroll::creature::{Creature, Species};
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

impl<'a> Parse<'a> for Scroll<'a> {
    fn parse(code: &'a str) -> IResult<&'a str, Scroll> {
        trace!("Code (syntax tree): {}", code);
        multispace0(code)?;
        into(many1(terminated(Creature::parse, alt((eof, multispace1)))))(code)
    }
}

impl<'a> Parse<'a> for Creature<'a> {
    fn parse(code: &'a str) -> IResult<&'a str, Creature> {
        trace!("Code (creature): {}", code);
        let (code, (name, species)) = terminated(
            separated_pair(
                parse_identifier,
                tuple((multispace1, tag("is"), multispace1)),
                Species::parse,
            ),
            pair(multispace1, tag("summon")),
        )(code)?;

        let (code, (statements, spell)) = many_till(
            preceded(
                multispace1,
                alt((
                    map(
                        preceded(pair(tag("remember"), multispace1), Value::parse),
                        Either::Left,
                    ),
                    map(Task::parse, Either::Right),
                )),
            ),
            preceded(
                multispace1,
                alt((tag("animate"), tag("bind"), tag("disturb"))),
            ),
        )(code)?;

        let active = match (species, spell) {
            (Species::Zombie, "animate") => true,
            (Species::Ghost, "disturb") => true,
            (Species::Vampire, _) | (Species::Demon, _) | (Species::Djinn, _) => true, // "bind" spell
            _ => false,
        };

        let statements = statements
            .into_iter()
            .partition::<Vec<Either<Value, Task>>, _>(Either::is_left);
        let memory = statements
            .0
            .into_iter()
            .next()
            .map(Either::unwrap_left)
            .unwrap_or(Value::Void);
        let tasks = statements
            .1
            .into_iter()
            .map(Either::unwrap_right)
            .collect::<IndexSet<Task>>();

        debug!(
            "Summoning creature {} of species {:?} with {} tasks, using {}.",
            name,
            species,
            tasks.len(),
            spell
        );

        Ok((code, Creature::summon(name, species, active, memory, tasks)))
    }
}

impl<'a> Parse<'a> for Species {
    fn parse(code: &'a str) -> IResult<&'a str, Species> {
        trace!("Code (kind): {}", code);
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

impl<'a> Parse<'a> for Task<'a> {
    fn parse(code: &'a str) -> IResult<&'a str, Task> {
        trace!("Code (task): {}", code);
        map(
            tuple((
                preceded(pair(tag("task"), multispace1), parse_identifier),
                many0(preceded(multispace1, Stmt::parse)),
                preceded(
                    multispace1,
                    alt((map(tag("animate"), |_| true), map(tag("bind"), |_| false))),
                ),
            )),
            |(name, statements, active)| Task::new(name, active, statements),
        )(code)
    }
}

impl<'a> Parse<'a> for Stmt<'a> {
    fn parse(code: &'a str) -> IResult<&'a str, Stmt> {
        trace!("Code (statement): {}", code);
        alt((
            map(
                separated_pair(tag("animatex"), multispace1, parse_identifier),
                |(_, name)| {
                    // TODO
                    Stmt::Animate(Some(name))
                },
            ),
            map(tag("animatex"), |_| Stmt::Animate(None)), // TODO
            map(
                separated_pair(tag("banish"), multispace1, parse_identifier),
                |(_, name)| Stmt::Banish(Some(name)),
            ),
            map(tag("banish"), |_| Stmt::Banish(None)),
            map(
                separated_pair(tag("disturbx"), multispace1, parse_identifier),
                |(_, name)| {
                    // TODO
                    Stmt::Disturb(Some(name))
                },
            ),
            map(tag("disturbx"), |_| Stmt::Disturb(None)), // TODO
            map(
                separated_pair(tag("forget"), multispace1, parse_identifier),
                |(_, name)| Stmt::Forget(Some(name)),
            ),
            map(tag("forget"), |_| Stmt::Forget(None)),
            map(
                separated_pair(tag("invoke"), multispace1, parse_identifier),
                |(_, name)| Stmt::Invoke(Some(name)),
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
                |(_, _, name, _, exprs)| Stmt::Remember(Some(name), exprs),
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
                |(_, _, name, _, exprs)| Stmt::Say(Some(name), exprs),
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

impl<'a> Parse<'a> for Vec<Expr<'a>> {
    fn parse(code: &'a str) -> IResult<&'a str, Vec<Expr>> {
        trace!("Code (expression vec): {}", code);
        separated_list1(multispace1, Expr::parse)(code)
    }
}

impl<'a> Parse<'a> for Expr<'a> {
    fn parse(code: &'a str) -> IResult<&'a str, Expr> {
        trace!("Code (expression): {}", code);
        alt((
            map(
                separated_pair(tag("moan"), multispace1, parse_identifier),
                |(_, name)| Expr::Moan(Some(name)),
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
                |(_, _, name, _, value)| Expr::Remembering(Some(name), value),
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

fn parse_integer<'a>(code: &'a str) -> IResult<&'a str, i64> {
    trace!("Code (int): {}", code);
    map_res(
        alt((digit1, recognize(pair(char('-'), digit1)))),
        str::parse::<i64>,
    )(code)
}

fn parse_string<'a>(code: &'a str) -> IResult<&'a str, &'a str> {
    trace!("Code (string): {}", code);
    delimited(char('"'), take_till(|c| c == '\"'), char('"'))(code)
}

fn parse_identifier<'a>(code: &'a str) -> IResult<&'a str, &'a str> {
    trace!("Code (identifier): {}", code);
    peek(not(keyword))(code)?;
    recognize(pair(alpha1, alphanumeric0))(code)
}

fn keyword<'a>(code: &'a str) -> IResult<&'a str, &'a str> {
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

pub fn parse<'a>(code: &'a str) -> Result<Scroll, Error<&'a str>> {
    match Finish::finish(terminated(Scroll::parse, pair(multispace0, eof))(&code)) {
        Ok((_, tree)) => Ok(tree),
        Err(error) => Err(error),
    }
}
