use std::io::{ Write, Seek };
use crate::{ utils::Saver, common::{ FileFmt, SampleFmt } };

enum RiffType {
    RIFF,
    RF64
}

const RIFF_HEADER_SIZE: usize = 12;
const DS64_CHUNK_SIZE : usize = 36;
const FMT_CHUNK_SIZE  : usize = 24;

pub struct Writer< T: Write + Seek > {
    saver  : Saver< T >,
    rt     : RiffType  ,
    format : FileFmt   ,
    max_pos: usize
}

impl< T: Write + Seek > Writer< T > {
    pub fn to(to: T, format: FileFmt) -> Option< Writer< T > > {
        let mut saver = Saver::to(to);

        saver.save(b"RIFF")?;
        saver.skip(4)?; // File size
        saver.save(b"WAVE")?;

        saver.save(b"fmt ")?;
        saver.save(&(FMT_CHUNK_SIZE as u32 - 8))?;
        saver.save(&(format.sample_fmt().audio_format() as u16))?;
        saver.save(& format             .num_channels())?;
        saver.save(& format             .sample_rate ())?;
        saver.save(& format             .byte_rate   ())?;
        saver.save(& format             .block_align ())?;
        saver.save(& format.sample_fmt().bit_depth   ())?;

        saver.save(b"data")?;
        saver.skip(4)?; // Data size

        let max_pos = saver.pos();
        Some(Writer { saver, rt: RiffType::RIFF, format, max_pos })
    }

    pub fn to_rf64(to: T, format: FileFmt) -> Option< Writer< T > > {
        let mut saver = Saver::to(to);

        saver.save(b"RF64")?;
        saver.skip(0xFFFFFFFF)?;
        saver.save(b"WAVE")?;

        saver.save(b"ds64")?;
        saver.save(&(DS64_CHUNK_SIZE as u64 - 8))?;
        saver.skip(8)?; // File size
        saver.skip(8)?; // Data size
        saver.skip(8)?; // Sample count

        saver.save(b"fmt ")?;
        saver.save(&(FMT_CHUNK_SIZE as u32 - 8))?;
        saver.save(&(format.sample_fmt().audio_format() as u16))?;
        saver.save(& format             .num_channels())?;
        saver.save(& format             .sample_rate ())?;
        saver.save(& format             .byte_rate   ())?;
        saver.save(& format             .block_align ())?;
        saver.save(& format.sample_fmt().bit_depth   ())?;

        saver.save(b"data")?;
        saver.skip(0xFFFFFFFF)?;

        let max_pos = saver.pos();
        Some(Writer { saver, rt: RiffType::RF64, format, max_pos })
    }

    const fn data_begin(&self) -> usize {
        match self.rt {
            RiffType::RIFF => RIFF_HEADER_SIZE                   + FMT_CHUNK_SIZE + 8,
            RiffType::RF64 => RIFF_HEADER_SIZE + DS64_CHUNK_SIZE + FMT_CHUNK_SIZE + 8
        }
    }

    const fn file_size(&self) -> usize {
        self.max_pos
    }

    const fn data_size(&self) -> usize {
        self.max_pos - self.data_begin()
    }

    pub fn pos(&mut self) -> usize {
        (self.saver.pos() - self.data_begin()) / self.format.sample_fmt().size() as usize
    }

    pub const fn len(&self) -> usize {
        self.data_size() / self.format.sample_fmt().size() as usize
    }

    pub const fn format(&self) -> FileFmt {
        self.format
    }

    pub fn skip(&mut self, n: usize) -> Option< () > {
        self.saver.skip(n * self.format.sample_fmt().size())?;
        self.max_pos = self.max_pos.max(self.saver.pos());
        Some(())
    }

    pub fn rewind(&mut self, n: usize) -> Option< () > {
        if self.saver.pos() - n * self.format.sample_fmt().size() >= self.data_begin() {
            self.saver.rewind(n * self.format.sample_fmt().size())?;
            Some(())
        }
        else {
            None
        }
    }

    pub fn seek(&mut self, n: usize) -> Option< () > {
        self.saver.seek(self.data_begin() + n * self.format.sample_fmt().size())?;
        self.max_pos = self.max_pos.max(self.saver.pos());
        Some(())
    }

    pub fn write(&mut self, from: &[f32]) -> Option< () > {
        match self.format.sample_fmt() {
            SampleFmt::U8 => {
                const A: f32 = u8::MAX as f32 / 2.0;

                for x in from {
                    let value = ((x + 1.0) * A) as u8;
                    self.saver.save(&value);
                }
            },
            SampleFmt::I16 => {
                const A: f32 = i16::MAX as f32;

                for x in from {
                    let value = (x * A) as i16;
                    self.saver.save(&value);
                }
            },
            SampleFmt::I24 => {
                const A: f32 = 0x7FFFFF as f32;

                for x in from {
                    let value = (x * A) as i32;
                    let value = [
                        (value >>  0) as u8,
                        (value >>  8) as u8,
                        (value >> 16) as u8
                    ];
                    self.saver.save(&value);
                }
            },
            SampleFmt::I32 => {
                const A: f32 = i32::MAX as f32;

                for x in from {
                    let value = (x * A) as i32;
                    self.saver.save(&value);
                }
            },
            SampleFmt::F32 => {
                self.saver.save(from);
            },
            SampleFmt::F64 => {
                for x in from {
                    let value = *x as f64;
                    self.saver.save(&value);
                }
            }
        }

        self.max_pos = self.max_pos.max(self.saver.pos());
        Some(())
    }

    pub fn finalize(&mut self) -> Option< () > {
        match self.rt {
            RiffType::RIFF => {
                self.saver.seek(4);
                self.saver.save(&(self.file_size() as u32))?;
                self.saver.seek(RIFF_HEADER_SIZE + FMT_CHUNK_SIZE + 4);
                self.saver.save(&(self.data_size() as u32))?
            },
            RiffType::RF64 => {
                self.saver.seek(RIFF_HEADER_SIZE + 12);
                self.saver.save(&(self.file_size() as u64))?;
                self.saver.save(&(self.data_size() as u64))?;
                self.saver.save(&(self.len      () as u64))?;
            }
        }

        Some(())
    }
}

impl< T: Write + Seek > Drop for Writer< T > {
    fn drop(&mut self) {
        self.finalize().unwrap();
    }
}
