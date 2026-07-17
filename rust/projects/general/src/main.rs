fn main() {
    let mut x = 5;
    println!("The value of x is {}", &mut x);
    x = 6; // 这里出现红色下划线的原因是,x没有携带可变标志位mut
    println!("The value of x is {}", x);

    const THREE_HOURES_IN_SECONDS: u32 = 60 * 60 *3; // const 定义的为常量
    // let与const的区别 -- let定义的是一个变量,const定义的是一个常量
    // let定义的是一个不可变变量,let mut 定义的是一个可变变量
    println!("三个小时用秒表示为 {}", THREE_HOURES_IN_SECONDS);

    let y = 5;
    let y = y + 1;
    {
        let y = y * 2;
        println!("The value of x is {}", y);
    }
    println!("The value of x is {}", y); 
    // 这里使用的y为 let y = y + 1 中的y,原来的y已经被修改了
    // 这里的y不是 let y = y * 2; 中的y,因为let y = y * 2;中的y的作用域只包含在{}中

    // let spaces = "   ";
    // spaces = spaces.len();
    // 这里spaces.len()报错的原因如下
    // spaces被定义为&str类型,但len()方法返回的类型为数字(usize),主要问题为类型不匹配
    // 正确写法如下
    let spaces_len; // 这里不使用mut的原因为rust只在确实是需要多次修改时才加mut
    let spaces = "   ";
    spaces_len = spaces.len();
    println!("The value of x is {}", spaces_len);
    let spaces_len_1 = spaces.len();
    println!("The value of x is {}", spaces_len_1);
}
