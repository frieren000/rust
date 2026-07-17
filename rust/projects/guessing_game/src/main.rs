// 猜数字游戏
// rust编译器十分严格,未使用的导入也会导致检查报错
use std::io; // std::io为标准库中的输入输出功能
use rand::prelude::*; // 随机数功能
use std::cmp::Ordering; // 枚举功能
fn main() {
    println!("Guess the number!");

    let secret_number = rand::rng().random_range(1..101);

    loop {
        // rust中,使用loop表示循环
        println!("生成的随机数为:{}", secret_number);

        println!("please input your Guess");

        let mut guess = String::new(); // 携带了mnt,代表可变参数
                                       // rust中使用let创建变量,mut表示代表可变参数,不携带mut代表不可变参数
                                       // let apples = 5; -- 不可变参数
                                       // let mnt apples = 5; -- 可变参数
                                       // Type::fn() -- Type指定了数据的类型,fn()指定了所使用的方法

        io::stdin() // 这里调用了标准库中的用户输入功能
            .read_line(&mut guess) // &表示是一个引用,引用默认是不可变的,需要携带可变标志mnt使引用可变
            .expect("Falied to read_line"); // 处理可能的错误,使用了Result类型

        println!("You guessed :{}", guess);

        let guess: u32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };
        // 定义一个新的guess,u32代表32为无符号整数
        // trim() -- 去除字符串开头和结尾的空白字符
        // parse() -- 将字符串转化为数字 -- 需要指定数字类型
        // rust中同样存在if,但优先考虑match

        match guess.cmp(&secret_number) {
            // 这里出现下划线的原因是因为guess是字符串类型,而secret_number是整数类型
            Ordering::Less => println!("Too Small!"),
            Ordering::Greater => println!("Too Big!"),
            Ordering::Equal => {
                println!("You win!");
                break;
            }
        }
    }
}
