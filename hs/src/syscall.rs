// use core::{arch::asm, prelude};


// fn syscall(id: usize, args: [usize; 3]) -> isize {
//     let mut ret: isize;
//     println!("ecall");
//     unsafe {
//         asm!(
//             "ecall",
//             inlateout("x10") args[0] => ret,
//             in("x11") args[1],
//             in("x12") args[2],
//             in("x17") id
//         );
//     }
//     ret
// }

// pub fn sys_wake(this_id: usize,dtb: usize) -> isize {
//     println!("sys wake ");
//     syscall(1, [this_id, dtb, 0])
// }
