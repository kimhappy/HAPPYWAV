use std::io::{ Read, Seek };
use crate::{ utils::Loader, common::{ AudioFmt, SampleFmt, FileFmt } };

enum RiffType {
    RIFF,
    RF64(Option< (u64, u64) >)
}

pub struct Reader< F: Read + Seek > {
    loader: Loader< F >,
    format: FileFmt    ,
    begin : usize      ,
    end   : usize
}

impl< F: Read + Seek > Reader< F > {
    pub fn from(from: F) -> Option< Reader< F > > {
        let mut loader = Loader::from(from);

        let riff_id       : [u8; 4] = loader.load()?;
        let riff_file_size: u32     = loader.load()?;
        let riff_format_id: [u8; 4] = loader.load()?;

        if &riff_format_id != b"WAVE" {
            return None
        }

        let mut riff = match &riff_id {
            b"RIFF" if riff_file_size as usize == loader.len() => RiffType::RIFF      ,
            b"RF64" if riff_file_size          == 0xFFFFFFFF   => RiffType::RF64(None),
            _                                                  => return None
        };

        let mut format = Option::< FileFmt        >::None;
        let mut be     = Option::< (usize, usize) >::None;

        while loader.end().is_none() {
            let chunk_id: [u8; 4] = loader.load()?;

            match &chunk_id {
                b"ds64" => {
                    if let RiffType::RIFF = riff {
                        return None
                    }

                    let ds64_chunk_size  : u64 = loader.load()?;
                    let ds64_end_pos           = loader.pos  () + ds64_chunk_size as usize;
                    let ds64_file_size   : u64 = loader.load()?;
                    let ds64_data_size   : u64 = loader.load()?;
                    let ds64_sample_count: u64 = loader.load()?;

                    if ds64_file_size as usize != loader.len() {
                        return None
                    }

                    riff = RiffType::RF64(Some((ds64_data_size, ds64_sample_count)));

                    if loader.pos() > ds64_end_pos {
                        return None
                    }

                    loader.seek(ds64_end_pos)?;
                },
                b"fmt " => {
                    let fmt_chunk_size  : u32 = loader.load()?;
                    let fmt_end_pos           = loader.pos  () + fmt_chunk_size as usize;
                    let fmt_audio_format: u16 = loader.load()?;
                    let fmt_num_channels: u16 = loader.load()?;
                    let fmt_sample_rate : u32 = loader.load()?;
                    let fmt_byte_rate   : u32 = loader.load()?;
                    let fmt_block_align : u16 = loader.load()?;
                    let fmt_bit_depth   : u16 = loader.load()?;

                    if fmt_byte_rate != fmt_sample_rate * fmt_num_channels as u32 * fmt_bit_depth as u32 / 8 {
                        return None
                    }

                    if fmt_block_align != fmt_num_channels * fmt_bit_depth as u16 / 8 {
                        return None
                    }

                    let audio_format = AudioFmt::new(fmt_audio_format)?;
                    let sample       = SampleFmt::new(audio_format, fmt_bit_depth)?;
                    format           = Some(FileFmt::new(sample, fmt_num_channels, fmt_sample_rate));

                    if loader.pos() > fmt_end_pos {
                        return None
                    }

                    loader.seek(fmt_end_pos)?;
                },
                b"data" => {
                    let data_chunk_size: u32 = loader.load()?;
                    let data_begin_pos       = loader.pos  () ;
                    let data_chunk_size      = match riff {
                        RiffType::RF64(Some((ds64_data_size, ..))) if data_chunk_size == 0xFFFFFFFF => ds64_data_size  as usize,
                        RiffType::RIFF                                                              => data_chunk_size as usize,
                        _                                                                           => return None
                    };
                    let data_end_pos = data_begin_pos + data_chunk_size;
                    be               = Some((data_begin_pos, data_begin_pos + data_chunk_size));
                    loader.seek(data_end_pos)?;
                },
                _ => {
                    let chunk_size: u32 = loader.load()?;
                    let end_offset      = loader.pos() + chunk_size as usize;
                    loader.seek(end_offset)?;
                },
            }
        }

        let format       = format?;
        let (begin, end) = be    ?;
        let blen         = end - begin;
        let bs           = format.sample_fmt().size() * format.num_channels() as usize;

        if blen % bs != 0 {
            return None
        }

        if let RiffType::RF64(Some((_, sample_count))) = riff && sample_count as usize != blen / bs {
            return None
        }

        Some(Reader { loader, format, begin, end })
    }

    pub const fn len(&self) -> usize {
        (self.end - self.begin) / self.format.sample_fmt().size()
    }

    pub const fn format(&self) -> FileFmt {
        self.format
    }

    pub fn pos(&mut self) -> usize {
        (self.loader.pos() - self.begin) / self.format.sample_fmt().size()
    }

    pub fn skip(&mut self, n: usize) -> Option< () > {
        if self.loader.pos() + n * self.format.sample_fmt().size() <= self.end {
            self.loader.skip(n * self.format.sample_fmt().size())?;
            Some(())
        }
        else {
            None
        }
    }

    pub fn rewind(&mut self, n: usize) -> Option< () > {
        if self.loader.pos() >= self.begin + n * self.format.sample_fmt().size() {
            self.loader.rewind(n * self.format.sample_fmt().size())?;
            Some(())
        }
        else {
            None
        }
    }

    pub fn seek(&mut self, n: usize) -> Option< () > {
        if self.begin + n * self.format.sample_fmt().size() <= self.end {
            self.loader.seek(self.begin + n * self.format.sample_fmt().size())?;
            Some(())
        }
        else {
            None
        }
    }

    pub fn read(&mut self, to: &mut [f32]) -> Option< () > {
        match self.format.sample_fmt() {
            SampleFmt::U8 => {
                const A: f32 = 2.0 / u8::MAX as f32;

                for x in to {
                    let value: u8 = self.loader.load()?;
                    *x = (value as f32 * A) - 1.0;
                }
            },
            SampleFmt::I16 => {
                const A: f32 = 1.0 / i16::MAX as f32;

                for x in to {
                    let value: i16 = self.loader.load()?;
                    *x = value as f32 / A;
                }
            }
            SampleFmt::I24 => {
                const A: f32 = 1.0 / 0x7FFFFF as f32;

                for x in to {
                    let bytes: [u8; 3] = self.loader.load()?;
                    let third          = (bytes[ 2 ] >> 7) * 0xFF;
                    let value          = i32::from_le_bytes([bytes[ 0 ], bytes[ 1 ], bytes[ 2 ], third]);
                    *x = value as f32 * A;
                }
            }
            SampleFmt::I32 => {
                const A: f32 = 1.0 / i32::MAX as f32;

                for x in to {
                    let value: i32 = self.loader.load()?;
                    *x = value as f32 * A;
                }
            }
            SampleFmt::F32 => {
                self.loader.load_to(to)?;
            }
            SampleFmt::F64 => {
                for x in to {
                    let value: f64 = self.loader.load()?;
                    *x = value as f32;
                }
            }
        }

        Some(())
    }
}
