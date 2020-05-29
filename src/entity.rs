#[derive(Debug)]
pub struct Entity {
    kind: EntityKind,
    name: String,
    remember: Remember<i64>,
    is_active: bool,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Remember<T>
where
    T: Copy + Clone + Eq + Ord,
{
    Value(T),
    None,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum EntityKind {
    Zombie,
    Ghost,
    Vampire,
    Demon,
    Djinn,
}

impl Entity {
    pub fn new<'a>(kind: EntityKind, name: String) -> Entity {
        Entity {
            kind,
            name,
            remember: Remember::None,
            is_active: false,
        }
    }

    fn _remember(&mut self, value: Remember<i64>) {
        self.remember = value;
    }

    pub fn remembering(&self) -> Remember<i64> {
        self.remember
    }

    pub fn kind(&self) -> EntityKind {
        self.kind
    }

    pub fn name<'a>(&'a self) -> &'a str {
        &self.name
    }

    pub fn active(&self) -> bool {
        self.is_active
    }

    fn _summon(&mut self, spell: &str) {
        self.is_active = spell == "animate";
    }
}
