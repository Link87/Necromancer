use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, anychar, char, digit1, multispace0, multispace1};
use nom::combinator::{complete, not, opt, peek};
use nom::error::ErrorKind;
use nom::multi::many1;
use nom::{Err, IResult};

use crate::entity::{Entity, EntityKind, Memory};
use crate::task::Task;

trait Parse<'a> {
    fn parse(code: &'a str) -> IResult<&'a str, Self>
    where
        Self: Sized;
}

#[derive(Debug)]
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
        let (code, entities) = many1(|code| {
            let (code, entities) = Entity::parse(code)?;
            let (code, _) = multispace0(code)?;
            Ok((code, entities))
        })(code)?;
        assert_eof(code)?;

        Ok((code, SyntaxTree::new(entities)))
    }
}

impl<'a> Parse<'a> for Entity {
    fn parse(code: &'a str) -> IResult<&'a str, Entity> {
        let (code, _) = multispace0(code)?;
        let (code, name) = alphanumeric1(code)?;
        let (code, _) = multispace1(code)?;
        let (code, _) = tag("is")(code)?;
        let (code, _) = multispace1(code)?;
        let (code, _) = alt((tag("an"), tag("a")))(code)?;
        let (code, _) = multispace1(code)?;

        let (code, kind) = EntityKind::parse(code)?;

        let (code, _) = multispace1(code)?;
        let (code, _) = tag("summon")(code)?;

        let mut tasks = Vec::new();
        let mut memory = Memory::None;
        let mut code = code;
        loop {
            // TODO fix when destructuring assignments (RFC 372) come to rust
            let (lcode, _) = multispace1(code)?;
            let (lcode, action) = opt(alt((peek(tag("remember")), peek(tag("task")))))(lcode)?;
            match action {
                Some("remember") => {
                    let (lcode, lmemory) = Memory::parse(lcode)?;
                    memory = lmemory;
                    code = lcode;
                }
                Some("task") => {
                    let (lcode, task) = Task::parse(lcode)?;
                    tasks.push(task);
                    code = lcode;
                }
                _ => break,
            }
        }

        let (code, _) = multispace1(code)?;
        let (code, spell) = alt((tag("animate"), tag("bind"), tag("disturb")))(code)?;

        println!(
            "Summoning entity {} of kind {:?} with {} tasks, using {}.",
            name,
            kind,
            tasks.len(),
            spell
        );

        let active = match (kind, spell) {
            (EntityKind::Zombie, "animate") => true,
            (EntityKind::Ghost, "disturb") => true,
            (EntityKind::Vampire, _) | (EntityKind::Demon, _) | (EntityKind::Djinn, _) => true,
            _ => false,
        };

        Ok((
            code,
            Entity::summon(kind, String::from(name), active, memory, tasks),
        ))
    }
}

impl<'a> Parse<'a> for EntityKind {
    fn parse(code: &'a str) -> IResult<&'a str, EntityKind> {
        let (rest, kind) = alt((
            tag("zombie"),
            tag("enslaved undead"),
            tag("ghost"),
            tag("restless undead"),
            tag("vampire"),
            tag("free-willed undead"),
            tag("demon"),
            tag("djinn"),
        ))(code)?;
        match kind {
            "zombie" | "enslaved undead" => Ok((rest, EntityKind::Zombie)),
            "ghost" | "restless undead" => Ok((rest, EntityKind::Ghost)),
            "vampire" | "free-willed undead" => Ok((rest, EntityKind::Vampire)),
            "demon" => Ok((rest, EntityKind::Demon)),
            "djinn" => Ok((rest, EntityKind::Djinn)),
            _ => panic!("Unrecognized entity kind: {:?}", kind),
        }
    }
}

impl<'a> Parse<'a> for Task {
    fn parse(code: &'a str) -> IResult<&'a str, Task> {
        println!("Code (task): {}", code);
        let (code, _) = multispace0(code)?;
        let (code, _) = tag("task")(code)?;
        let (code, _) = multispace1(code)?;
        let (code, name) = alphanumeric1(code)?;

        let (code, _) = multispace1(code)?;
        let (code, spell) = alt((tag("animate"), tag("bind")))(code)?;

        Ok((code, Task::new(String::from(name), spell == "animate")))
    }
}

impl<'a> Parse<'a> for Memory {
    fn parse(code: &'a str) -> IResult<&'a str, Memory> {
        println!("Code (task): {}", code);
        let (code, _) = multispace0(code)?;
        let (code, _) = tag("remember")(code)?;
        let (code, _) = multispace1(code)?;
        let (code, value) = parse_integer(code)?;

        Ok((code, Memory::Number(value)))
    }
}

fn assert_eof<'a>(code: &'a str) -> IResult<&'a str, ()> {
    not(anychar)(code)
}

fn parse_integer<'a>(code: &'a str) -> IResult<&'a str, i64> {
    let mut sign: i64 = 1;
    let mut code = code;
    if let Ok((lcode, _)) = char::<_, (&'a str, ErrorKind)>('-')(code) {
        sign = -1;
        code = lcode;
    }

    let (code, num) = digit1(code)?;
    let num = str::parse::<i64>(num);

    match num {
        Ok(num) => Ok((code, num * sign)),
        Err(_) => Err(nom::Err::Error((code, ErrorKind::Digit))),
    }
}

pub fn parse<'a>(code: &'a str) -> Result<SyntaxTree, Err<(&'a str, ErrorKind)>> {
    match complete(SyntaxTree::parse)(code) {
        Ok((_, tree)) => Ok(tree),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::Memory;

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
animate
";

        let tree = parse(code).unwrap();

        assert_eq!(tree.entities().len(), 6);

        assert_eq!(tree.entities()[0].kind(), EntityKind::Zombie);
        assert_eq!(tree.entities()[0].name(), "Peter");
        assert_eq!(tree.entities()[0].moan(), Memory::None);

        assert_eq!(tree.entities()[1].kind(), EntityKind::Zombie);
        assert_eq!(tree.entities()[1].name(), "Jay");
        assert_eq!(tree.entities()[1].moan(), Memory::None);

        assert_eq!(tree.entities()[2].kind(), EntityKind::Zombie);
        assert_eq!(tree.entities()[2].name(), "Sarah");
        assert_eq!(tree.entities()[2].moan(), Memory::None);

        assert_eq!(tree.entities()[3].kind(), EntityKind::Vampire);
        assert_eq!(tree.entities()[3].name(), "Max");
        assert_eq!(tree.entities()[3].moan(), Memory::None);

        assert_eq!(tree.entities()[4].kind(), EntityKind::Djinn);
        assert_eq!(tree.entities()[4].name(), "Anna");
        assert_eq!(tree.entities()[4].moan(), Memory::None);

        assert_eq!(tree.entities()[5].kind(), EntityKind::Demon);
        assert_eq!(tree.entities()[5].name(), "Beatrix");
        assert_eq!(tree.entities()[5].moan(), Memory::None);
    }

    #[test]
    fn skip_whitespace() {
        let code = "\

   Peter is a zombie
summon
   animate
    
\t\t";

        let tree = parse(code).unwrap();
        assert_eq!(tree.entities().len(), 1);

        assert_eq!(tree.entities()[0].kind(), EntityKind::Zombie);
        assert_eq!(tree.entities()[0].name(), "Peter");
        assert_eq!(tree.entities()[0].moan(), Memory::None);
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
        assert_eq!(tree.entities()[0].moan(), Memory::Number(-161));

        assert_eq!(tree.entities()[1].tasks().len(), 2);
        assert_eq!(tree.entities()[1].tasks()[0].name(), "Test1");
        assert_eq!(tree.entities()[1].tasks()[1].name(), "Test2");
        assert_eq!(tree.entities()[1].moan(), Memory::Number(1312));
    }

    #[test]
    fn parse_i64() -> Result<(), Err<(&'static str, ErrorKind)>> {
        let (_, num) = parse_integer("2341")?;
        assert_eq!(num, 2341);

        let (_, num) = parse_integer("-2341")?;
        assert_eq!(num, -2341);

        let (_, num) = parse_integer("0")?;
        assert_eq!(num, 0);

        Ok(())
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
