fn main() {
    // 数据类型
    // 1. 整型(i代表有符号,u代表无符号) -- 默认值 -- i32(有符号的32位数字)
    // 8位 -- i8/u8
    // 16位 -- i16/u16
    // 32位 -- i32/u32
    // 64位 -- i64/u64
    // 128位 -- i128/u128
    // arch -- isize/usize
    // 整型溢出的处理方法
    // a. 使用warpping_*进行包裹 -- 自动进行二进制补码转换
    // b. 使用checked_* -- 发生溢出时返回None
    // c. 使用overflowing_* -- 返回该值以及一个是否溢出的布尔值
    // d. 使用saturating_* -- 使值达到最小值或最大值
    // 2. 浮点数
    // 32位 -- f32
    // 64位 -- f64
    // 3. bool类型 -- true/false
    // 4. 字符类型 -- char
    // 5. 复合类型 
    // 5.1 元组类型 -- tup -- ()
    // 5.2 数组类型 -- array -- rust中的数组具有固定的长度 -- []

    // 元组示例
    let  a = (500, 6.4, 1);
    let (x, y, z) = a; // 对元组a进行解构,必须携带let
    println!("the value of x is {}", x);
    println!("the value of y is {}", y);
    println!("the value of z is {}", z);

    let x: (i8, i32, f64, u8) = (100, 500, 6.4, 1);
    let i_8 = x.0;
    println!("the value of i_8 is {}", i_8);
    let i_32 = x.1;
    println!("the value of i_32 is {}", i_32);
    let f_64 = x.2;
    println!("the value of f_64 is {}", f_64);
    let u_8 = x.3;
    println!("the value of u_8 is {}", u_8);
    // 在rust中,使用"."访问元组中索引所对应的值

    // 数组示例 -- 数组中的元素必须为同一类型,且长度不可变
    // 动态数组的定义方式为 -- vector
    let  b = [1, 2, 3, 4, 5]; // 不可变数组
    let c = vec![1, 2, 3, 4, 5, 6]; // 动态数组 -- 动态数组中,元素的类型必须一致

    let b_1 = b[0];
    println!("the value of b_1 is {}", b_1);
    let b_2 = b[1];
    println!("the value of b_2 is {}", b_2);
    let b_3 = b[2];
    println!("the value of b_3 is {}", b_3);
    let b_4 = b[3];
    println!("the value of b_4 is {}", b_4);
    let b_5: i32 = b[4];
    println!("the value of b_5 is {}", b_5);

    let c_1 = c[0];
    println!("the value of c_1 is {}", c_1);
    let c_2 = c[1];
    println!("the value of c_2 is {}", c_2);
    let c_3 = c[2];
    println!("the value of c_3 is {}", c_3);
    let c_4 = c[3];
    println!("the value of c_4 is {}", c_4);
    let c_5 = c[4];
    println!("the value of c_5 is {}", c_5);
    let c_6 = c[5];
    println!("the value of c_6 is {}", c_6);

    // 无效的数组访问如下
    // let c_7 = c[6];
    // println!("the value of c_6 is {}", c_7);
    // 编译时不会出现错误,但运行后会提示以下内容
    // thread 'main' (2432290) panicked at src/main.rs:72:16:
    // index out of bounds: the len is 6 but the index is 6
    // note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
    
    
}
