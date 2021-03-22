use either::Either;
use indexmap::IndexSet;
use log::{debug, trace};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_till};
use nom::character::complete::{alpha1, alphanumeric0, char, digit1, multispace0, multispace1};
use nom::combinator::{eof, into, map, not, peek, recognize, success};
use nom::error::{Error, ErrorKind};
use nom::multi::{many0, many1, many_till};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
use nom::{Err, Finish, IResult};

use crate::recipe::creature::{Creature, Species};
use crate::recipe::statement::Statement;
use crate::recipe::task::Task;
use crate::recipe::Recipe;
use crate::value::Value;

#[cfg(test)]
mod tests;

trait Parse<'a> {
    fn parse(code: &'a str) -> IResult<&'a str, Self>
    where
        Self: Sized;
}

impl<'a> Parse<'a> for Recipe {
    fn parse(code: &'a str) -> IResult<&'a str, Recipe> {
        trace!("Code (syntax tree): {}", code);
        multispace0(code)?;
        into(many1(terminated(Creature::parse, alt((eof, multispace1)))))(code)
    }
}

impl<'a> Parse<'a> for Creature {
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
            (Species::Vampire, _) | (Species::Demon, _) | (Species::Djinn, _) => true,
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

        Ok((
            code,
            Creature::summon(String::from(name), species, active, memory, tasks),
        ))
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

impl<'a> Parse<'a> for Task {
    fn parse(code: &'a str) -> IResult<&'a str, Task> {
        trace!("Code (task): {}", code);
        map(
            tuple((
                preceded(pair(tag("task"), multispace1), parse_identifier),
                many0(preceded(multispace1, Statement::parse)),
                preceded(
                    multispace1,
                    alt((map(tag("animate"), |_| true), map(tag("bind"), |_| false))),
                ),
            )),
            |(name, statements, active)| Task::new(String::from(name), active, statements),
        )(code)
    }
}

impl<'a> Parse<'a> for Statement {
    fn parse(code: &'a str) -> IResult<&'a str, Statement> {
        trace!("Code (statement): {}", code);
        alt((
            preceded(
                // TOOD
                tag("animatex"),
                alt((
                    map(preceded(multispace1, parse_identifier), |name| {
                        Statement::AnimateNamed(String::from(name))
                    }),
                    success(Statement::Animate),
                )),
            ),
            preceded(
                tag("banish"),
                alt((
                    map(preceded(multispace1, parse_identifier), |name| {
                        Statement::BanishNamed(String::from(name))
                    }),
                    success(Statement::Banish),
                )),
            ),
            preceded(
                // TOOD
                tag("disturbx"),
                alt((
                    map(preceded(multispace1, parse_identifier), |name| {
                        Statement::DisturbNamed(String::from(name))
                    }),
                    success(Statement::Disturb),
                )),
            ),
            preceded(
                tag("forget"),
                alt((
                    map(preceded(multispace1, parse_identifier), |name| {
                        Statement::ForgetNamed(String::from(name))
                    }),
                    success(Statement::Forget),
                )),
            ),
            preceded(
                tag("invoke"),
                alt((
                    map(preceded(multispace1, parse_identifier), |name| {
                        Statement::InvokeNamed(String::from(name))
                    }),
                    success(Statement::Invoke),
                )),
            ),
            preceded(
                pair(tag("remember"), multispace1),
                alt((
                    map(Value::parse, Statement::Remember),
                    map(
                        separated_pair(parse_identifier, multispace1, Value::parse),
                        |(name, value)| Statement::RememberNamed(String::from(name), value),
                    ),
                )),
            ),
            preceded(
                pair(tag("say"), multispace1),
                alt((
                    map(Value::parse, Statement::Say),
                    map(
                        separated_pair(parse_identifier, multispace1, Value::parse),
                        |(name, value)| Statement::SayNamed(String::from(name), value),
                    ),
                )),
            ),
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
    let (code, num) = alt((digit1, recognize(pair(char('-'), digit1))))(code)?;

    match str::parse::<i64>(num) {
        Ok(num) => Ok((code, num)),
        Err(_) => Err(Err::Error(Error {
            input: code,
            code: ErrorKind::Digit,
        })),
    }
}

fn parse_string<'a>(code: &'a str) -> IResult<&'a str, &'a str> {
    delimited(char('"'), take_till(|c| c == '\"'), char('"'))(code)
}

fn parse_identifier<'a>(code: &'a str) -> IResult<&'a str, &'a str> {
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

pub fn parse<'a>(code: &'a str) -> Result<Recipe, Error<&'a str>> {
    match Finish::finish(terminated(Recipe::parse, pair(multispace0, eof))(&code)) {
        Ok((_, tree)) => Ok(tree),
        Err(error) => Err(error),
    }
}

