use super::*;
use crate::value::Value;

#[test]
fn parse_creatures() {
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

    let recipe = parse(code).unwrap();

    assert_eq!(recipe.creatures().len(), 6);

    assert_eq!(
        recipe.creatures().get("Peter").unwrap().species(),
        Species::Zombie
    );
    assert_eq!(recipe.creatures().get("Peter").unwrap().name(), "Peter");
    assert_eq!(recipe.creatures().get("Peter").unwrap().moan(), Value::Void);

    assert_eq!(
        recipe.creatures().get("Jay").unwrap().species(),
        Species::Zombie
    );
    assert_eq!(recipe.creatures().get("Jay").unwrap().name(), "Jay");
    assert_eq!(recipe.creatures().get("Jay").unwrap().moan(), Value::Void);

    assert_eq!(
        recipe.creatures().get("Sarah").unwrap().species(),
        Species::Zombie
    );
    assert_eq!(recipe.creatures().get("Sarah").unwrap().name(), "Sarah");
    assert_eq!(recipe.creatures().get("Sarah").unwrap().moan(), Value::Void);

    assert_eq!(
        recipe.creatures().get("Max").unwrap().species(),
        Species::Vampire
    );
    assert_eq!(recipe.creatures().get("Max").unwrap().name(), "Max");
    assert_eq!(recipe.creatures().get("Max").unwrap().moan(), Value::Void);

    assert_eq!(
        recipe.creatures().get("Anna").unwrap().species(),
        Species::Djinn
    );
    assert_eq!(recipe.creatures().get("Anna").unwrap().name(), "Anna");
    assert_eq!(recipe.creatures().get("Anna").unwrap().moan(), Value::Void);

    assert_eq!(
        recipe.creatures().get("Beatrix").unwrap().species(),
        Species::Demon
    );
    assert_eq!(recipe.creatures().get("Beatrix").unwrap().name(), "Beatrix");
    assert_eq!(
        recipe.creatures().get("Beatrix").unwrap().moan(),
        Value::Void
    );
}

#[test]
fn skip_whitespace() {
    let code = "\

   Peter is a zombie\tsummon
   \r\n\nanimate
    
\t\t";

    let recipe = parse(code).unwrap();
    assert_eq!(recipe.creatures().len(), 1);

    assert_eq!(
        recipe.creatures().get("Peter").unwrap().species(),
        Species::Zombie
    );
    assert_eq!(recipe.creatures().get("Peter").unwrap().name(), "Peter");
    assert_eq!(recipe.creatures().get("Peter").unwrap().moan(), Value::Void);
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

    let recipe = parse(code).unwrap();

    assert_eq!(recipe.creatures().get("Peter").unwrap().tasks().len(), 2);
    assert_eq!(
        recipe
            .creatures()
            .get("Peter")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .name(),
        "Test1"
    );
    assert_eq!(
        recipe
            .creatures()
            .get("Peter")
            .unwrap()
            .tasks()
            .get("Test2")
            .unwrap()
            .name(),
        "Test2"
    );

    assert_eq!(recipe.creatures().get("Jay").unwrap().tasks().len(), 2);
    assert_eq!(
        recipe
            .creatures()
            .get("Jay")
            .unwrap()
            .tasks()
            .get("Test3")
            .unwrap()
            .name(),
        "Test3"
    );
    assert_eq!(
        recipe
            .creatures()
            .get("Jay")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .name(),
        "Test1"
    );
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

    let recipe = parse(code).unwrap();

    assert_eq!(recipe.creatures().get("Peter").unwrap().tasks().len(), 0);
    assert_eq!(
        recipe.creatures().get("Peter").unwrap().moan(),
        Value::Integer(-161)
    );

    assert_eq!(recipe.creatures().get("Jay").unwrap().tasks().len(), 2);
    assert_eq!(
        recipe
            .creatures()
            .get("Jay")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .name(),
        "Test1"
    );
    assert_eq!(
        recipe
            .creatures()
            .get("Jay")
            .unwrap()
            .tasks()
            .get("Test2")
            .unwrap()
            .name(),
        "Test2"
    );
    assert_eq!(
        recipe.creatures().get("Jay").unwrap().moan(),
        Value::Integer(1312)
    );
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

    let recipe = parse(code).unwrap();

    assert_eq!(recipe.creatures().get("Peter").unwrap().tasks().len(), 1);
    assert_eq!(
        recipe
            .creatures()
            .get("Peter")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .statements()
            .len(),
        7
    );

    assert_eq!(
        recipe
            .creatures()
            .get("Peter")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .statements()
            .get(0)
            .unwrap(),
        &Statement::Say(Value::Integer(-161))
    );
    assert_eq!(
        recipe
            .creatures()
            .get("Peter")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .statements()
            .get(1)
            .unwrap(),
        &Statement::Say(Value::Integer(1312))
    );
    assert_eq!(
        recipe
            .creatures()
            .get("Peter")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .statements()
            .get(2)
            .unwrap(),
        &Statement::Say(Value::String(String::from("+161")))
    );
    assert_eq!(
        recipe
            .creatures()
            .get("Peter")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .statements()
            .get(3)
            .unwrap(),
        &Statement::Say(Value::String(String::from("Hello World")))
    );
    assert_eq!(
        recipe
            .creatures()
            .get("Peter")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .statements()
            .get(4)
            .unwrap(),
        &Statement::SayNamed(String::from("Markus"), Value::Integer(-161))
    );
    assert_eq!(
        recipe
            .creatures()
            .get("Peter")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .statements()
            .get(5)
            .unwrap(),
        &Statement::SayNamed(String::from("Dorni"), Value::Integer(1312))
    );
    assert_eq!(
        recipe
            .creatures()
            .get("Peter")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .statements()
            .get(6)
            .unwrap(),
        &Statement::SayNamed(
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

    let recipe = parse(code).unwrap();

    assert_eq!(recipe.creatures().get("Peter").unwrap().tasks().len(), 1);
    assert_eq!(
        recipe
            .creatures()
            .get("Peter")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .statements()
            .len(),
        7
    );

    assert_eq!(
        recipe
            .creatures()
            .get("Peter")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .statements()
            .get(0)
            .unwrap(),
        &Statement::Remember(Value::Integer(-161))
    );
    assert_eq!(
        recipe
            .creatures()
            .get("Peter")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .statements()
            .get(1)
            .unwrap(),
        &Statement::Remember(Value::Integer(1312))
    );
    assert_eq!(
        recipe
            .creatures()
            .get("Peter")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .statements()
            .get(2)
            .unwrap(),
        &Statement::Remember(Value::String(String::from("+161")))
    );
    assert_eq!(
        recipe
            .creatures()
            .get("Peter")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .statements()
            .get(3)
            .unwrap(),
        &Statement::Remember(Value::String(String::from("Hello World")))
    );
    assert_eq!(
        recipe
            .creatures()
            .get("Peter")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .statements()
            .get(4)
            .unwrap(),
        &Statement::RememberNamed(String::from("Markus"), Value::Integer(-161))
    );
    assert_eq!(
        recipe
            .creatures()
            .get("Peter")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .statements()
            .get(5)
            .unwrap(),
        &Statement::RememberNamed(String::from("Dorni"), Value::Integer(1312))
    );
    assert_eq!(
        recipe
            .creatures()
            .get("Peter")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .statements()
            .get(6)
            .unwrap(),
        &Statement::RememberNamed(
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

    let recipe = parse(code).unwrap();

    assert_eq!(recipe.creatures().get("Peter").unwrap().tasks().len(), 1);
    assert_eq!(
        recipe
            .creatures()
            .get("Peter")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .statements()
            .len(),
        8
    );

    assert_eq!(
        recipe
            .creatures()
            .get("Peter")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .statements()
            .get(0)
            .unwrap(),
        &Statement::Remember(Value::Integer(-161))
    );
    assert_eq!(
        recipe
            .creatures()
            .get("Peter")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .statements()
            .get(1)
            .unwrap(),
        &Statement::Remember(Value::Integer(1312))
    );
    assert_eq!(
        recipe
            .creatures()
            .get("Peter")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .statements()
            .get(2)
            .unwrap(),
        &Statement::Banish,
    );
    assert_eq!(
        recipe
            .creatures()
            .get("Peter")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .statements()
            .get(3)
            .unwrap(),
        &Statement::BanishNamed(String::from("Peter")),
    );
    assert_eq!(
        recipe
            .creatures()
            .get("Peter")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .statements()
            .get(4)
            .unwrap(),
        &Statement::ForgetNamed(String::from("Peter")),
    );
    assert_eq!(
        recipe
            .creatures()
            .get("Peter")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .statements()
            .get(5)
            .unwrap(),
        &Statement::Forget,
    );
    assert_eq!(
        recipe
            .creatures()
            .get("Peter")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .statements()
            .get(6)
            .unwrap(),
        &Statement::Invoke,
    );
    assert_eq!(
        recipe
            .creatures()
            .get("Peter")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .statements()
            .get(7)
            .unwrap(),
        &Statement::InvokeNamed(String::from("Peter")),
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

    let recipe = parse(code).unwrap();

    assert_eq!(recipe.creatures().get("Peter").unwrap().active(), true);
    assert_eq!(recipe.creatures().get("Peter").unwrap().tasks().len(), 2);
    assert_eq!(
        recipe
            .creatures()
            .get("Peter")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .active(),
        false
    );
    assert_eq!(
        recipe
            .creatures()
            .get("Peter")
            .unwrap()
            .tasks()
            .get("Test2")
            .unwrap()
            .active(),
        true
    );

    assert_eq!(recipe.creatures().get("Jay").unwrap().active(), false);
    assert_eq!(recipe.creatures().get("Jay").unwrap().tasks().len(), 2);
    assert_eq!(
        recipe
            .creatures()
            .get("Jay")
            .unwrap()
            .tasks()
            .get("Test3")
            .unwrap()
            .active(),
        true
    );
    assert_eq!(
        recipe
            .creatures()
            .get("Jay")
            .unwrap()
            .tasks()
            .get("Test1")
            .unwrap()
            .active(),
        false
    );

    assert_eq!(recipe.creatures().get("Myrte").unwrap().active(), true);
    assert_eq!(recipe.creatures().get("BuhHuh").unwrap().active(), false);
    assert_eq!(recipe.creatures().get("Max").unwrap().active(), true);
    assert_eq!(recipe.creatures().get("Anna").unwrap().active(), true);
    assert_eq!(recipe.creatures().get("Beatrix").unwrap().active(), true);
}
