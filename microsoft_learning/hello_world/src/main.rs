//use std::fs::File;

mod variables;

struct Person {
    name: String,
    age: u8,
    likes_oranges: bool,
}

#[derive(Eq, PartialEq)]
struct Student {
    name: String,
}

enum WebEvents {
    PageLoad,
    PageUnload,
    KeyPress(char),
    Paste(String),
    Click {x: i64, y:i64},
}

enum Option<T> {
    Some(T),
    None,
}

struct Point2D(u32, u32);

fn main() {
    //println!("Hello, world!");
    println!("{}", last_char(String::from("Hello")));

    println!("Person name is: {:?}, age is: {:?} and likes orange: {:?}", learn_class_struct().name, learn_class_struct().age, learn_class_struct().likes_oranges);

    //Destructuring tuple struct. Interesting, isn't it?
    let Point2D(x, y) = learn_tuple_struct();
    println!("Point contains {:?} and {:?}", x, y);

    error_handling();

    let mut my_vec = vec![1, 2, 3];
    println!("{:?}", my_vec);
    add_to_vec(&mut my_vec); // Note this. mutatable
    println!("{:?}", my_vec);

    let text = "Hello\nworld\n!";
    println!("{}", first_line(text));

    let mut students = vec![Student{
        name: "Ryan".to_string()
    }];

    students.push(Student{
        name: "Lisa".to_string(),
    });

    assert!(&students[0] == &Student{name: "Ryan".to_string()}); //Use get method

}

pub fn first_line(string: &str) -> &str {
    string.lines().next().unwrap()
}

fn add_to_vec(a_vec: &mut Vec<i32>){
    a_vec.push(4);
}

fn print_out(to_print: String){
    println!("{}", to_print);
}

fn last_char(string: String) -> char{
    if string.is_empty() {
        return 'c';
    }
    string.chars().next_back().unwrap()
}

fn learn_class_struct() -> Person {
    Person{
        name: String::from("Adam"),
        age : 25,
        likes_oranges :true,
    }
}

fn learn_tuple_struct() -> Point2D {
    Point2D(100, 200)
}

fn learn_enum() {
    let _quit = WebEvents::KeyPress('q');

    let _something = Some(1);
}

fn error_handling() {
    //panic!("Farewell");

    let v = vec![0, 1, 2, 3];
    //println!("{}", v[6]);
    println!("{:?}", v.get(0));

    println!("{:?}", v.get(99));

    /*let f = File::open("hello.txt");
    let f = match f {
        Ok(file) => file,
        //Err(error) => panic!("Can't open: {:?}", error),
        Err(error) => panic!("Can't open: {:?}", error),
    };*/
}