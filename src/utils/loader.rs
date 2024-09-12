use std::{ io::{ Read, Seek }, mem::MaybeUninit };
use super::{ AsU8Slice, Pod };

pub struct Loader< F: Read + Seek > {
    from: F
}

impl< F: Read + Seek > Loader< F > {
    pub const fn from(from: F) -> Self {
        Self {
            from
        }
    }

    pub fn load_to< T: AsU8Slice + ?Sized >(&mut self, to: &mut T) -> Option< () > {
        let s = to.as_mut_u8_slice();
        self.from.read_exact(s).ok()
    }

    pub fn load< T: Pod >(&mut self) -> Option< T > {
        let mut ret = unsafe { MaybeUninit::uninit().assume_init() };
        self.load_to(&mut ret).map(|_| ret)
    }

    pub fn skip(&mut self, n: usize) -> Option< () > {
        self.from.seek(std::io::SeekFrom::Current(n as i64)).ok().map(|_| ())
    }

    pub fn rewind(&mut self, n: usize) -> Option< () > {
        self.from.seek(std::io::SeekFrom::Current(-(n as i64))).ok().map(|_| ())
    }

    pub fn seek(&mut self, n: usize) -> Option< () > {
        self.from.seek(std::io::SeekFrom::Start(n as u64)).ok().map(|_| ())
    }

    pub fn pos(&mut self) -> usize {
        self.from.stream_position().unwrap() as usize
    }

    pub fn len(&mut self) -> usize {
        self.from.stream_len().unwrap() as usize
    }

    pub fn end(&mut self) -> Option< () > {
        if self.pos() == self.len() {
            Some(())
        }
        else {
            None
        }
    }
}
