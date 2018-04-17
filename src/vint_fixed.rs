

// #[repr(C)]
// union VIntData {
//     bytes: [u8; 4],
//     val: u32,
// }

// #[derive(Debug, Clone, Default)]
// pub struct VIntArrayFixed{
//     pub data: Vec<u8>,
// }

// impl VIntArrayFixed {
//     pub fn encode_vals(&mut self, vals: &[u32]) {
//         for val in vals{
//             self.encode(*val);
//         }
//     }
//     pub fn encode(&mut self, val:u32) {
//         unsafe{
//             self.data.reserve(4);
//             let mut el = VIntData{val};
//             el.val <<= 2; // shift two to the left, to make space for the signal flags

//             if val < 1 << 6 { //64
//                 self.data.push(el.bytes[0]);
//             }else if val < 1 << 14 {
//                 el.val |= 1;
//                 self.data.push(el.bytes[0]);
//                 self.data.push(el.bytes[1]);
//             }else if val < 1 << 22 {
//                 el.val |= 2;
//                 self.data.push(el.bytes[0]);
//                 self.data.push(el.bytes[1]);
//                 self.data.push(el.bytes[2]);
//             }else{
//                 el.val |= 3;
//                 self.data.push(el.bytes[0]);
//                 self.data.push(el.bytes[1]);
//                 self.data.push(el.bytes[2]);
//                 self.data.push(el.bytes[3]);
//             };

//         }

//     }

//     pub fn iter(& self) -> VintArrayFixedIterator {
//         VintArrayFixedIterator {
//             list: &self,
//             pos: 0,
//             len: self.data.len(),
//         }
//     }

// }

// #[derive(Debug, Clone)]
// pub struct VintArrayFixedIterator<'a>  {
//     list: & 'a VIntArrayFixed,
//     pos:usize,
//     len:usize
// }

// impl<'a> Iterator for VintArrayFixedIterator<'a> {
//     type Item = u32;

//     #[inline]
//     fn next(&mut self) -> Option<u32> {
//         unsafe{
//             if self.pos == self.len {
//                 None
//             }else {

//                 let mut val = *self.list.data.get_unchecked(self.pos);
//                 let flags:u8 = val & 0b0000_0011;

//                 if flags == 0 {
//                     val >>= 2;
//                     self.pos += 1;
//                     Some(val as u32)
//                 } else if flags == 1 {
//                     let mut el = VIntData{val: 0};
//                     el.bytes[..=1].copy_from_slice(&self.list.data[self.pos..=self.pos+1]);
//                     el.val >>= 2;
//                     self.pos += 2;
//                     Some(el.val)
//                 } else if flags == 2 {
//                     let mut el = VIntData{val: 0};
//                     el.bytes[..=2].copy_from_slice(&self.list.data[self.pos..=self.pos+2]);
//                     el.val >>= 2;
//                     self.pos += 3;
//                     Some(el.val)
//                 }else {
//                     let mut el = VIntData{val: 0};
//                     el.bytes[..=3].copy_from_slice(&self.list.data[self.pos..=self.pos+3]);
//                     el.val >>= 2;
//                     self.pos += 4;
//                     Some(el.val)
//                 }
//             }
//         }

//     }

//     #[inline]
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         (self.len-self.pos / 2, Some(self.len-self.pos))
//     }

// }

