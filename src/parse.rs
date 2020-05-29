use nom::branch::alt;
use nom::bytes::complete::{tag, take_till, take_while};
use nom::character::complete::{alphanumeric1, anychar, multispace0, space1};
use nom::combinator::not;
use nom::error::ErrorKind;
use nom::{Err, IResult};

use crate::entity::{Entity, EntityKind};

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
        let (rest, entities) = nom::multi::many1(|code| Entity::parse(code))(code)?;
        let (rest, line) = read_line(rest)?;
        assert_line_ending(line)?;
        assert_line_ending(rest)?;

        Ok((rest, SyntaxTree::new(entities)))
    }
}

impl<'a> Parse<'a> for Entity {
    fn parse(code: &'a str) -> IResult<&'a str, Entity> {
        let (code_rest, line) = read_line(code)?;
        println!("line: {:?}", line);
        let (line_rest, name) = alphanumeric1(line)?;
        let (line_rest, _) = space1(line_rest)?;
        let (line_rest, _) = alt((tag("is an"), tag("is a")))(line_rest)?;
        let (line_rest, _) = space1(line_rest)?;
        let (line_rest, kind) = EntityKind::parse(line_rest)?;
        let _ = assert_line_ending(line_rest)?;

        let (code_rest, line) = read_line(code_rest)?;
        println!("line: {:?}", line);
        let (line_rest, _) = tag("summon")(line)?;
        let _ = assert_line_ending(line_rest)?;

        let (code_rest, line) = read_line(code_rest)?;
        println!("line: {:?}", line);
        let (line_rest, _) = tag("animate")(line)?;
        let _ = assert_line_ending(line_rest)?;

        println!("Summoning entity {} of kind {:?}.", name, kind);

        Ok((code_rest, Entity::new(kind, String::from(name))))
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

pub fn parse<'a>(code: &'a str) -> Result<SyntaxTree, Err<(&'a str, ErrorKind)>> {
    match SyntaxTree::parse(code) {
        Ok((_, tree)) => Ok(tree),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::Remember;

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
        assert_eq!(tree.entities()[0].remembering(), Remember::None);

        assert_eq!(tree.entities()[1].kind(), EntityKind::Zombie);
        assert_eq!(tree.entities()[1].name(), "Jay");
        assert_eq!(tree.entities()[1].remembering(), Remember::None);

        assert_eq!(tree.entities()[2].kind(), EntityKind::Zombie);
        assert_eq!(tree.entities()[2].name(), "Sarah");
        assert_eq!(tree.entities()[2].remembering(), Remember::None);

        assert_eq!(tree.entities()[3].kind(), EntityKind::Vampire);
        assert_eq!(tree.entities()[3].name(), "Max");
        assert_eq!(tree.entities()[3].remembering(), Remember::None);

        assert_eq!(tree.entities()[4].kind(), EntityKind::Djinn);
        assert_eq!(tree.entities()[4].name(), "Anna");
        assert_eq!(tree.entities()[4].remembering(), Remember::None);

        assert_eq!(tree.entities()[5].kind(), EntityKind::Demon);
        assert_eq!(tree.entities()[5].name(), "Beatrix");
        assert_eq!(tree.entities()[5].remembering(), Remember::None);
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
        assert_eq!(tree.entities()[0].remembering(), Remember::None);
    }
}