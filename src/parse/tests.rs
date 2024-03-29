use super::*;
use crate::scroll::expression::Expr;
use crate::value::Value;

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[test]
fn parse_creatures() {
    init();

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
    init();

    let code = "\
\
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
    init();

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
    init();

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
    init();

    let (_, num) = parse_integer("2341").unwrap();
    assert_eq!(num, 2341);

    let (_, num) = parse_integer("-2341").unwrap();
    assert_eq!(num, -2341);

    let (_, num) = parse_integer("0").unwrap();
    assert_eq!(num, 0);
}

#[test]
fn parse_str() {
    init();

    let (_, s) = parse_string("\"\"").unwrap();
    assert_eq!(s, "");

    let (_, s) = parse_string("\"foo\"").unwrap();
    assert_eq!(s, "foo");

    let (_, s) = parse_string("\"bar\"  fadf").unwrap();
    assert_eq!(s, "bar");
}

#[test]
fn parse_value() {
    init();

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
    init();

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
        &Stmt::Say(None, vec![Expr::Value(Value::Integer(-161))])
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
        &Stmt::Say(None, vec![Expr::Value(Value::Integer(1312))])
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
        &Stmt::Say(None, vec![Expr::Value(Value::String(String::from("+161")))])
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
        &Stmt::Say(
            None,
            vec![Expr::Value(Value::String(String::from("Hello World")))]
        )
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
        &Stmt::Say(Some("Markus".into()), vec![Expr::Value(Value::Integer(-161))])
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
        &Stmt::Say(Some("Dorni".into()), vec![Expr::Value(Value::Integer(1312))])
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
        &Stmt::Say(
            Some("Isa".into()),
            vec![Expr::Value(Value::String(String::from("Hello World")))]
        )
    );
}

#[test]
fn parse_remember_value() {
    init();

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
        &Stmt::Remember(None, vec![Expr::Value(Value::Integer(-161))])
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
        &Stmt::Remember(None, vec![Expr::Value(Value::Integer(1312))])
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
        &Stmt::Remember(None, vec![Expr::Value(Value::String(String::from("+161")))])
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
        &Stmt::Remember(
            None,
            vec![Expr::Value(Value::String(String::from("Hello World")))]
        )
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
        &Stmt::Remember(Some("Markus".into()), vec![Expr::Value(Value::Integer(-161))])
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
        &Stmt::Remember(Some("Dorni".into()), vec![Expr::Value(Value::Integer(1312))])
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
        &Stmt::Remember(
            Some("Isa".into()),
            vec![Expr::Value(Value::String(String::from("Hello World")))]
        )
    );
}

#[test]
fn parse_statements() {
    init();

    let code = "\
Peter is a zombie
summon
    task Test1
        remember -161
        remember 1312
        animate
        animate Peter
        banish
        banish Peter
        disturb
        disturb Peter
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
        12
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
        &Stmt::Remember(None, vec![Expr::Value(Value::Integer(-161))])
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
        &Stmt::Remember(None, vec![Expr::Value(Value::Integer(1312))])
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
        &Stmt::Animate(None),
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
        &Stmt::Animate(Some("Peter".into())),
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
        &Stmt::Banish(None),
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
        &Stmt::Banish(Some("Peter".into())),
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
        &Stmt::Disturb(None),
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
        &Stmt::Disturb(Some("Peter".into())),
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
            .get(8)
            .unwrap(),
        &Stmt::Forget(Some("Peter".into())),
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
            .get(9)
            .unwrap(),
        &Stmt::Forget(None),
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
            .get(10)
            .unwrap(),
        &Stmt::Invoke(None),
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
            .get(11)
            .unwrap(),
        &Stmt::Invoke(Some("Peter".into())),
    );
}

#[test]
fn parse_active() {
    init();

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

#[test]
fn fibonacci() {
    init();

    let code = "\
Zombie1 is a zombie
summon
    remember 1
bind

Zombie2 is a zombie
summon
    remember 1
bind

Fibonacci is a zombie
summon
    remember 0
    task SayFibonaccis
        shamble
            say moan Zombie1
            say moan Zombie2
            remember Zombie1 moan Zombie1 moan Zombie2
            remember Zombie2 moan Zombie1 moan Zombie2
            remember moan Zombie2
        until remembering 100
    animate
animate";

    let recipe = parse(code).unwrap();

    assert_eq!(recipe.creatures().len(), 3);

    assert_eq!(recipe.creatures().get("Zombie1").unwrap().active(), false);
    assert_eq!(recipe.creatures().get("Zombie1").unwrap().tasks().len(), 0);
    assert_eq!(
        recipe.creatures().get("Zombie1").unwrap().moan(),
        Value::Integer(1)
    );

    assert_eq!(recipe.creatures().get("Zombie2").unwrap().active(), false);
    assert_eq!(recipe.creatures().get("Zombie2").unwrap().tasks().len(), 0);
    assert_eq!(
        recipe.creatures().get("Zombie2").unwrap().moan(),
        Value::Integer(1)
    );

    assert_eq!(recipe.creatures().get("Fibonacci").unwrap().active(), true);
    assert_eq!(
        recipe.creatures().get("Fibonacci").unwrap().tasks().len(),
        1
    );
    assert_eq!(
        recipe
            .creatures()
            .get("Fibonacci")
            .unwrap()
            .tasks()
            .get("SayFibonaccis")
            .unwrap()
            .active(),
        true
    );

    let statements = recipe
        .creatures()
        .get("Fibonacci")
        .unwrap()
        .tasks()
        .get("SayFibonaccis")
        .unwrap()
        .statements();

    assert_eq!(statements.len(), 1);

    match &statements[0] {
        Stmt::ShambleUntil(expr, statements) => {
            assert_eq!(expr, &Expr::Remembering(None, Value::Integer(100)));

            assert_eq!(statements.len(), 5);
            assert_eq!(
                statements[0],
                Stmt::Say(None, vec![Expr::Moan(Some("Zombie1".into()))])
            );
            assert_eq!(
                statements[1],
                Stmt::Say(None, vec![Expr::Moan(Some("Zombie2".into()))])
            );
            assert_eq!(
                statements[2],
                Stmt::Remember(
                    Some("Zombie1".into()),
                    vec![Expr::Moan(Some("Zombie1".into())), Expr::Moan(Some("Zombie2".into()))]
                )
            );
            assert_eq!(
                statements[3],
                Stmt::Remember(
                    Some("Zombie2".into()),
                    vec![Expr::Moan(Some("Zombie1".into())), Expr::Moan(Some("Zombie2".into()))]
                )
            );
            assert_eq!(
                statements[4],
                Stmt::Remember(None, vec![Expr::Moan(Some("Zombie2".into()))])
            );
        }
        _ => assert!(false),
    }
}

#[test]
fn parse_control_flow() {
    init();

    let code = "\
Peter is a zombie
summon
    task Test1
        shamble
            say 1312
            remember moan
        around
        shamble around
        remember \"foo\"
        stumble
        shamble
            say 1312
            remember moan
        until remembering 42
        shamble until remembering 42
        taste moan good
            say 1312
            remember moan
        bad
            stumble
        spit
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
        &Stmt::ShambleAround(vec![
            Stmt::Say(None, vec![Expr::Value(Value::Integer(1312))]),
            Stmt::Remember(None, vec![Expr::Moan(None)]),
        ])
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
        &Stmt::ShambleAround(vec![])
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
        &Stmt::Remember(None, vec![Expr::Value(Value::String(String::from("foo")))])
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
        &Stmt::Stumble,
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
        &Stmt::ShambleUntil(
            Expr::Remembering(None, Value::Integer(42)),
            vec![
                Stmt::Say(None, vec![Expr::Value(Value::Integer(1312))]),
                Stmt::Remember(None, vec![Expr::Moan(None)]),
            ]
        )
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
        &Stmt::ShambleUntil(Expr::Remembering(None, Value::Integer(42)), vec![])
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
        &Stmt::Taste(
            Expr::Moan(None),
            vec![
                Stmt::Say(None, vec![Expr::Value(Value::Integer(1312))]),
                Stmt::Remember(None, vec![Expr::Moan(None)]),
            ],
            vec![Stmt::Stumble]
        ),
    );
}

#[test]
fn parse_expressions() {
    init();

    let code = "\
Peter is a zombie
summon
    task Test1
        remember moan X moan moan Y
        remember moan
        remember moan moan moan
        say remembering 69 moan
        say moan Y remembering X 1312
        remember rend turn moan X moan
        remember moan \"X\"
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
        &Stmt::Remember(
            None,
            vec![
                Expr::Moan(Some("X".into())),
                Expr::Moan(None),
                Expr::Moan(Some("Y".into()))
            ]
        )
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
        &Stmt::Remember(None, vec![Expr::Moan(None)])
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
        &Stmt::Remember(
            None,
            vec![Expr::Moan(None), Expr::Moan(None), Expr::Moan(None)]
        ),
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
        &Stmt::Say(
            None,
            vec![
                Expr::Remembering(None, Value::Integer(69)),
                Expr::Moan(None),
            ]
        ),
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
        &Stmt::Say(
            None,
            vec![
                Expr::Moan(Some("Y".into())),
                Expr::Remembering(Some("X".into()), Value::Integer(1312))
            ]
        ),
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
        &Stmt::Remember(
            None,
            vec![
                Expr::Rend,
                Expr::Turn,
                Expr::Moan(Some("X".into())),
                Expr::Moan(None)
            ]
        ),
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
        &Stmt::Remember(
            None,
            vec![
                Expr::Moan(None),
                Expr::Value(Value::String(String::from("X")))
            ]
        ),
    );
}
