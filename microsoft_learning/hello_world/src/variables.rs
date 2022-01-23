//This file is not used.
fn variable_tests() {
    //println!("Hello, world!");
    let x = 5;
    println!("The value of x is: {}", x);
    let y = true;
    println!("The value of y is: {}", y);
    //Shadow a variable
    let x = x + 1;
    println!("The value of x is: {}", x);

    const STRING: &str = "hello";
    println!("The value of the string constant is: {}", STRING);

    let _array = [1u32, 2, 3];

    let _tuple = (1u32, 3, true);
}