use nom::branch::alt;
use nom::bytes::complete::{tag, take_till, take_while};
use nom::character::complete::{alphanumeric1, anychar, multispace0, space1};
use nom::combinator::not;
use nom::error::ErrorKind;
use nom::{Err, IResult};

use crate::entity::{Entity, EntityKind};

pub trait Parse<'a> {
    fn parse(code: &'a str) -> IResult<&'a str, Self>
    where
        Self: Sized;
}

#[derive(Debug)]
pub struct SyntaxTree<'a> {
    entities: Vec<Entity<'a>>,
}

impl SyntaxTree<'_> {
    fn new(entities: Vec<Entity>) -> SyntaxTree {
        SyntaxTree { entities }
    }
}

impl<'a> Parse<'a> for SyntaxTree<'a> {
    fn parse(code: &'a str) -> IResult<&'a str, SyntaxTree<'a>> {
        // let entities: Vec<Entity<'a>> = Vec::new();
        let (rest, entities) = nom::multi::many1(|code| Entity::parse(code))(code)?;

        // TODO validate rest

        Ok((rest, SyntaxTree::new(entities)))
    }
}

impl<'a> Parse<'a> for Entity<'a> {
    fn parse(code: &'a str) -> IResult<&'a str, Entity<'a>> {
        let (code_rest, line) = read_line(code)?;
        println!("line: {:?}", line);
        println!("code rest: {:?}", code_rest);
        let (line_rest, name) = alphanumeric1(line)?;
        let (line_rest, _) = space1(line_rest)?;
        let (line_rest, _) = alt((tag("is a"), tag("is an")))(line_rest)?;
        let (line_rest, _) = space1(line_rest)?;
        let (line_rest, kind) = EntityKind::parse(line_rest)?;
        let _ = assert_line_ending(line_rest)?;

        let (code_rest, line) = read_line(code_rest)?;
        println!("line: {:?}", line);
        let (line_rest, _) = tag("summon")(line)?;
        let _ = assert_line_ending(line_rest)?;

        println!("Summoning entity {} of kind {:?}.", name, kind);

        Ok((code_rest, Entity::new(kind, name)))
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
            "vampire" => Ok((rest, EntityKind::Vampire)),
            "demon" => Ok((rest, EntityKind::Demon)),
            "djinn" => Ok((rest, EntityKind::Djinn)),
            _ => panic!("Unrecognized entity kind: {:?}", kind),
        }
    }
}

fn read_line<'a>(code: &'a str) -> IResult<&'a str, &'a str> {
    let (line, _) = multispace0(code)?;
    let (rest, line) = take_while(|c| c != '\r' && c != '\n')(line)?;
    let (_, line) = take_till_comment(line)?;
    // TODO drop lines that only contain a comment
    Ok((rest, line))
}

fn take_till_comment<'a>(code: &'a str) -> IResult<&'a str, &'a str> {
    take_till(|c| c == '#')(code)
}

fn assert_line_ending<'a>(code: &'a str) -> IResult<&'a str, ()> {
    not(anychar)(code)
}

pub fn parse(code: &str) -> Result<SyntaxTree, Err<(&str, ErrorKind)>> {
    match SyntaxTree::parse(code) {
        Ok((_, tree)) => Ok(tree),
        Err(error) => Err(error),
    }
}

#[cfg(tests)]
mod tests {
    use super::*;

    #[test]
    fn parse_entities() {
        let code = "\
Peter is a zombie
summon
    task Greet
        say \"Hello World!\"
    animate
animate";

        assert_eq!(parse(code), SyntaxTree {
            vec![
                Zombie {
                },
            ],
        });
    }
}
