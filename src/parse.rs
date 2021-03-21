use std::collections::HashMap;

use either::*;
use indexmap::IndexMap;
use log::{debug, trace};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_till};
use nom::character::complete::{alpha1, alphanumeric0, char, digit1, multispace0, multispace1};
use nom::combinator::{eof, into, map, not, peek, recognize, success};
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
    entities: HashMap<String, Entity>,
}

impl SyntaxTree {
    fn new(entities: HashMap<String, Entity>) -> SyntaxTree {
        SyntaxTree { entities }
    }

    pub fn entities(&self) -> &HashMap<String, Entity> {
        &self.entities
    }
}

impl From<Vec<Entity>> for SyntaxTree {
    fn from(value: Vec<Entity>) -> SyntaxTree {
        SyntaxTree::new(
            value
                .into_iter()
                .map(|e| (String::from(e.name()), e))
                .collect(),
        )
    }
}

impl<'a> Parse<'a> for SyntaxTree {
    fn parse(code: &'a str) -> IResult<&'a str, SyntaxTree> {
        trace!("Code (syntax tree): {}", code);
        multispace0(code)?;
        into(many1(terminated(Entity::parse, alt((eof, multispace1)))))(code)
    }
}

impl<'a> Parse<'a> for Entity {
    fn parse(code: &'a str) -> IResult<&'a str, Entity> {
        trace!("Code (entity): {}", code);
        let (code, (name, kind)) = terminated(
            separated_pair(
                parse_identifier,
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
            .map(|t| (String::from(t.name()), t))
            .collect::<IndexMap<String, Task>>();

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
        map(
            alt((
                preceded(
                    // TOOD
                    tag("animatex"),
                    alt((
                        map(preceded(multispace1, parse_identifier), |name| {
                            StatementCmd::AnimateNamed(String::from(name))
                        }),
                        success(StatementCmd::Animate),
                    )),
                ),
                preceded(
                    tag("banish"),
                    alt((
                        map(preceded(multispace1, parse_identifier), |name| {
                            StatementCmd::BanishNamed(String::from(name))
                        }),
                        success(StatementCmd::Banish),
                    )),
                ),
                preceded(
                    // TOOD
                    tag("disturbx"),
                    alt((
                        map(preceded(multispace1, parse_identifier), |name| {
                            StatementCmd::DisturbNamed(String::from(name))
                        }),
                        success(StatementCmd::Disturb),
                    )),
                ),
                preceded(
                    tag("forget"),
                    alt((
                        map(preceded(multispace1, parse_identifier), |name| {
                            StatementCmd::ForgetNamed(String::from(name))
                        }),
                        success(StatementCmd::Forget),
                    )),
                ),
                preceded(
                    tag("invoke"),
                    alt((
                        map(preceded(multispace1, parse_identifier), |name| {
                            StatementCmd::InvokeNamed(String::from(name))
                        }),
                        success(StatementCmd::Invoke),
                    )),
                ),
                preceded(
                    pair(tag("remember"), multispace1),
                    alt((
                        map(Value::parse, StatementCmd::Remember),
                        map(
                            separated_pair(parse_identifier, multispace1, Value::parse),
                            |(name, value)| StatementCmd::RememberNamed(String::from(name), value),
                        ),
                    )),
                ),
                preceded(
                    pair(tag("say"), multispace1),
                    alt((
                        map(Value::parse, StatementCmd::Say),
                        map(
                            separated_pair(parse_identifier, multispace1, Value::parse),
                            |(name, value)| StatementCmd::SayNamed(String::from(name), value),
                        ),
                    )),
                ),
            )),
            Statement::new,
        )(code)
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

        assert_eq!(tree.entities()["Peter"].kind(), EntityKind::Zombie);
        assert_eq!(tree.entities()["Peter"].name(), "Peter");
        assert_eq!(tree.entities()["Peter"].moan(), Value::Void);

        assert_eq!(tree.entities()["Jay"].kind(), EntityKind::Zombie);
        assert_eq!(tree.entities()["Jay"].name(), "Jay");
        assert_eq!(tree.entities()["Jay"].moan(), Value::Void);

        assert_eq!(tree.entities()["Sarah"].kind(), EntityKind::Zombie);
        assert_eq!(tree.entities()["Sarah"].name(), "Sarah");
        assert_eq!(tree.entities()["Sarah"].moan(), Value::Void);

        assert_eq!(tree.entities()["Max"].kind(), EntityKind::Vampire);
        assert_eq!(tree.entities()["Max"].name(), "Max");
        assert_eq!(tree.entities()["Max"].moan(), Value::Void);

        assert_eq!(tree.entities()["Anna"].kind(), EntityKind::Djinn);
        assert_eq!(tree.entities()["Anna"].name(), "Anna");
        assert_eq!(tree.entities()["Anna"].moan(), Value::Void);

        assert_eq!(tree.entities()["Beatrix"].kind(), EntityKind::Demon);
        assert_eq!(tree.entities()["Beatrix"].name(), "Beatrix");
        assert_eq!(tree.entities()["Beatrix"].moan(), Value::Void);
    }

    #[test]
    fn skip_whitespace() {
        let code = "\

   Peter is a zombie\tsummon
   \r\n\nanimate
    
\t\t";

        let tree = parse(code).unwrap();
        assert_eq!(tree.entities().len(), 1);

        assert_eq!(tree.entities()["Peter"].kind(), EntityKind::Zombie);
        assert_eq!(tree.entities()["Peter"].name(), "Peter");
        assert_eq!(tree.entities()["Peter"].moan(), Value::Void);
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

        assert_eq!(tree.entities()["Peter"].tasks().len(), 2);
        assert_eq!(tree.entities()["Peter"].tasks()["Test1"].name(), "Test1");
        assert_eq!(tree.entities()["Peter"].tasks()["Test2"].name(), "Test2");

        assert_eq!(tree.entities()["Jay"].tasks().len(), 2);
        assert_eq!(tree.entities()["Jay"].tasks()["Test3"].name(), "Test3");
        assert_eq!(tree.entities()["Jay"].tasks()["Test1"].name(), "Test1");
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

        assert_eq!(tree.entities()["Peter"].tasks().len(), 0);
        assert_eq!(tree.entities()["Peter"].moan(), Value::Integer(-161));

        assert_eq!(tree.entities()["Jay"].tasks().len(), 2);
        assert_eq!(tree.entities()["Jay"].tasks()["Test1"].name(), "Test1");
        assert_eq!(tree.entities()["Jay"].tasks()["Test2"].name(), "Test2");
        assert_eq!(tree.entities()["Jay"].moan(), Value::Integer(1312));
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
        say Markus -161
        say Dorni  1312
        say Isa \t\"Hello World\"
    animate
animate
";

        let tree = parse(code).unwrap();

        assert_eq!(tree.entities()["Peter"].tasks().len(), 1);
        assert_eq!(
            tree.entities()["Peter"].tasks()["Test1"].statements().len(),
            7
        );

        assert_eq!(
            tree.entities()["Peter"].tasks()["Test1"].statements()[0].cmd(),
            &StatementCmd::Say(Value::Integer(-161))
        );
        assert_eq!(
            tree.entities()["Peter"].tasks()["Test1"].statements()[1].cmd(),
            &StatementCmd::Say(Value::Integer(1312))
        );
        assert_eq!(
            tree.entities()["Peter"].tasks()["Test1"].statements()[2].cmd(),
            &StatementCmd::Say(Value::String(String::from("+161")))
        );
        assert_eq!(
            tree.entities()["Peter"].tasks()["Test1"].statements()[3].cmd(),
            &StatementCmd::Say(Value::String(String::from("Hello World")))
        );
        assert_eq!(
            tree.entities()["Peter"].tasks()["Test1"].statements()[4].cmd(),
            &StatementCmd::SayNamed(String::from("Markus"), Value::Integer(-161))
        );
        assert_eq!(
            tree.entities()["Peter"].tasks()["Test1"].statements()[5].cmd(),
            &StatementCmd::SayNamed(String::from("Dorni"), Value::Integer(1312))
        );
        assert_eq!(
            tree.entities()["Peter"].tasks()["Test1"].statements()[6].cmd(),
            &StatementCmd::SayNamed(
                String::from("Isa"),
                Value::String(String::from("Hello World"))
            )
        );
    }

    #[test]
    fn parse_remember_value() {
        let code = "\
Peter is a zombie
summon
    task Test1
        remember -161
        remember 1312
        remember \"+161\"
        remember \"Hello World\"
        remember Markus -161
        remember Dorni  1312
        remember Isa \t\"Hello World\"
    animate
animate
";

        let tree = parse(code).unwrap();

        assert_eq!(tree.entities()["Peter"].tasks().len(), 1);
        assert_eq!(
            tree.entities()["Peter"].tasks()["Test1"].statements().len(),
            7
        );

        assert_eq!(
            tree.entities()["Peter"].tasks()["Test1"].statements()[0].cmd(),
            &StatementCmd::Remember(Value::Integer(-161))
        );
        assert_eq!(
            tree.entities()["Peter"].tasks()["Test1"].statements()[1].cmd(),
            &StatementCmd::Remember(Value::Integer(1312))
        );
        assert_eq!(
            tree.entities()["Peter"].tasks()["Test1"].statements()[2].cmd(),
            &StatementCmd::Remember(Value::String(String::from("+161")))
        );
        assert_eq!(
            tree.entities()["Peter"].tasks()["Test1"].statements()[3].cmd(),
            &StatementCmd::Remember(Value::String(String::from("Hello World")))
        );
        assert_eq!(
            tree.entities()["Peter"].tasks()["Test1"].statements()[4].cmd(),
            &StatementCmd::RememberNamed(String::from("Markus"), Value::Integer(-161))
        );
        assert_eq!(
            tree.entities()["Peter"].tasks()["Test1"].statements()[5].cmd(),
            &StatementCmd::RememberNamed(String::from("Dorni"), Value::Integer(1312))
        );
        assert_eq!(
            tree.entities()["Peter"].tasks()["Test1"].statements()[6].cmd(),
            &StatementCmd::RememberNamed(
                String::from("Isa"),
                Value::String(String::from("Hello World"))
            )
        );
    }

    #[test]
    fn parse_statements() {
        let code = "\
Peter is a zombie
summon
    task Test1
        remember -161
        remember 1312
        banish
        banish Peter
        forget Peter
        forget
        invoke
        invoke Peter
    animate
animate
";

        let tree = parse(code).unwrap();

        assert_eq!(tree.entities()["Peter"].tasks().len(), 1);
        assert_eq!(
            tree.entities()["Peter"].tasks()["Test1"].statements().len(),
            8
        );

        assert_eq!(
            tree.entities()["Peter"].tasks()["Test1"].statements()[0].cmd(),
            &StatementCmd::Remember(Value::Integer(-161))
        );
        assert_eq!(
            tree.entities()["Peter"].tasks()["Test1"].statements()[1].cmd(),
            &StatementCmd::Remember(Value::Integer(1312))
        );
        assert_eq!(
            tree.entities()["Peter"].tasks()["Test1"].statements()[2].cmd(),
            &StatementCmd::Banish,
        );
        assert_eq!(
            tree.entities()["Peter"].tasks()["Test1"].statements()[3].cmd(),
            &StatementCmd::BanishNamed(String::from("Peter")),
        );
        assert_eq!(
            tree.entities()["Peter"].tasks()["Test1"].statements()[4].cmd(),
            &StatementCmd::ForgetNamed(String::from("Peter")),
        );
        assert_eq!(
            tree.entities()["Peter"].tasks()["Test1"].statements()[5].cmd(),
            &StatementCmd::Forget,
        );
        assert_eq!(
            tree.entities()["Peter"].tasks()["Test1"].statements()[6].cmd(),
            &StatementCmd::Invoke,
        );
        assert_eq!(
            tree.entities()["Peter"].tasks()["Test1"].statements()[7].cmd(),
            &StatementCmd::InvokeNamed(String::from("Peter")),
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

        assert_eq!(tree.entities()["Peter"].active(), true);
        assert_eq!(tree.entities()["Peter"].tasks().len(), 2);
        assert_eq!(tree.entities()["Peter"].tasks()["Test1"].active(), false);
        assert_eq!(tree.entities()["Peter"].tasks()["Test2"].active(), true);

        assert_eq!(tree.entities()["Jay"].active(), false);
        assert_eq!(tree.entities()["Jay"].tasks().len(), 2);
        assert_eq!(tree.entities()["Jay"].tasks()["Test3"].active(), true);
        assert_eq!(tree.entities()["Jay"].tasks()["Test1"].active(), false);

        assert_eq!(tree.entities()["Myrte"].active(), true);
        assert_eq!(tree.entities()["BuhHuh"].active(), false);
        assert_eq!(tree.entities()["Max"].active(), true);
        assert_eq!(tree.entities()["Anna"].active(), true);
        assert_eq!(tree.entities()["Beatrix"].active(), true);
    }
}
