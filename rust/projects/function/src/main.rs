// 一个带参数的函数定义
fn add_man(a: f32, b: f32) -> f32 { // 在rust中, 函数定义时需要指定返回值的类型
    let c: f32 = a + b;
    return c;
}

fn main() {
    let a: f32 = 10.0;
    let b: f32 = 20.0;
    let c = add_man(a, b);
    println!("a + b = {}", c);

    let d: i32 = 98;
    test_if(d);

    // 将if的返回值赋值给变量
    // if分支的返回值必须为同一类型
    let codition = true;
    let number = if codition {5} else {6};
    println!("number 的值为{}", number);

    let cycle_num = 10;
    test_cycle(cycle_num);

}


fn test_if(num: i32) {
    // 测试if
    if num > 0 {
        println!("当前输入的数字为{}", num);
    } else {
        println!("当前输入的数字小于0!")
    }

}

fn test_cycle(cycle_num: i32) {
    // rust的循环有三种方式
    // loop 循环
    let mut i: i32 = 0;
    loop {
        if i > cycle_num {
            println!("loop循环到此为止");
            break;
        } else {
            i = i + 1;
            println!("loop循环{}", i);
        }
    };

    // while 循环
    let mut i_while: i32 = 0;
    while i_while < 10 {
        i_while = i_while + 1;
        println!("while循环{}", i_while);
    }
    println!("while循环到此为止");

    // for 循环
    for i_for in (1..4).rev() {
        println!("{}", i_for);
    }
    println!("for循环到此为止");
}
