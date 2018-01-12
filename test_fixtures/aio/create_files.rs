use std::fs::File;
use std::mem;
use std::io::Write;

fn main() {
  let mut file = File::create("./small.txt").unwrap();
  write!(file, "try reading this with direct IO").unwrap();

  let mut file = File::create("./u16-inc-small.bin").unwrap();
  file.set_len(0).unwrap();
  //write 9kb of increasing u16
  for counter in 0 .. 4608u16 {
    let bytes = unsafe { mem::transmute::<u16, [u8; 2]>(counter) };
    file.write(&bytes).unwrap();
  }

  let mut file = File::create("./u32-inc-big.bin").unwrap();
  file.set_len(0).unwrap();
  //write 1 mb of increasing u32
  for counter in 0 .. (1024u32 * 1024u32) / 4u32 {
    let bytes = unsafe { mem::transmute::<u32, [u8; 4]>(counter) };
    file.write(&bytes).unwrap();
  }
}
