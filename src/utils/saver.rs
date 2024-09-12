use std::io::{ Write, Seek };
use super::AsU8Slice;

pub struct Saver< T: Write + Seek > {
    to: T
}

impl< T: Write + Seek > Saver< T > {
    pub const fn to(to: T) -> Self {
        Self {
            to
        }
    }

    pub fn save< F: AsU8Slice + ?Sized >(&mut self, from: &F) -> Option< () > {
        let s = from.as_u8_slice();
        self.to.write_all(s).ok()
    }

    pub fn skip(&mut self, n: usize) -> Option< () > {
        self.to.seek(std::io::SeekFrom::Current(n as i64)).ok().map(|_| ())
    }

    pub fn rewind(&mut self, n: usize) -> Option< () > {
        self.to.seek(std::io::SeekFrom::Current(-(n as i64))).ok().map(|_| ())
    }

    pub fn seek(&mut self, n: usize) -> Option< () > {
        self.to.seek(std::io::SeekFrom::Start(n as u64)).ok().map(|_| ())
    }

    pub fn pos(&mut self) -> usize {
        self.to.stream_position().unwrap() as usize
    }
}
