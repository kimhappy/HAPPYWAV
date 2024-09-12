#[repr(u16)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AudioFmt {
    Pcm       = 1,
    IeeeFloat = 3
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SampleFmt {
    U8 ,
    I16,
    I24,
    I32,
    F32,
    F64
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FileFmt {
    sample_fmt  : SampleFmt,
    num_channels: u16      ,
    sample_rate : u32
}

impl AudioFmt {
    pub const fn new(n: u16) -> Option< Self > {
        match n {
            1 => Some(AudioFmt::Pcm      ),
            3 => Some(AudioFmt::IeeeFloat),
            _ => None
        }
    }
}

impl SampleFmt {
    pub const fn new(audio_format: AudioFmt, bit_depth: u16) -> Option< Self > {
        match (audio_format, bit_depth) {
            (AudioFmt::Pcm      ,  8) => Some(SampleFmt::U8 ),
            (AudioFmt::Pcm      , 16) => Some(SampleFmt::I16),
            (AudioFmt::Pcm      , 24) => Some(SampleFmt::I24),
            (AudioFmt::Pcm      , 32) => Some(SampleFmt::I32),
            (AudioFmt::IeeeFloat, 32) => Some(SampleFmt::F32),
            (AudioFmt::IeeeFloat, 64) => Some(SampleFmt::F64),
            _                         => None
        }
    }

    pub const fn bit_depth(&self) -> u16 {
        match self {
            SampleFmt::U8  =>  8,
            SampleFmt::I16 => 16,
            SampleFmt::I24 => 24,
            SampleFmt::I32 => 32,
            SampleFmt::F32 => 32,
            SampleFmt::F64 => 64
        }
    }

    pub const fn size(&self) -> usize {
        match self {
            SampleFmt::U8  => 1,
            SampleFmt::I16 => 2,
            SampleFmt::I24 => 3,
            SampleFmt::I32 => 4,
            SampleFmt::F32 => 4,
            SampleFmt::F64 => 8
        }
    }

    pub const fn audio_format(&self) -> AudioFmt {
        match self {
            SampleFmt::U8  => AudioFmt::Pcm      ,
            SampleFmt::I16 => AudioFmt::Pcm      ,
            SampleFmt::I24 => AudioFmt::Pcm      ,
            SampleFmt::I32 => AudioFmt::Pcm      ,
            SampleFmt::F32 => AudioFmt::IeeeFloat,
            SampleFmt::F64 => AudioFmt::IeeeFloat
        }
    }
}

impl FileFmt {
    pub const fn new(sample_fmt: SampleFmt, num_channels: u16, sample_rate: u32) -> Self {
        Self {
            sample_fmt  ,
            num_channels,
            sample_rate
        }
    }

    pub const fn sample_fmt(&self) -> SampleFmt {
        self.sample_fmt
    }

    pub const fn num_channels(&self) -> u16 {
        self.num_channels
    }

    pub const fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub const fn byte_rate(&self) -> u32 {
        self.sample_rate * self.num_channels as u32 * self.sample_fmt.bit_depth() as u32 / 8
    }

    pub const fn block_align(&self) -> u16 {
        self.num_channels * self.sample_fmt.size() as u16
    }
}

impl std::fmt::Display for SampleFmt {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SampleFmt::U8  => write!(f, "U8" ),
            SampleFmt::I16 => write!(f, "I16"),
            SampleFmt::I24 => write!(f, "I24"),
            SampleFmt::I32 => write!(f, "I32"),
            SampleFmt::F32 => write!(f, "F32"),
            SampleFmt::F64 => write!(f, "F64")
        }
    }
}

impl std::fmt::Display for FileFmt {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {}Hz {}ch", self.sample_fmt, self.sample_rate, self.num_channels)
    }
}
