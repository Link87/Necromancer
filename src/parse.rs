use either::*;
use log::{debug, trace};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_till};
use nom::character::complete::{alphanumeric1, char, digit1, multispace0, multispace1};
use nom::combinator::{eof, map, recognize};
use nom::error::{Error, ErrorKind};
use nom::multi::{many0, many1, many_till};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
use nom::{Err, Finish, IResult};

use crate::entity::{Entity, EntityKind};
use crate::statement::{Statement, StatementCmd};
use crate::task::Task;
use crate::value::Value;

trait Parse<'a> {
    fn parse(code: &'a str) -> IResult<&'a str, Self>
    where
        Self: Sized;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxTree {
    entities: Vec<Entity>,
}

impl SyntaxTree {
    fn new(entities: Vec<Entity>) -> SyntaxTree {
        SyntaxTree { entities }
    }

    pub fn entities(&self) -> &Vec<Entity> {
        &self.entities
    }
}

impl<'a> Parse<'a> for SyntaxTree {
    fn parse(code: &'a str) -> IResult<&'a str, SyntaxTree> {
        trace!("Code (syntax tree): {}", code);
        multispace0(code)?;
        map(
            many1(terminated(Entity::parse, alt((eof, multispace1)))),
            SyntaxTree::new,
        )(code)
    }
}

impl<'a> Parse<'a> for Entity {
    fn parse(code: &'a str) -> IResult<&'a str, Entity> {
        trace!("Code (entity): {}", code);
        let (code, (name, kind)) = terminated(
            separated_pair(
                alphanumeric1,
                tuple((multispace1, tag("is"), multispace1)),
                EntityKind::parse,
            ),
            pair(multispace1, tag("summon")),
        )(code)?;

        let (code, (statements, spell)) = many_till(
            preceded(
                multispace1,
                alt((
                    map(
                        preceded(pair(tag("remember"), multispace1), Value::parse),
                        Left,
                    ),
                    map(Task::parse, Right),
                )),
            ),
            preceded(
                multispace1,
                alt((tag("animate"), tag("bind"), tag("disturb"))),
            ),
        )(code)?;

        let active = match (kind, spell) {
            (EntityKind::Zombie, "animate") => true,
            (EntityKind::Ghost, "disturb") => true,
            (EntityKind::Vampire, _) | (EntityKind::Demon, _) | (EntityKind::Djinn, _) => true,
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
            .collect::<Vec<Task>>();

        debug!(
            "Summoning entity {} of kind {:?} with {} tasks, using {}.",
            name,
            kind,
            tasks.len(),
            spell
        );

        Ok((
            code,
            Entity::summon(kind, String::from(name), active, memory, tasks),
        ))
    }
}

impl<'a> Parse<'a> for EntityKind {
    fn parse(code: &'a str) -> IResult<&'a str, EntityKind> {
        trace!("Code (kind): {}", code);
        alt((
            map(tuple((tag("a"), multispace1, tag("zombie"))), |_| {
                EntityKind::Zombie
            }),
            map(
                tuple((tag("an"), multispace1, tag("enslaved undead"))),
                |_| EntityKind::Zombie,
            ),
            map(tuple((tag("a"), multispace1, tag("ghost"))), |_| {
                EntityKind::Ghost
            }),
            map(
                tuple((tag("a"), multispace1, tag("restless undead"))),
                |_| EntityKind::Ghost,
            ),
            map(tuple((tag("a"), multispace1, tag("vampire"))), |_| {
                EntityKind::Vampire
            }),
            map(
                tuple((tag("a"), multispace1, tag("free-willed undead"))),
                |_| EntityKind::Vampire,
            ),
            map(tuple((tag("a"), multispace1, tag("demon"))), |_| {
                EntityKind::Demon
            }),
            map(tuple((tag("a"), multispace1, tag("djinn"))), |_| {
                EntityKind::Djinn
            }),
        ))(code)
    }
}

impl<'a> Parse<'a> for Task {
    fn parse(code: &'a str) -> IResult<&'a str, Task> {
        trace!("Code (task): {}", code);
        map(
            tuple((
                preceded(pair(tag("task"), multispace1), alphanumeric1),
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
        map(
            alt((
                preceded(
                    pair(tag("say"), multispace1),
                    map(Value::parse, StatementCmd::Say),
                ),
                preceded(
                    pair(tag("remember"), multispace1),
                    map(Value::parse, StatementCmd::Remember),
                ),
            )),
            Statement::new,
        )(code)
    }
}

impl<'a> Parse<'a> for Value {
    fn parse(code: &'a str) -> IResult<&'a str, Value> {
        println!("Code (value): {}", code);
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

pub fn parse<'a>(code: &'a str) -> Result<SyntaxTree, Error<&'a str>> {
    match Finish::finish(terminated(SyntaxTree::parse, pair(multispace0, eof))(&code)) {
        Ok((_, tree)) => Ok(tree),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::statement::StatementCmd;
    use crate::value::Value;

    #[test]
    fn parse_entities() {
        let code = "\
Peter is a zombie
summon
animate

Jay is an enslaved undead
summon
animate

Sarah is a zombie
summon
animate

Max is a free-willed undead
summon
animate

Anna is a djinn
summon
animate

Beatrix is a demon
summon
animate";

        let tree = parse(code).unwrap();

        assert_eq!(tree.entities().len(), 6);

        assert_eq!(tree.entities()[0].kind(), EntityKind::Zombie);
        assert_eq!(tree.entities()[0].name(), "Peter");
        assert_eq!(tree.entities()[0].moan(), Value::Void);

        assert_eq!(tree.entities()[1].kind(), EntityKind::Zombie);
        assert_eq!(tree.entities()[1].name(), "Jay");
        assert_eq!(tree.entities()[1].moan(), Value::Void);

        assert_eq!(tree.entities()[2].kind(), EntityKind::Zombie);
        assert_eq!(tree.entities()[2].name(), "Sarah");
        assert_eq!(tree.entities()[2].moan(), Value::Void);

        assert_eq!(tree.entities()[3].kind(), EntityKind::Vampire);
        assert_eq!(tree.entities()[3].name(), "Max");
        assert_eq!(tree.entities()[3].moan(), Value::Void);

        assert_eq!(tree.entities()[4].kind(), EntityKind::Djinn);
        assert_eq!(tree.entities()[4].name(), "Anna");
        assert_eq!(tree.entities()[4].moan(), Value::Void);

        assert_eq!(tree.entities()[5].kind(), EntityKind::Demon);
        assert_eq!(tree.entities()[5].name(), "Beatrix");
        assert_eq!(tree.entities()[5].moan(), Value::Void);
    }

    #[test]
    fn skip_whitespace() {
        let code = "\

   Peter is a zombie\tsummon
   \r\n\nanimate
    
\t\t";

        let tree = parse(code).unwrap();
        assert_eq!(tree.entities().len(), 1);

        assert_eq!(tree.entities()[0].kind(), EntityKind::Zombie);
        assert_eq!(tree.entities()[0].name(), "Peter");
        assert_eq!(tree.entities()[0].moan(), Value::Void);
    }

    #[test]
    fn parse_tasks() {
        let code = "\
Peter is a zombie
summon
    task Test1
    animate
    task Test2
    animate
animate

Jay is an enslaved undead
summon
    task Test3
    animate
    task Test1
    animate
animate";

        let tree = parse(code).unwrap();

        assert_eq!(tree.entities()[0].tasks().len(), 2);
        assert_eq!(tree.entities()[0].tasks()[0].name(), "Test1");
        assert_eq!(tree.entities()[0].tasks()[1].name(), "Test2");

        assert_eq!(tree.entities()[0].tasks().len(), 2);
        assert_eq!(tree.entities()[1].tasks()[0].name(), "Test3");
        assert_eq!(tree.entities()[1].tasks()[1].name(), "Test1");
    }

    #[test]
    fn parse_remember() {
        let code = "\
Peter is a zombie
summon
    remember -161
animate

Jay is an enslaved undead
summon
    task Test1
    animate
    remember 1312
    task Test2
    animate
animate";

        let tree = parse(code).unwrap();

        assert_eq!(tree.entities()[0].tasks().len(), 0);
        assert_eq!(tree.entities()[0].moan(), Value::Integer(-161));

        assert_eq!(tree.entities()[1].tasks().len(), 2);
        assert_eq!(tree.entities()[1].tasks()[0].name(), "Test1");
        assert_eq!(tree.entities()[1].tasks()[1].name(), "Test2");
        assert_eq!(tree.entities()[1].moan(), Value::Integer(1312));
    }

    #[test]
    fn parse_i64() {
        let (_, num) = parse_integer("2341").unwrap();
        assert_eq!(num, 2341);

        let (_, num) = parse_integer("-2341").unwrap();
        assert_eq!(num, -2341);

        let (_, num) = parse_integer("0").unwrap();
        assert_eq!(num, 0);
    }

    #[test]
    fn parse_str() {
        let (_, s) = parse_string("\"\"").unwrap();
        assert_eq!(s, "");

        let (_, s) = parse_string("\"foo\"").unwrap();
        assert_eq!(s, "foo");

        let (_, s) = parse_string("\"bar\"  fadf").unwrap();
        assert_eq!(s, "bar");
    }

    #[test]
    fn parse_value() {
        let (_, num) = Value::parse("2341").unwrap();
        assert_eq!(num, Value::Integer(2341));

        let (_, num) = Value::parse("-2341").unwrap();
        assert_eq!(num, Value::Integer(-2341));

        let (_, num) = Value::parse("0").unwrap();
        assert_eq!(num, Value::Integer(0));

        let (_, s) = Value::parse("\"\"").unwrap();
        assert_eq!(s, Value::String(String::from("")));

        let (_, s) = Value::parse("\"foo\"").unwrap();
        assert_eq!(s, Value::String(String::from("foo")));

        let (_, s) = Value::parse("\"bar\"  fadf").unwrap();
        assert_eq!(s, Value::String(String::from("bar")));
    }

    #[test]
    fn parse_say_value() {
        let code = "\
Peter is a zombie
summon
    task Test1
        say -161
        say 1312
        say \"+161\"
        say \"Hello World\"
    animate
animate
";

        let tree = parse(code).unwrap();

        assert_eq!(tree.entities()[0].tasks().len(), 1);
        assert_eq!(tree.entities()[0].tasks()[0].statements().len(), 4);

        assert_eq!(
            tree.entities()[0].tasks()[0].statements()[0].cmd(),
            &StatementCmd::Say(Value::Integer(-161))
        );
        assert_eq!(
            tree.entities()[0].tasks()[0].statements()[1].cmd(),
            &StatementCmd::Say(Value::Integer(1312))
        );
        assert_eq!(
            tree.entities()[0].tasks()[0].statements()[2].cmd(),
            &StatementCmd::Say(Value::String(String::from("+161")))
        );
        assert_eq!(
            tree.entities()[0].tasks()[0].statements()[3].cmd(),
            &StatementCmd::Say(Value::String(String::from("Hello World")))
        );
    }

    #[test]
    fn parse_active() {
        let code = "\
Peter is a zombie
summon
    task Test1
    bind
    task Test2
    animate
animate

Jay is an enslaved undead
summon
    task Test3
    animate
    task Test1
    bind
bind

Myrte is a ghost
summon
disturb

BuhHuh is a ghost
summon
bind

Max is a free-willed undead
summon
bind

Anna is a djinn
summon
bind

Beatrix is a demon
summon
bind";

        let tree = parse(code).unwrap();

        assert_eq!(tree.entities()[0].active(), true);
        assert_eq!(tree.entities()[0].tasks().len(), 2);
        assert_eq!(tree.entities()[0].tasks()[0].active(), false);
        assert_eq!(tree.entities()[0].tasks()[1].active(), true);

        assert_eq!(tree.entities()[1].active(), false);
        assert_eq!(tree.entities()[1].tasks().len(), 2);
        assert_eq!(tree.entities()[1].tasks()[0].active(), true);
        assert_eq!(tree.entities()[1].tasks()[1].active(), false);

        assert_eq!(tree.entities()[2].active(), true);
        assert_eq!(tree.entities()[3].active(), false);
        assert_eq!(tree.entities()[4].active(), true);
        assert_eq!(tree.entities()[5].active(), true);
        assert_eq!(tree.entities()[6].active(), true);
    }
}
