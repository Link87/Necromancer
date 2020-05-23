#[derive(Debug)]
pub struct Entity<'a> {
    kind: EntityKind,
    name: &'a str,
    remember: Remember<i64>,
    is_active: bool,
}

#[derive(Debug, Copy, Clone)]
pub enum Remember<T>
where
    T: Copy + Clone,
{
    Value(T),
    None,
}

#[derive(Debug)]
pub enum EntityKind {
    Zombie,
    Ghost,
    Vampire,
    Demon,
    Djinn,
}

impl Entity<'_> {
    pub fn new<'a>(kind: EntityKind, name: &'a str) -> Entity {
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

    fn _remembering(&self) -> Remember<i64> {
        self.remember
    }

    fn _summon(&mut self, spell: &str) {
        self.is_active = spell == "animate";
    }
}
