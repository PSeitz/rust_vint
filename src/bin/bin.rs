use std::mem::transmute;
fn main()  {
    let bytes: [u8; 4] = unsafe { transmute(1 as u32) };
    println!("{:?}", &bytes[0..2]);

}